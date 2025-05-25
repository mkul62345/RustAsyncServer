use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait  DbBackendModelController {
    const TABLE: &'static str;
}

//TODO: Finish implementing and then implement update.
pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
  MC: DbBackendModelController,
{
    let db = mm.db();
    let sql = format!("INSERT {} INTO {} VALUES {}", MC::TABLE, MC::TABLE, MC::TABLE);

    //let (id, ) = sqlx::query_as(&sql);


    Ok(4)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
  MC: DbBackendModelController,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();
    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);  

    let entity: E = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound { entity: MC::TABLE, id})?;
    
    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>> 
where 
  MC: DbBackendModelController,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();
    let sql = format!("SELECT * FROM {}", MC::TABLE);    

    let entities: Vec<E> = sqlx::query_as(&sql)
        .fetch_all(db)
        .await?;
    
    Ok(entities)
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<u64> 
where 
  MC: DbBackendModelController,
{
    let db = mm.db();
    let sql = format!("DELETE FROM {} WHERE id = $1", MC::TABLE); 

    let count = sqlx::query(&sql)
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

        if count == 0 {
            return Err(Error::EntityNotFound { entity:  MC::TABLE, id })
        }
        
    Ok(count)
}