use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBackendModelController, UserForLogin};
use crate::model::ModelManager;
use crate::web::error::{Error, Result};
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::post;
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::{Cookie, Cookies};
use crate::web;

pub fn routes(mm: ModelManager) -> Router {
    Router::new().route("/login", post(api_login_handler))
    .with_state(mm)
}

async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>
) -> Result<Json<Value>> {
    
    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload;
    let root_ctx = Ctx::root_ctx();
    
    let user: UserForLogin = UserBackendModelController::get_user_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;


    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHasNoPwd{ user_id: user.id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            content: pwd_clear.clone(),
            salt: user.pwd_salt.to_string()
        },
        &pwd
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id: user.id })?;
     
    // TODO: Replace with real token generation
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    //Create success response body
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}


#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}