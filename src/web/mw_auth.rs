use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::response::Response;
use axum::middleware::Next;
use axum::http::Request;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use crate::model::ModelController;
use crate::ctx::Ctx;
use axum::http::request::Parts;


pub async fn mw_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>, 
    next: Next,
) -> Result<Response>{
    println!("MIDDLEWARE AUTH REQUEST");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
	_mc: State<ModelController>,
	cookies: Cookies,
	mut req: Request<Body>,
	next: Next,
) -> Result<Response> {
	println!("->> mw_ctx_resolver");

	let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

	// Compute Result<Ctx>.
	let result_ctx = match auth_token
		.ok_or(Error::AuthFailNoAuthTokenCookie)
		.and_then(parse_token)
	{
		Ok((user_id, _exp, _sign)) => {
			// TODO: Token components validations.
			Ok(Ctx::new(user_id))
		}
		Err(e) => Err(e),
	};

	// Remove the cookie if something went wrong other than NoAuthTokenCookie.
	if result_ctx.is_err()
		&& !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie))
	{
		cookies.remove(Cookie::from(AUTH_TOKEN))
	}

	// Store the ctx_result in the request extension.
	req.extensions_mut().insert(result_ctx);

	Ok(next.run(req).await)
}



impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		println!("->> {:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<Result<Ctx>>()
			.ok_or(Error::AuthFailCtxNotInRequestExt)?
			.clone()
	}

}

//Currently parsed using a regex pattern
fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(
        r#"user-(\d+)\.(.+)\.(.+)"#, // Pattern
        &token
    ).ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id: u64 = user_id
    .parse()
    .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}