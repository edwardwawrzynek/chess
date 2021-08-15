use super::schema::{game_players, games, users};

pub type UserId = i32;
pub type GameId = i32;
pub type GamePlayerId = i32;

#[derive(Queryable, AsChangeset)]
#[table_name = "users"]
pub struct User {
    pub id: UserId,
    pub email: Option<String>,
    pub name: String,
    pub is_admin: bool,
    pub password_hash: Option<String>,
    pub api_key_hash: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: Option<&'a str>,
    pub name: &'a str,
    pub is_admin: bool,
    pub password_hash: Option<&'a str>,
    pub api_key_hash: &'a str,
}

#[derive(Queryable, AsChangeset)]
#[table_name = "games"]
pub struct DBGame {
    pub id: GameId,
    pub owner_id: UserId,
    pub game_type: String,
    pub state: Option<String>,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub is_tie: Option<bool>,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub current_move_start_ms: Option<i64>,
    pub turn_id: Option<i64>,
}

#[derive(Insertable)]
#[table_name = "games"]
pub struct NewDBGame<'a> {
    pub owner_id: UserId,
    pub game_type: &'a str,
    pub state: Option<&'a str>,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub is_tie: Option<bool>,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub current_move_start_ms: Option<i64>,
    pub turn_id: Option<i64>,
}

#[derive(Queryable, AsChangeset)]
#[table_name = "game_players"]
pub struct GamePlayer {
    pub id: GamePlayerId,
    pub user_id: UserId,
    pub game_id: GameId,
    pub score: Option<f64>,
    pub waiting_for_move: bool,
    pub time_ms: i64,
}

#[derive(Insertable)]
#[table_name = "game_players"]
pub struct NewGamePlayer {
    pub user_id: UserId,
    pub game_id: GameId,
    pub score: Option<f64>,
    pub waiting_for_move: bool,
    pub time_ms: i64,
}
