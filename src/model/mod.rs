pub use self::error::{Error, Result};
use store::{new_db_pool, Db};

pub mod task;
mod base;
mod error;
mod store;


#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;

        Ok(ModelManager { db })
    }

    //Lock access to store outside of model layer
    //Returns: sqlx Db pool reference
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}