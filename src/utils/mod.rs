use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

pub use self::error::{Error, Result};

mod error;

pub fn b64u_encode(content: &str) -> String {
    base64_url::encode(content)
}

pub fn b64u_decode(content: &str) -> Result<String> {
    let decoded_string = base64_url::decode(content)
        .ok()
        .and_then(|r| String::from_utf8(r).ok())
        .ok_or(Error::FailedB64DecodingString)?;

    Ok(decoded_string)
}

// region: Time
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> String {
    time.format(&Rfc3339).unwrap() // TODO: check if unwrapping is safe. | Fetching clock shouldnt be failable.
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
    let new_time = now_utc() + Duration::seconds_f64(sec);
    format_time(new_time)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(moment, &Rfc3339)
        .map_err(|_| Error::DateFailedParsing(moment.to_string()))
}
// endregion: Time