use crate::db::{DBWrapper, GameTimeCfg};
use crate::error::Error;
use crate::games::GameState;
use crate::models::{TournamentPlayer, UserId};
use std::collections::HashMap;
use std::fmt;

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
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> Result<(), Error>;

    /// Return the state of the tournament -- if there is a winner or not
    fn end_state(
        &self,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> GameState;

    /// Return each player's record
    /// You should only override this is your tournament's score records aren't player's actual records
    fn player_records(
        &self,
        cfg: &TournamentCfg,
        players: &[TournamentPlayer],
        db: &DBWrapper,
    ) -> HashMap<UserId, PlayerScoreRecord> {
        let res = HashMap::new();

        res
    }
}

pub type TournamentTypeMap = HashMap<&'static str, Box<dyn TournamentType>>;
