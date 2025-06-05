use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use modql::field::HasSeaFields;
use modql::SIden;
use sea_query::{Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

#[derive(Iden)]
pub enum CommonIdentifier {
    Id,
}

pub trait  DbBackendModelController {
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

//TODO: Finish implementing and then implement update.
pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
  MC: DbBackendModelController,
  E: HasSeaFields,
{
    // Arrange data
    let fields = data.not_none_sea_fields();    
    let (columns, sea_values) = fields.for_sea_insert();

    // Build query
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIdentifier::Id]));

    // Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
        .fetch_one(mm.db())
        .await?;

    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
  MC: DbBackendModelController,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
    // Build query
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::sea_column_refs())
        .and_where(Expr::col(CommonIdentifier::Id).eq(id));

    // Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_optional(mm.db())
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE, 
            id, 
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>> 
where 
  MC: DbBackendModelController,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
    // Build query
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::sea_column_refs());

    // Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_all(mm.db())
        .await?;

    Ok(entities)
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
  MC: DbBackendModelController,
  E: HasSeaFields,
{
    // Arrange data
    let fields = data.not_none_sea_fields();   
    let fields = fields.for_sea_update(); 

    // Build query
    let mut query = Query::update();
    query.
        table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIdentifier::Id).eq(id));

    // Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(mm.db())
        .await?
        .rows_affected();
    
    // Verify execution
    if count == 0 {
        Err(Error::EntityNotFound { 
            entity: MC::TABLE, 
            id, 
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> 
where 
  MC: DbBackendModelController,
{
    // Build query
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(CommonIdentifier::Id).eq(id));

    // Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(mm.db())
        .await?
        .rows_affected();

    // Verify execution
    if count == 0 {
        Err(Error::EntityNotFound { 
            entity: MC::TABLE, 
            id, 
        })
    } else {
        Ok(())
    }       
}

