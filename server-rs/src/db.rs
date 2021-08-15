use crate::apikey::ApiKey;
use crate::diesel::prelude::*;
use crate::error::Error;
use crate::games::ended_game::EndedGameInstance;
use crate::games::{Fmt, GameInstance, GameState, GameTurn, GameTypeMap};
use crate::models::{
    DBGame, GameId, GamePlayer, GamePlayerId, NewDBGame, NewGamePlayer, NewUser, User, UserId,
};
use crate::schema::{game_players, games, users};
use bcrypt;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use futures_channel::mpsc;
use rand::random;
use std::cmp::max;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// Time control configuration for a game
#[derive(Debug, PartialEq, Eq)]
pub struct GameTimeCfg {
    // Time given for each move
    pub per_move: Duration,
    // Total time given for whole game (starts counting once dur_per_move is exhausted)
    pub sudden_death: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameTimeMs {
    pub per_move_ms: i64,
    pub sudden_death_ms: i64,
}

impl GameTimeCfg {
    /// Convert times to whole milliseconds
    pub fn to_ms(&self) -> GameTimeMs {
        GameTimeMs {
            per_move_ms: self.per_move.as_millis() as i64,
            sudden_death_ms: self.sudden_death.as_millis() as i64,
        }
    }

    pub fn from_ms(per_move: i64, sudden_death: i64) -> Self {
        GameTimeCfg {
            per_move: Duration::from_millis(per_move as u64),
            sudden_death: Duration::from_millis(sudden_death as u64),
        }
    }
}

/// in memory representation of a game
pub struct Game {
    pub id: GameId,
    pub owner_id: UserId,
    pub game_type: String,
    pub instance: Option<Box<dyn GameInstance>>,
    pub time: GameTimeCfg,
    pub current_move_start: Option<SystemTime>,
    pub turn_id: Option<i64>,
}

pub type GameAndPlayers = (Game, Vec<GamePlayer>);

impl Game {
    pub fn from_dbgame(game: DBGame, type_map: &GameTypeMap, players: &[GamePlayerId]) -> Game {
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
            time: GameTimeCfg {
                per_move: Duration::from_millis(game.dur_per_move_ms as u64),
                sudden_death: Duration::from_millis(game.dur_sudden_death_ms as u64),
            },
            current_move_start: game
                .current_move_start_ms
                .map(|ms| UNIX_EPOCH + Duration::from_millis(ms as u64)),
            turn_id: game.turn_id,
        }
    }

    pub fn to_dbgame(&self) -> DBGame {
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
            dur_per_move_ms: self.time.to_ms().per_move_ms,
            dur_sudden_death_ms: self.time.to_ms().sudden_death_ms,
            current_move_start_ms: self.current_move_start.map(|t| {
                t.duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_millis() as i64
            }),
            turn_id: self.turn_id,
        }
    }

    /// calculate the amount of elapsed time since the current move started
    pub fn elapsed_since_current_move(&self) -> Option<Duration> {
        self.current_move_start
            .map(|t| t.elapsed().unwrap_or(Duration::ZERO))
    }

    /// calculate how much time has elapsed in sudden death since the current move started
    pub fn elapsed_sudden_death(&self, elapsed: Duration) -> Duration {
        elapsed
            .checked_sub(self.time.per_move)
            .unwrap_or(Duration::ZERO)
    }

    /// calculate how much time the current player has left in their turn + overall
    pub fn current_player_time(&self, sudden_death_start: Duration) -> GameTimeCfg {
        let elapsed = self.elapsed_since_current_move().unwrap_or(Duration::ZERO);
        let elapsed_sudden_death = self.elapsed_sudden_death(elapsed);

        GameTimeCfg {
            per_move: self
                .time
                .per_move
                .checked_sub(elapsed)
                .unwrap_or(Duration::ZERO),
            sudden_death: sudden_death_start
                .checked_sub(elapsed_sudden_death)
                .unwrap_or(Duration::ZERO),
        }
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db_pool(db_url: &str) -> Result<PgPool, PoolError> {
    let manage = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder().build(manage)
}

/// A message that a player's time in a game may have expired
#[derive()]
pub struct PlayerTimeExpiry {
    // The turn on which this expiry is valid. If the game is no longer on this turn, then the player's turn has not expired.
    pub turn_id: i64,

    pub game_id: GameId,
    pub user_id: UserId,
}

/// A database connection wrapper, which associates the database with functions to manipulate it
pub struct DBWrapper<'a, 'b> {
    db: PooledConnection<ConnectionManager<PgConnection>>,
    game_type_map: &'a GameTypeMap,
    game_update_callback: Box<dyn Fn(&Game, &[GamePlayer], &DBWrapper<'a, 'b>) + 'b>,
    time_expiry_channel: mpsc::UnboundedSender<PlayerTimeExpiry>,
}

impl DBWrapper<'_, '_> {
    /// Wrap an existing pg connection
    pub fn from_pg_pool<'a, 'b>(
        pool: &PgPool,
        type_map: &'a GameTypeMap,
        game_update_callback: impl Fn(&Game, &[GamePlayer], &DBWrapper<'a, 'b>) + 'b,
        time_expiry_channel: mpsc::UnboundedSender<PlayerTimeExpiry>,
    ) -> Result<DBWrapper<'a, 'b>, Error> {
        Ok(DBWrapper {
            db: pool.get()?,
            game_type_map: type_map,
            game_update_callback: Box::new(game_update_callback),
            time_expiry_channel,
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
    pub fn new_game(
        &self,
        game_type: &str,
        owner: UserId,
        time_cfg: GameTimeCfg,
    ) -> Result<DBGame, Error> {
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
            dur_per_move_ms: time_cfg.to_ms().per_move_ms,
            dur_sudden_death_ms: time_cfg.to_ms().sudden_death_ms,
            current_move_start_ms: None,
            turn_id: None,
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
            .order(dsl::id.asc())
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
            waiting_for_move: false,
            time_ms: game.time.to_ms().sudden_death_ms,
        };
        let new_player = diesel::insert_into(game_players::table)
            .values(&player)
            .get_result::<GamePlayer>(&self.db)?;

        players.push(new_player);
        (self.game_update_callback)(&game, &players, self);
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
        (self.game_update_callback)(&game, &players, self);
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

    /// Find all games a user is in that are waiting for that user to play
    pub fn find_waiting_games_for_user(&self, user_id: UserId) -> Result<Vec<GameId>, Error> {
        use game_players::dsl;
        Ok(dsl::game_players
            .filter(dsl::user_id.eq(user_id).and(dsl::waiting_for_move.eq(true)))
            .order(dsl::id.asc())
            .select(dsl::game_id)
            .load::<GameId>(&self.db)?)
    }

    /// Find the oldest game a user is in that is waiting for that user to play
    pub fn find_oldest_waiting_game_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<GameId>, Error> {
        use game_players::dsl;
        Ok(dsl::game_players
            .filter(dsl::user_id.eq(user_id).and(dsl::waiting_for_move.eq(true)))
            .order(dsl::id.asc())
            .select(dsl::game_id)
            .first::<GameId>(&self.db)
            .optional()?)
    }

    /// Update the waiting_for_move field on each game player (doesn't save game players)
    fn update_players_waiting_for_move(
        &self,
        game_inst: &dyn GameInstance,
        players: &mut [GamePlayer],
    ) {
        match game_inst.turn() {
            GameTurn::Finished => {
                // set all players as not waiting
                for player in players.iter_mut() {
                    player.waiting_for_move = false;
                }
            }
            GameTurn::Turn(uid) => {
                for player in players.iter_mut() {
                    player.waiting_for_move = player.user_id == uid;
                }
            }
        }
    }

    /// Update a game and its player's scores in the database
    pub fn save_game_and_players(
        &self,
        game: &Game,
        players: &mut [GamePlayer],
    ) -> Result<(), Error> {
        self.save_dbgame(&game.to_dbgame())?;
        if let Some(instance) = &game.instance {
            // update waiting to move
            self.update_players_waiting_for_move(instance.as_ref(), players);
            // adjust scores
            if let Some(scores) = instance.scores() {
                for player in players.iter_mut() {
                    player.score = Some(scores[&player.user_id]);
                }
            }
            // save players
            for player in players.iter() {
                self.save_game_player(player)?;
            }
        }
        (self.game_update_callback)(game, players, self);
        Ok(())
    }

    /// Update a game and its player's scores in the database (load players from db first)
    pub fn save_game(&self, game: &Game) -> Result<(), Error> {
        let mut players = self.find_game_players(game.id)?;
        self.save_game_and_players(game, &mut *players)
    }

    /// Start timing a move for a game
    fn start_game_timer(&self, game: &mut Game, players: &[GamePlayer]) {
        let game_id = game.id;
        // give this turn a new random id
        let turn_id: i64 = random();
        game.turn_id = Some(turn_id);

        if let Some(ref instance) = game.instance {
            if let GameTurn::Turn(user_id) = instance.turn() {
                // find remaining time for user
                let mut remaining = Duration::ZERO;
                for player in players.iter() {
                    if player.user_id == user_id {
                        remaining = Duration::from_millis(player.time_ms as u64);
                        break;
                    }
                }

                let till_expired = game.time.per_move + remaining;
                let tx = self.time_expiry_channel.clone();
                // start thread to wait for when this player's time will have fully expired
                tokio::spawn((|| async move {
                    tokio::time::sleep(till_expired).await;
                    tx.unbounded_send(PlayerTimeExpiry {
                        turn_id,
                        game_id,
                        user_id,
                    })
                    .unwrap_or_else(|e| eprintln!("Couldn't send game expiry information: {}", e));
                })());
                // mark when turn began
                game.current_move_start = Some(SystemTime::now());
            }
        }
    }

    /// Turn a game into a EndedGameInstance
    pub fn end_game(
        &self,
        game: &mut Game,
        players: &mut [GamePlayer],
        winner: Option<UserId>,
        reason: String,
    ) -> Result<(), Error> {
        let inst = game.instance.as_ref().map(|i| &**i);
        // update time elapsed during turn
        if let Some(inst) = inst {
            if let GameTurn::Turn(user_id) = inst.turn() {
                self.adjust_players_time(&game, &mut *players, user_id);
            }
        }
        // set game state to EndedGameInstance
        game.instance = Some(Box::new(EndedGameInstance::from_current_state(
            inst,
            game.game_type.clone(),
            winner,
            reason,
        )));
        self.save_game_and_players(&game, &mut *players)?;
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
                // start timer for first move
                self.start_game_timer(&mut game, &*players);
                self.save_game(&game)?;
                Ok(())
            }
            None => Err(Error::InvalidNumberOfPlayers),
        }
    }

    /// Subtract elapsed time from the current player in a game. (Doesn't save game players)
    fn adjust_players_time(&self, game: &Game, players: &mut [GamePlayer], current_user: UserId) {
        let elapsed = game.elapsed_since_current_move().unwrap_or(Duration::ZERO);
        let elapsed_sudden_death = game.elapsed_sudden_death(elapsed);

        // make sure time was actually lost
        if elapsed_sudden_death <= Duration::ZERO {
            return;
        }
        for player in players.iter_mut() {
            if player.user_id == current_user {
                player.time_ms -= elapsed_sudden_death.as_millis() as i64;
                player.time_ms = max(player.time_ms, 0);
                break;
            }
        }
    }

    /// Make a move in a game as the given user
    pub fn make_move(&self, game_id: GameId, user_id: UserId, play: &str) -> Result<(), Error> {
        let (mut game, mut players) = self.find_game(game_id)?;
        if let Some(ref mut inst) = game.instance {
            match inst.turn() {
                GameTurn::Turn(uid) if uid == user_id => {
                    // apply move
                    inst.make_move(user_id, play)
                        .map_err(|e| Error::InvalidMove(e))?;
                    // subtract elapsed time from player
                    self.adjust_players_time(&game, &mut *players, user_id);
                    // start timer for next move
                    self.start_game_timer(&mut game, &*players);
                    self.save_game_and_players(&game, &mut *players)?;
                    Ok(())
                }
                _ => Err(Error::NotTurn),
            }
        } else {
            Err(Error::NotTurn)
        }
    }
}
