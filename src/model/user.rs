use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::base::{self, DbBackendModelController};
use crate::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use uuid::Uuid;


// region: User Types
#[derive(Clone, FromRow, Debug, Serialize)]
pub struct  User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}

#[derive(Deserialize)]
struct UserForInsert {
    username: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    // Password and token
    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    // Token
    pub token_salt: Uuid,
}

// Marker trait
pub trait UserBy: for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}
// endregion: User Types

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

    pub async fn get_user_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager, 
        username: &str
    ) -> Result<Option<E>>
    where 
        E: UserBy,
    {
        let db = mm.db();
        let sql = format!(r#"SELECT * FROM "{}" WHERE "username" = $1 LIMIT 1"#, Self::TABLE);  

        let user: Option<E> = sqlx::query_as(&sql)
            .bind(username)
            .fetch_optional(db)
            .await?;
        
        Ok(user)
    }

    pub async fn update_pwd(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        pwd_clear: &str,
    ) -> Result<()>{
        let db = mm.db();
        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let sql = format!(r#"UPDATE "{}" SET "pwd" = $1 WHERE "id" = $2"#, Self::TABLE);  //

        let pwd = pwd::encrypt_pwd(&EncryptContent{
            content: pwd_clear.to_string(),
            salt: user.pwd_salt.to_string()
        })?;

        let res = sqlx::query(&sql)
            .bind(pwd.to_string())
            .bind(id)
            .execute(db)
            .await?;

        Ok(())
    }

}
