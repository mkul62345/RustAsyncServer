use std::sync::Arc;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

use crate::{crypt, model};

use super::mw_auth;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
pub enum Error {
	// RPC
	RpcMethodUnknown(String),
	RpcMissingParams{ rpc_mathod: String},
	RpcFailJsonParams{ rpc_mathod: String},

    // Login
    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd{user_id: i64},
    LoginFailPwdNotMatching{user_id: i64},
    
    // Modules
    Model(model::Error),
	Crypt(crypt::Error),

	// Ext Modules,
	SerdeJson(String),

	// Context Extraction
	CtxExt(mw_auth::CtxExtError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		response.extensions_mut().insert(Arc::new(self));   // SILENCED ERROR, UNCOMMENT | Derive clone if something pops after uncommenting

		response
    }
}

// region: From Impls
impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::SerdeJson(val.to_string())
    }
}

impl From<model::Error> for Error {
    fn from(val: model::Error) -> Self {
        Self::Model(val)
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<mw_auth::CtxExtError> for Error {
    fn from(val: mw_auth::CtxExtError) -> Self {
        Self::CtxExt(val)
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

			// Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// Model
			Model(model::Error::EntityNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id }
			),

			// Fallback
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR, 
				ClientError::SERVICE_ERROR
			),
		}
	}
}

#[derive(Serialize, Debug, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	ENTITY_NOT_FOUND { entity: &'static str, id: i64},

	SERVICE_ERROR,
}