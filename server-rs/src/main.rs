#![feature(async_closure)]

use server_rs::*;

use db::GameTypeMap;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let addr = env::var("SERVER_URL").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    let db_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set to the postgres database url");

    let mut game_type_map: GameTypeMap = HashMap::new();
    game_type_map.insert("chess", Box::new(games::chess_game::ChessGame()));

    server::run_server(&addr, &db_url, Arc::new(game_type_map)).await;
}
