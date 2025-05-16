use core::fmt;

use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Error{
    LoginFail,

	// Auth Errors
	AuthFailNoAuthTokenCookie,
	AuthFailTokenWrongFormat,
	AuthFailCtxNotInRequestExt,

	// Model Errors  | TODO: Refactor into model layer
	TicketDeleteFailIdNotFound { id: u64},
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		response.extensions_mut().insert(self);

		response
    }
}



// region:      Error boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion:    Error boilerplate