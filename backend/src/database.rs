use rusqlite::{Connection, Result, Error};
use super::schema::Schema;
use log::{info, error};
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> Result<Connection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .map_err(|e| {
            error!("DATABASE_URL environment variable not set: {}", e);
            Error::InvalidParameterName(format!("DATABASE_URL must be set: {}", e))
        })?;

    info!("Attempting to connect to database");
    let conn = Connection::open(&database_url)
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            Error::InvalidParameterName(format!("Error connecting to {}: {}", database_url, e))
        })?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    info!("Initializing database schemas");
    Schema::create_all(&conn)?;

    info!("Database connection established successfully");
    Ok(conn)
}