#![allow(unused)] // Temporary

pub use self::error::{Error, Result};

use std::net::SocketAddr;
use axum::{
    extract::{Path, Query}, 
    middleware, 
    response::{Html, IntoResponse, Response}, 
    routing::{get, get_service, Route}, 
    Router
};
use model::ModelController;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{span, Level};
use serde_json::json;
use serde::Deserialize;


mod model;
mod web;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mc = ModelController::new().await?;

    let app = Router::new()
        .merge(web::routes_login::routes())
        .nest("/api", web::routes_hello::routes())
        .nest("/api", web::routes_tickets::routes(mc.clone()))
        .nest_service("/pic", ServeDir::new("assets/tba.png"))
        .nest_service("/text", ServeDir::new("assets/dror.txt"))
        .nest_service("/vid", ServeDir::new("assets/helooks.mp4"))
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new())
        .fallback_service(ServeDir::new("./assets/missing.jpg"));
        
    ////// Server start
    tokio::join!(serve(app, 3000));
    ///// Server end

    Ok(())
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}


async fn main_response_mapper(res: Response) -> Response{
    println!();
    res
}





///    Tests section
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hello_test() {
        let test_query = web::routes_hello::HelloParams { name: Some(String::from("TEST"))};
        let resp = true;
        //assert_eq!("<h1>Hello, World!</h1>", hello_handler().await.0); 
        assert!(resp)
    }
    
/* 
    async fn login_test() -> Result<()>{
        let req_login = json!({
            "username": "user",
            "pwd": "123"
        });
        
        
        Ok(())
    }
*/

    //Add tests here

}
