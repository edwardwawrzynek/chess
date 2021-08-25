use crate::db::{DBWrapper, GameTimeCfg};
use crate::error::Error;
use crate::games::{GameState, GameTurn};
use crate::models::{TournamentId, TournamentPlayer, UserId};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

pub struct TournamentCfg {
    pub game_type: String,
    pub time_cfg: GameTimeCfg,
}

/// A type of tournament game assignment method
pub trait TournamentType: Send + Sync {
    /// Create an instance of the method
    fn new(
        &self,
        data: &str,
        cfg: &TournamentCfg,
    ) -> Result<Box<dyn TournamentTypeInstance>, Error>;
}

/// A player's score record in a tournament
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct PlayerScoreRecord {
    wins: i32,
    losses: i32,
    ties: i32,
}

pub trait TournamentTypeInstance {
    /// Serialize to a format suitable for deserialization with TournamentType::new
    fn serialize(&self, cfg: &TournamentCfg, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Advance the tournament -- create or start games + otherwise move the tournament forwards.
    /// Called when the tournament is first created, and when a game finishes
    fn advance(
        &mut self,
        id: TournamentId,
        owner: UserId,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<(), Error>;

    /// Return the state of the tournament -- if there is a winner or not
    fn end_state(
        &self,
        id: TournamentId,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<GameState, Error>;
}

pub type TournamentTypeMap = HashMap<&'static str, Box<dyn TournamentType>>;

// A round robin tournament, where each permutation of players in run once
pub struct RoundRobin();
pub struct RoundRobinInstance {
    // number of players in each game
    num_players_per_game: usize,
}

impl TournamentType for RoundRobin {
    fn new(
        &self,
        data: &str,
        _cfg: &TournamentCfg,
    ) -> Result<Box<dyn TournamentTypeInstance>, Error> {
        Ok(Box::new(RoundRobinInstance {
            num_players_per_game: data.parse::<usize>()?,
        }))
    }
}

impl RoundRobinInstance {
    fn create_games(
        &mut self,
        owner: UserId,
        id: TournamentId,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<(), Error> {
        // create all permutations of players
        for players in players
            .iter()
            .permutations(self.num_players_per_game)
            .unique()
        {
            // make game
            let game = db.new_game(&*cfg.game_type, owner, cfg.time_cfg, Some(id))?;
            // attach players to game
            for player in players {
                db.join_game(game.id, player.user_id)?;
            }
        }

        Ok(())
    }
}

impl TournamentTypeInstance for RoundRobinInstance {
    fn serialize(&self, _cfg: &TournamentCfg, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.num_players_per_game)
    }

    fn advance(
        &mut self,
        id: TournamentId,
        owner: UserId,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<(), Error> {
        // if no players, do nothing (tournament ended)
        if players.len() == 0 {
            return Ok(());
        }
        // otherwise, load existing games
        let games = db.find_tournament_games(id)?;
        // if no games exist, tournament was just started, so make games
        if games.len() == 0 {
            // create games
            self.create_games(id, owner, cfg, players, db)?;
            // advance to start games
            return self.advance(id, owner, cfg, players, db);
        }
        // otherwise, load full game information
        let mut games_and_players = vec![];
        for game in games {
            games_and_players.push(db.dbgame_to_game_and_players(game)?);
        }
        // count number of active games each player is in
        let mut games_per_player = HashMap::new();
        for player in players.iter() {
            games_per_player.insert(player.user_id, 0);
        }
        for (game, players) in &games_and_players {
            if let Some(instance) = game.instance.as_ref() {
                if let GameTurn::Turn(_) = instance.turn() {
                    // game is active, so mark players as being in active game
                    for player in players {
                        if let Some(active) = games_per_player.get_mut(&player.user_id) {
                            *active += 1
                        }
                    }
                }
            }
        }
        // threshold of most active games a player can be in at once (TODO: make configurable)
        let max_active_games = 1;
        // start games that don't include any active players
        for (game, players) in &games_and_players {
            let mut violates_thresh = false;
            for player in players {
                if games_per_player[&player.user_id] >= max_active_games {
                    violates_thresh = true;
                    break;
                }
            }
            if violates_thresh {
                continue;
            }

            // no players are involved in too many games, so we can start this game
            db.start_game(game.id, owner)?;
        }

        Ok(())
    }

    fn end_state(
        &self,
        id: TournamentId,
        _cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<GameState, Error> {
        // no players forces a tie
        if players.len() == 0 {
            Ok(GameState::Tie)
        } else {
            // check for non finished games
            let games = db.find_tournament_games(id)?;
            for game in &games {
                if !game.finished {
                    return Ok(GameState::InProgress);
                }
            }

            // find winner (greatest wins - loses)
            let mut max_score = -(games.len() as i32) - 1;
            // players who got this score
            let mut max_winner = vec![];

            let players = db.find_tournament_players(id)?;
            for player in &players {
                let score = player.win - player.loss;
                if score > max_score {
                    max_score = score;
                    max_winner = vec![player.user_id];
                } else if score == max_score {
                    max_winner.push(player.user_id);
                }
            }

            if max_winner.len() == 1 {
                Ok(GameState::Win(max_winner[0]))
            } else {
                // TODO: express winners of tie
                Ok(GameState::Tie)
            }
        }
    }
}
