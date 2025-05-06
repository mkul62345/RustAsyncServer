use crate::{Error, Result};
use axum::{Json, Router};
use axum::routing::post;
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::{Cookie, Cookies};
use crate::web;



pub fn routes() -> Router {
    Router::new().route("/login", post(api_login))
}


async fn api_login(cookies: Cookies ,payload: Json<LoginPayload>) -> Result<Json<Value>> {
    
    // TODO: Swap out the if for proper db auth 
    if payload.username != "user" || payload.pwd != "123"{ 
        return Err(Error::LoginFail);
    }
    
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