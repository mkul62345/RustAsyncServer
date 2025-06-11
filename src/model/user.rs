use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::base::{self, DbBackendModelController};
use crate::ModelManager;
use crate::model::{Error, Result};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use uuid::Uuid;
use modql::field::{Fields, HasSeaFields};

pub struct UserBackendModelController;

impl DbBackendModelController for UserBackendModelController {
    const TABLE: &'static str = "user";
}

impl UserBackendModelController {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where 
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn first_user_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager, 
        username: &str
    ) -> Result<Option<E>>
    where 
        E: UserBy,
    {
        // Build query
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(E::sea_idens())
            .and_where(Expr::col(UserIdentity::Username).eq(username));
        
        // Execute query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let user = sqlx::query_as_with::<_, E, _>(&sql, values)
            .fetch_optional(mm.db())
            .await?;

        Ok(user)
    }

    pub async fn update_pwd(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        pwd_clear: &str,
    ) -> Result<()>{
        // Prepare password
        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let pwd = pwd::encrypt_pwd(&EncryptContent{
            content: pwd_clear.to_string(),
            salt: user.pwd_salt.to_string()
        })?;

        // Build query
        let mut query = Query::update();
        query
            .table(Self::table_ref())
            .value(UserIdentity::Pwd, SimpleExpr::from(pwd))
            .and_where(Expr::col(UserIdentity::Id).eq(id));

        // Execute query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let _count = sqlx::query_with(&sql, values)
            .execute(mm.db())
            .await?
            .rows_affected();
        
        Ok(())
    }

}

// region: User Types
#[derive(Clone, FromRow, Debug, Serialize, Fields)]
pub struct  User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}

#[derive(Fields)]
struct UserForInsert {
    username: String,
}

#[derive(Clone, FromRow, Debug, Fields)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    // Password and token
    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Debug, Fields)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    // Token
    pub token_salt: Uuid,
}

// Marker trait
pub trait UserBy: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

#[derive(Iden)]
enum UserIdentity {
    Id,
    Username,
    Pwd,
}
// endregion: User Types