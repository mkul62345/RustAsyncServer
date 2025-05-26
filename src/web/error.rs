use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

use crate::model;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    // Login
    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd{user_id: i64},
    LoginFailPwdNotMatching{user_id: i64},
    
    // Modules
    Model(model::Error)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		//response.extensions_mut().insert(self);   // SILENCED ERROR, UNCOMMENT | Derive clone if something pops after uncommenting

		response
    }
}

// region: From Impls
impl From<model::Error> for Error {
    fn from(val: model::Error) -> Self {
        Self::Model(val)
    }
}
// endregion: From Impls

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
        use Error::*;

		#[allow(unreachable_patterns)] //For cases where fallback is redundant]
		match self {
            // Login
            LoginFailUsernameNotFound
            | LoginFailUserHasNoPwd { .. }
            | LoginFailPwdNotMatching { .. } => {
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
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
	SERVICE_ERROR,
}