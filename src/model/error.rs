use modql::filter::IntoSeaError;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use crate::{crypt, model::store};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    EntityNotFound { entity: &'static str, id: i64},
    ListLimitOverMax { max: i64, actual: i64},

    // Modules
    Crypt(crypt::Error),
    Store(store::Error),

    // Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),
    ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error)
    
}

// region: From Impls
impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::SerdeJson(val)
    }
}

impl From<modql::filter::IntoSeaError> for Error {
    fn from(val: modql::filter::IntoSeaError) -> Self {
        Self::ModqlIntoSea(val)
    }
}

impl From<sea_query::error::Error> for Error {
    fn from(val: sea_query::error::Error) -> Self {
        Self::SeaQuery(val)
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(val)
    }
}
// endregion: From Impls

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate