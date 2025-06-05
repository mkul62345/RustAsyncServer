use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBackendModelController, UserForLogin};
use crate::model::ModelManager;
use crate::web::error::{Error, Result};
use crate::web::{self, remove_token_cookie};
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::post;
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::Cookies;
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
    .route("/api/login", post(api_login_handler))
    .route("/api/logout", post(api_logout_handler))
    .with_state(mm)
}

// region: Logout
#[derive(Debug, Deserialize)]
struct LogoutPayload {
    logout: bool,
}

async fn api_logout_handler(
    cookies: Cookies,
    Json(payload): Json<LogoutPayload>
) -> Result<Json<Value>>{
    debug!("api_logout_handler");

    let should_logout = payload.logout;

    if should_logout {
        remove_token_cookie(&cookies)?;
    }

    //Create success response body
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}
// endregion: Logout

// region: Login
#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
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
    
    let user: UserForLogin = UserBackendModelController::first_user_by_username(&root_ctx, &mm, &username)
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
     
    // Set web-token
    web::set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    //Create success response body
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}
// endregion: Login

