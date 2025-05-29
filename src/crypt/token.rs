use std::fmt::Display;
use std::str::FromStr;

use crate::config;
use crate::crypt::{encrypt_into_b64u, EncryptContent, Error, Result};
use crate::utils::{b64u_decode, b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc};

// region: Token type
#[derive(Debug)]
pub struct Token {
    pub identifier: String,     // Identifier e.g username
    pub expiration: String,     // Expiration date in Rfc3339
    pub signature_b64u: String  // Signature, base64url encoded.
}

impl FromStr for Token{
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let splits: Vec<&str> = token_str.split('.').collect();
        if splits.len() != 3 {
            return Err(Error::TokenInvalidFormat)
        }

        let (identity_b64u, expiration_b64u, signature_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Self { 
            identifier: b64u_decode(identity_b64u)
                .map_err(|_| Error::TokenCannotDecodeIdentifier)?,

            expiration: b64u_decode(expiration_b64u)
                .map_err(|_| Error::TokenCannotDecodeExpiration)?,

            signature_b64u: signature_b64u.to_string(),
        })
    }
}

impl Display for Token {
	fn fmt(
		&self,
		fmt: &mut std::fmt::Formatter,
	) -> core::result::Result<(), std::fmt::Error> {
		write!(
            fmt,
            "{}.{}.{}",
            b64u_encode(&self.identifier),
            b64u_encode(&self.expiration),
            self.signature_b64u
            )
	}
}
// endregion: Token type

// region: Web-token Generation/Validation
pub fn generate_web_token(user: &str, salt: &str) -> Result<Token>{
    let config = &config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(original_token: &Token, salt: &str) -> Result<()>{
    let config = &config();
    _validate_token_sign_and_exp(original_token, salt, &config.TOKEN_KEY)?;

    Ok(())
}
// endregion: Web-token Generation/Validation

// region: (private) Token Generation/Validation
fn _generate_token(
    identifier: &str,
    duration_sec: f64,
    salt: &str,
    key: &[u8],
) -> Result<Token> {
    let ident = identifier.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);

    // Sign the first two token components
    let sign_b64u = _token_sign_into_b64u(&identifier, &exp, salt, key)?;

    Ok(Token { 
        identifier: ident, 
        expiration: exp, 
        signature_b64u: sign_b64u, 
    })
}

fn _validate_token_sign_and_exp(
    original_token: &Token,
    salt: &str,
    key: &[u8],
) -> Result<()> {
    let new_signature_b64u=
        _token_sign_into_b64u(&original_token.identifier, &original_token.expiration, salt, key)?;

    if new_signature_b64u != original_token.signature_b64u {
        return Err(Error::TokenSignatureNotMatching);
    }

    // Validate expiration
    let original_exp = 
        parse_utc(&original_token.expiration).map_err(|_| Error::TokenExpirationNotIso)?;
    
    let now = now_utc();

    if original_exp < now {
        return Err(Error::TokenExpired);
    }

    Ok(())
}

// Create token signature from token parts and salt
fn _token_sign_into_b64u(
    identifier: &str,
    expiration: &str,
    salt: &str,
    key: &[u8],
) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(identifier), b64u_encode(expiration));
    let signature = encrypt_into_b64u(
        key, 
        &EncryptContent {
        content: content,
        salt: salt.to_string()
    })?;

    Ok(signature)
}
// endregion: (private) Token Generation/Validation

///    Tests section
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;
    use anyhow::{Ok, Result};

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        //Setup
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.02; // 20ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = 
            _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;
        
        //Execute
        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        //Check
        res?; 

        Ok(())
    }

    #[test]
    fn test_validate_web_token_err_expired() -> Result<()> {
        //Setup
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.01; // 10ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = 
            _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;
        
        //Execute
        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        //Check
        assert!(
            matches!(res, Err(Error::TokenExpired)),
            "Expected Error::TokenExpired but got '{res:?}'"
        );

        Ok(())
    }
    //Add tests here

}