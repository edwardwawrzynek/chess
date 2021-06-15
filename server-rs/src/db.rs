use crate::apikey::ApiKey;
use crate::diesel::prelude::*;
use crate::error::Error;
use crate::games::{Fmt, GameInstance, GameState, GameTurn, GameType};
use crate::models::{
    DBGame, GameId, GamePlayer, GamePlayerId, NewDBGame, NewGamePlayer, NewUser, User, UserId,
};
use crate::schema::{game_players, games, users};
use bcrypt;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use std::collections::HashMap;

impl User {
    pub fn check_password(&self, password: &str) -> bool {
        match self.password_hash.as_deref() {
            None => false,
            Some(hash) => match bcrypt::verify(password.as_bytes(), &hash) {
                Ok(true) => true,
                _ => false,
            },
        }
    }
}

// mapping from game type string to GameType
pub type GameTypeMap = HashMap<&'static str, Box<dyn GameType>>;

// in memory representation of a game
pub struct Game {
    pub id: GameId,
    pub owner_id: UserId,
    pub game_type: String,
    pub instance: Option<Box<dyn GameInstance>>,
}

pub type GameAndPlayers = (Game, Vec<GamePlayer>);

impl Game {
    fn from_dbgame(game: DBGame, type_map: &GameTypeMap, players: &[GamePlayerId]) -> Game {
        let instance = if let Some(ref state) = game.state {
            type_map[&*game.game_type].deserialize(state, players)
        } else {
            None
        };

        Game {
            id: game.id,
            owner_id: game.owner_id,
            game_type: game.game_type,
            instance,
        }
    }

    fn to_dbgame(&self) -> DBGame {
        let (finished, winner, is_tie) = match &self.instance {
            Some(instance) => match instance.turn() {
                GameTurn::Finished => {
                    if let Some(end_state) = instance.end_state() {
                        match end_state {
                            GameState::Win(uid) => (true, Some(uid), Some(false)),
                            GameState::Tie => (true, None, Some(true)),
                            GameState::InProgress => (false, None, None),
                        }
                    } else {
                        (true, None, None)
                    }
                }
                _ => (false, None, None),
            },
            None => (false, None, None),
        };

        DBGame {
            id: self.id,
            owner_id: self.owner_id,
            game_type: self.game_type.clone(),
            state: self
                .instance
                .as_ref()
                .and_then(|i| Some(format!("{}", Fmt(|f| i.serialize(f))))),
            finished,
            winner,
            is_tie,
        }
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db_pool(db_url: &str) -> Result<PgPool, PoolError> {
    let manage = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder().build(manage)
}

/// A database connection wrapper, which associates the database with functions to manipulate it
pub struct DBWrapper<'a, F: Fn(&Game, &Vec<GamePlayer>)> {
    db: PooledConnection<ConnectionManager<PgConnection>>,
    game_type_map: &'a GameTypeMap,
    game_update_callback: F,
}

impl<F: Fn(&Game, &Vec<GamePlayer>)> DBWrapper<'_, F> {
    /// Wrap an existing pg connection
    pub fn from_pg_pool<'a>(
        pool: &PgPool,
        type_map: &'a GameTypeMap,
        game_update_callback: F,
    ) -> Result<DBWrapper<'a, F>, Error> {
        Ok(DBWrapper {
            db: pool.get()?,
            game_type_map: type_map,
            game_update_callback,
        })
    }

    // ---- Users ----

    /// Lookup a user with the given id
    pub fn find_user(&self, id: UserId) -> Result<User, Error> {
        match users::dsl::users
            .find(id)
            .first::<User>(&self.db)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUser),
        }
    }

    /// Lookup user by api key
    pub fn find_user_by_apikey(&self, key: &ApiKey) -> Result<User, Error> {
        let hashed = key.hash();
        match users::dsl::users
            .filter(users::dsl::api_key_hash.eq(hashed.to_string()))
            .first::<User>(&self.db)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::InvalidApiKey),
        }
    }

    /// Lookup user by email
    fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        match users::dsl::users
            .filter(users::dsl::email.eq(email))
            .first::<User>(&self.db)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUser),
        }
    }

    /// Lookup user by email and password
    pub fn find_user_by_credentials(&self, email: &str, pass: &str) -> Result<User, Error> {
        let user = self.find_user_by_email(email)?;
        match user.check_password(pass) {
            true => Ok(user),
            false => Err(Error::IncorrectCredentials),
        }
    }

    /// Insert a new user into the db
    fn insert_user(&self, user: NewUser) -> Result<User, Error> {
        Ok(diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(&self.db)?)
    }

    /// Create a new user with given info
    pub fn new_user(&self, name: &str, email: &str, pass: &str) -> Result<User, Error> {
        // check for existing user
        match self.find_user_by_email(email) {
            Ok(_) => Err(Error::EmailAlreadyTaken),
            Err(Error::NoSuchUser) => {
                let hashed_pass = bcrypt::hash(pass.as_bytes(), bcrypt::DEFAULT_COST)?;
                let user = NewUser {
                    name,
                    email: Some(email),
                    is_admin: false,
                    password_hash: Some(&*hashed_pass),
                    api_key_hash: &*ApiKey::new().hash().to_string(),
                };
                self.insert_user(user)
            }
            Err(err) => Err(err),
        }
    }

    /// Create a new user with no login credentials
    pub fn new_tmp_user(&self, name: &str) -> Result<User, Error> {
        let user = NewUser {
            name,
            email: None,
            is_admin: false,
            password_hash: None,
            api_key_hash: &*ApiKey::new().hash().to_string(),
        };
        self.insert_user(user)
    }

    /// Update a user already in the db
    pub fn save_user(&self, user: &User) -> Result<(), Error> {
        diesel::update(users::dsl::users.find(user.id))
            .set(user)
            .execute(&self.db)?;
        Ok(())
    }

    // ---- Games ----
    /// Create a new game with the given type
    pub fn new_game(&self, game_type: &str, owner: UserId) -> Result<DBGame, Error> {
        if !self.game_type_map.contains_key(game_type) {
            return Err(Error::NoSuchGameType(game_type.to_string()));
        }
        let game = NewDBGame {
            game_type,
            state: None,
            owner_id: owner,
            winner: None,
            finished: false,
            is_tie: None,
        };
        Ok(diesel::insert_into(games::table)
            .values(&game)
            .get_result::<DBGame>(&self.db)?)
    }

    /// Load a DBGame from the database
    fn find_dbgame(&self, id: GameId) -> Result<DBGame, Error> {
        match games::dsl::games
            .find(id)
            .first::<DBGame>(&self.db)
            .optional()?
        {
            Some(game) => Ok(game),
            None => Err(Error::NoSuchGame),
        }
    }

    /// Load a game and it's players from the database
    pub fn find_game(&self, id: GameId) -> Result<GameAndPlayers, Error> {
        self.dbgame_to_game_and_players(self.find_dbgame(id)?, self.game_type_map)
    }

    /// Load all players in a game
    pub fn find_game_players(&self, game_id: GameId) -> Result<Vec<GamePlayer>, Error> {
        use game_players::dsl;
        Ok(dsl::game_players
            .filter(dsl::game_id.eq(game_id))
            .load::<GamePlayer>(&self.db)?)
    }

    /// Convert a DBGame -> Game + GamePlayers
    fn dbgame_to_game_and_players(
        &self,
        game: DBGame,
        type_map: &GameTypeMap,
    ) -> Result<GameAndPlayers, Error> {
        let players = self.find_game_players(game.id)?;
        let player_ids = (&players)
            .iter()
            .map(|p| p.user_id)
            .collect::<Vec<UserId>>();
        let game_mem = Game::from_dbgame(game, type_map, &*player_ids);
        Ok((game_mem, players))
    }

    fn find_game_player(&self, game_id: GameId, user_id: UserId) -> Result<GamePlayer, Error> {
        use game_players::dsl;
        match dsl::game_players
            .filter(dsl::game_id.eq(game_id).and(dsl::user_id.eq(user_id)))
            .first::<GamePlayer>(&self.db)
            .optional()?
        {
            Some(player) => Ok(player),
            None => Err(Error::NotInGame),
        }
    }

    /// Check if a user is a player in a game
    fn user_in_game(&self, game_id: GameId, user_id: UserId) -> Result<bool, Error> {
        match self.find_game_player(game_id, user_id) {
            Ok(_) => Ok(true),
            Err(Error::NotInGame) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Add a user as a player in a game
    pub fn join_game(&self, game_id: GameId, user_id: UserId) -> Result<GamePlayer, Error> {
        if self.user_in_game(game_id, user_id)? {
            return Err(Error::AlreadyInGame);
        }
        let (game, mut players) = self.find_game(game_id)?;
        if let Some(_) = game.instance {
            return Err(Error::GameAlreadyStarted);
        }

        let player = NewGamePlayer {
            game_id,
            user_id,
            score: None,
        };
        let new_player = diesel::insert_into(game_players::table)
            .values(&player)
            .get_result::<GamePlayer>(&self.db)?;

        players.push(new_player);
        (self.game_update_callback)(&game, &players);

        Ok(players.pop().unwrap())
    }

    /// Remove a user as a player in a game
    pub fn leave_game(&self, game_id: GameId, user_id: UserId) -> Result<(), Error> {
        use game_players::dsl;
        let player = self.find_game_player(game_id, user_id)?;
        let (game, mut players) = self.find_game(game_id)?;
        if let Some(_) = game.instance {
            return Err(Error::GameAlreadyStarted);
        }

        diesel::delete(dsl::game_players.filter(dsl::id.eq(player.id))).execute(&self.db)?;

        if let Some(index) = players.iter().position(|p| p.user_id == user_id) {
            players.remove(index);
        }
        (self.game_update_callback)(&game, &players);
        Ok(())
    }

    /// Update a DBGame in the database
    fn save_dbgame(&self, game: &DBGame) -> Result<(), Error> {
        diesel::update(games::dsl::games.find(game.id))
            .set(game)
            .execute(&self.db)?;
        Ok(())
    }

    /// Save a GamePlayer in the database
    pub fn save_game_player(&self, game_player: &GamePlayer) -> Result<(), Error> {
        diesel::update(game_players::dsl::game_players.find(game_player.id))
            .set(game_player)
            .execute(&self.db)?;
        Ok(())
    }

    /// Update a game and its player's scores in the database
    pub fn save_game(&self, game: &Game) -> Result<(), Error> {
        self.save_dbgame(&game.to_dbgame())?;
        let mut players = self.find_game_players(game.id)?;
        if let Some(instance) = &game.instance {
            if let Some(scores) = instance.scores() {
                for player in &mut players {
                    player.score = Some(scores[&player.id]);
                    self.save_game_player(player)?;
                }
            }
        }
        (self.game_update_callback)(game, &players);
        Ok(())
    }

    /// Start a game as the given user
    pub fn start_game(&self, game_id: GameId, user_id: UserId) -> Result<(), Error> {
        let (mut game, players) = self.find_game(game_id)?;
        let player_ids = (&players)
            .iter()
            .map(|p| p.user_id)
            .collect::<Vec<UserId>>();
        if game.owner_id != user_id {
            return Err(Error::DontOwnGame);
        }
        if let Some(_) = game.instance {
            return Err(Error::GameAlreadyStarted);
        }

        let new_instance = self.game_type_map[&*game.game_type].new(&player_ids);

        match new_instance {
            Some(new_instance) => {
                game.instance = Some(new_instance);
                self.save_game(&game)?;
                Ok(())
            }
            None => Err(Error::InvalidNumberOfPlayers),
        }
    }
}
