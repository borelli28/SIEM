use rusqlite::{Connection, Result, Error};
use super::schema::Schema;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> Result<Connection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .map_err(|e| Error::InvalidParameterName(format!("DATABASE_URL must be set: {}", e)))?;

    let conn = Connection::open(&database_url)
        .map_err(|e| Error::InvalidParameterName(format!("Error connecting to {}: {}", database_url, e)))?;

    conn.execute("PRAGMA foreign_keys = ON", [])?; // Enable foreign keys
    Schema::create_all(&conn)?;

    Ok(conn)
}