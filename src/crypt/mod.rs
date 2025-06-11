pub use self::error::{Error, Result};
use hmac::{Hmac, Mac};
use sha2::Sha512;

pub mod pwd;
pub mod token;
mod error;

pub struct EncryptContent {
    pub content: String,    // Plaintext content
    pub salt: String,       // Plaintext salt
}

pub fn encrypt_into_b64u(
    key: &[u8],
    enc_content: &EncryptContent,
) -> Result<String> {
    let EncryptContent { content, salt} = enc_content;

    let mut hmac_sha512 =
        Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());
    let hmac_result = hmac_sha512.finalize();
    let result_bytes = hmac_result.into_bytes();
    let result = base64_url::encode(&result_bytes);

    Ok(result)
}


///    Tests section
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};
    use rand::RngCore;

    #[test]
    fn test_encrypt_into_b64u_ok() -> Result<()> {
        let mut fx_key = [0u8; 64];
        rand::rng().fill_bytes(&mut fx_key);
        let fx_encoded_content = EncryptContent {
            content:"some content".to_string(),
            salt: "some salt".to_string(),
        };

        //TODO: Fix the key and compare with precomputed data
        let fx_result = encrypt_into_b64u(&fx_key, &fx_encoded_content)?;
        let res = encrypt_into_b64u(&fx_key, &fx_encoded_content)?;

        //Testing that the function is returning the same result given the same arguments
        assert_eq!(fx_result, res);

        Ok(())
    }
    //Add tests here

}