#![allow(unused)] // Temporary
pub use self::error::{Error, Result};
pub use crate::config::config; 
use crate::model::ModelManager;
use crate::web::{
    mw_auth::mw_ctx_require, 
    mw_res_map::mw_response_map, 
    rpc
};
use std::net::SocketAddr;
use axum::{middleware, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    services::{ServeDir},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt
};

mod utils;
mod crypt;
mod config;
mod ctx;
mod model; 
mod web;
mod error;
mod _dev_utils;

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


    ////////////    FOR DEVELOPMENT ONLY
    //_dev_utils::init_dev().await; // For tests: DB teardown -> setup.
    ////////////    FOR DEVELOPMENT ONLY

    let mm: ModelManager = ModelManager::new().await?;

    let routes_rpc = rpc::routes(mm.clone())
        .route_layer(middleware::from_fn(mw_ctx_require));

    let app = Router::new()
        .merge(web::routes_login::routes(mm.clone()))                     //Log in/out API
        .nest("/api", routes_rpc)
        .nest_service("/pic", ServeDir::new("assets/tba.png"))
        .nest_service("/text", ServeDir::new("assets/dror.txt"))
        .nest_service("/vid", ServeDir::new("assets/helooks.mp4"))
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(ServeDir::new("./assets/missing.jpg"));
        
    ////// Server start
    tokio::join!(serve(app, 3000));
    ///// Server end

    Ok(())
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

