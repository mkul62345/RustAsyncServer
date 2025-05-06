use axum::routing::Route;
use crate::{Error, Result};
use serde::Deserialize;
use axum::{
    extract::{Path, Query, State}, 
    response::{Html, IntoResponse, Response}, 
    routing::get, 
    Router
};

// region: Hello funcs and handler
#[derive(Debug, Deserialize)]
pub struct HelloParams {
    pub name: Option<String>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/hello2/{:name}", get(hello_handler_personal))
}

async fn hello_handler(Query(params): Query<HelloParams>) -> impl IntoResponse {

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("<h1>Hello, {name}!</h1>"))
}

async fn hello_handler_personal(Path(name): Path<String>) -> impl IntoResponse {

    Html(format!("Hello to <strong>{name}</strong>"))
}
// endregion: Hello funcs and handler


