#![feature(async_closure)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use dotenv::dotenv;
use std::env;

mod apikey;
mod cmd;
mod db;
mod error;
mod models;
mod schema;
mod server;
mod games;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let addr = env::var("SERVER_URL").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    let db_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set to the psotgres database url");

    server::run_server(&addr, &db_url).await;
}
