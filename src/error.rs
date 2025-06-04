use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

use crate::model;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error{
	CtxCannotNewRootCtx,

	// Initialization Errors
	ConfigMissingEnv(&'static str),
	ConfigWrongFormat(&'static str),

	// Modules
	Model(model::Error),
}

impl From<model::Error> for Error {
    fn from(val: model::Error) -> Self {
        Self::Model(val)
    }
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