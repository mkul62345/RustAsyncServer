mod error;

pub use self::error::{Error, Result};

use crate::config; 
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
    .max_connections(1) // Pool size set to 1 for testing, otherwise it locks up.
    .connect(&config().DB_URL).await
    .map_err(|ex| Error::FailToCreatePool(ex.to_string())) 
}


pub type Db = Pool<Postgres>;