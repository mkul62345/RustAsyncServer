#![allow(unused)] // Temporary

use std::net::SocketAddr;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{span, Level};


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    let app = Router::new()
        .route("/", get(hello_handler))
        .nest_service("/pic", ServeDir::new("assets/lilnas.jpg"))
        .nest_service("/text", ServeDir::new("assets/dror.txt"))
        .nest_service("/vid", ServeDir::new("assets/helooks.mp4"));

    ////// Server start
    tokio::join!(serve(app, 3000));
    ///// Server end


}


async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}



async fn hello_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}










#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let resp = hello_handler().await.0;
        println!("{}",resp);
        assert_eq!("<h1>Hello, World!</h1>", hello_handler().await.0);
    }

}
