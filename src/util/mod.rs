use crate::error::Error;
use chrono::NaiveDateTime;

pub fn base64_to_vec(b64: &str) -> Result<Vec<u8>, Error> {
    base64::decode(b64).map_err(|e| e.into())
}

pub fn vec_to_base64(bytes_vec: &Vec<u8>) -> String {
    base64::encode(bytes_vec)
}

/// With `0x` prefix
pub fn vec_to_hex(bytes_vec: &Vec<u8>) -> String {
    format!("0x{}",  hex::encode(bytes_vec))
}

/// Returns current UNIX timestamp (unit: second).
pub fn timestamp() -> i64 {
    naive_now().timestamp()
}

/// Work as `NaiveDateTime::now()`
pub fn naive_now() -> NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

/// Convert timestamp into NaiveDateTime struct.
pub fn timestamp_to_naive(ts: i64) -> NaiveDateTime {
    NaiveDateTime::from_timestamp_opt(ts, 0).unwrap()
}
