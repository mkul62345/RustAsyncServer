#![allow(unused)] // Temporary
pub use self::error::{Error, Result};

use crate::ctx::Ctx;
use axum::http::{Method, Uri};
use log::log_request;
use std::net::SocketAddr;
use axum::{
    extract::{Path, Query}, middleware, response::{Html, IntoResponse, Response}, routing::{get, get_service, Route}, Json, Router
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
use uuid::Uuid;

mod log;
mod ctx;
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

    let routes_apis = web::routes_tickets::routes(mc.clone())
 		.route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));
    
    let app = Router::new()
        .merge(web::routes_hello::routes())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .nest_service("/pic", ServeDir::new("assets/tba.png"))
        .nest_service("/text", ServeDir::new("assets/dror.txt"))
        .nest_service("/vid", ServeDir::new("assets/helooks.mp4"))
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
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
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

async fn main_response_mapper(
    ctx: Result<Ctx>,
	uri: Uri,
	req_method: Method,
	res: Response,
) -> Response {
	println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
	let uuid = Uuid::new_v4();

	// -- Get the eventual response error.
	let service_error = res.extensions().get::<Error>();
	let client_status_error = service_error.map(|se| se.client_status_and_error());

	// -- If client error, build the new reponse.
	let error_response =
		client_status_error
			.as_ref()
			.map(|(status_code, client_error)| {
				let client_error_body = json!({
					"error": {
						"type": client_error.as_ref(),
						"req_uuid": uuid.to_string(),
					}
				});

				println!("    ->> client_error_body: {client_error_body}");

				// Build the new response from the client_error_body
				(*status_code, Json(client_error_body)).into_response()
			});

	// Build and log the server log line.
	let client_error = client_status_error.unzip().1;
	// TODO: Need to handle if log_request fail (but should not fail request)                                   
	let _ =
		log_request(uuid, req_method, uri, ctx.ok(), service_error, client_error).await;

	println!();
	error_response.unwrap_or(res)
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
