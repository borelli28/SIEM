use rusqlite::{Connection, Result};
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> Result<Connection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Connection::open(&database_url)
        .map_err(|_| panic!("Error connecting to {}", database_url))
}