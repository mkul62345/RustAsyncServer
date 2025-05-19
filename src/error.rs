use core::fmt;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

use crate::model;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error{
    LoginFail,

	// Initialization Errors
	ConfigMissingEnv{ var: &'static str},

	// Auth Errors
	AuthFailNoAuthTokenCookie,
	AuthFailTokenWrongFormat,
	AuthFailCtxNotInRequestExt,

	// Model Errors  | TODO: Refactor into model layer
	TicketDeleteFailIdNotFound { id: u64},


	// Modules
	//Model(model::Error),
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


impl Error{
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError){
		#[allow(unreachable_patterns)] //For cases where fallback is redundant]
		match self {

			//Login Fail
			Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

			//Auth
			Self::AuthFailCtxNotInRequestExt
			| Self::AuthFailNoAuthTokenCookie
			| Self::AuthFailTokenWrongFormat => {
			 (StatusCode::FORBIDDEN, ClientError::NO_AUTH)
			}

			//Model
			Self::TicketDeleteFailIdNotFound { id } => {
				(StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
			}

			//Fallback
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR, 
				ClientError::SERVICE_ERROR
			),
		}
	}
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	INVALID_PARAMS,
	SERVICE_ERROR,
}