use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    // Key Errors
    KeyFailHmac,

	// Password Errors
	PwdNotMatching,

	// Token Errors
	TokenInvalidFormat,
	TokenCannotDecodeIdentifier,
	TokenCannotDecodeExpiration,
	TokenSignatureNotMatching,
	TokenExpirationNotIso,
	TokenExpired,
	
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