use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::response::Response;
use axum::middleware::Next;
use axum::http::Request;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;
use crate::crypt::token::{validate_web_token, Token};
use crate::model::user::{UserBackendModelController, UserForAuth};
use crate::web::AUTH_TOKEN;
use crate::web::{Error, Result};
use crate::model::ModelManager;
use crate::ctx::Ctx;
use axum::http::request::Parts;

use super::set_token_cookie;


pub async fn mw_ctx_require(
    ctx: Result<CtxW>,
    req: Request<Body>, 
    next: Next,
) -> Result<Response>{
    debug!("mw_ctx_require - {ctx:?}");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	debug!("mw_ctx_resolver"); 

	let ctx_extraction_result = _ctx_resolve(mm, &cookies).await;

	if ctx_extraction_result.is_err()
		&& !matches!(ctx_extraction_result, Err(CtxExtError::TokenNotInCookie))
	{
		cookies.remove(Cookie::from(AUTH_TOKEN)); 
	}

	// Store ctx_extraction_result in the request extension
	req.extensions_mut().insert(ctx_extraction_result);

	next.run(req).await
}

async fn _ctx_resolve(mm: ModelManager, cookies: &Cookies) -> CtxExtResult {
	// Get Token String
	let token = cookies
		.get(AUTH_TOKEN)
		.map(|c| c.value().to_string())
		.ok_or(CtxExtError::TokenNotInCookie)?;

	// Parse Token
	let token = token.parse::<Token>().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// Get UserForAuth
	let user: UserForAuth =
		UserBackendModelController::first_user_by_username(&Ctx::root_ctx(), &mm, &token.identifier)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// Validate Token
	validate_web_token(&token, &user.token_salt.to_string())
		.map_err(|_| CtxExtError::FailValidate)?;

	// Update Token
	set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
		.map_err(|_| CtxExtError::CannotSetTokenCookie)?;

	// Create CtxExtResult
	Ctx::new(user.id)
		.map(CtxW)
		.map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

impl<S: Send + Sync> FromRequestParts<S> for CtxW {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	TokenNotInCookie,
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,
	CannotSetTokenCookie,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error

