use lambda_http::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // general
    #[error("{0}")]
    General(String, StatusCode),
    // http
    #[error("Param missing: {0}")]
    ParamMissing(String),
    #[error("Param error: {0}")]
    ParamError(String),
    #[error("no body provided")]
    BodyMissing,
    #[error("JSON parse error")]
    ParseError(#[from] serde_json::error::Error),
    #[error("HTTP general error")]
    HttpError(#[from] lambda_http::http::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Crypto error: {0}")]
    CryptoError(#[from] libsecp256k1::Error),
    #[error("Signature validation error: {0}")]
    SignatureValidationError(String),
    #[error("Parse hex error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Error when calling remote server: {0}")]
    HttpClientError(#[from] hyper::Error),
    #[error("base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("UUID parse error: {0}")]
    UuidParseError(#[from] uuid::parser::ParseError),
}

impl Error {
    pub fn http_status(&self) -> StatusCode {
        match self {
            Error::General(_, status) => *status,
            Error::ParamMissing(_) => StatusCode::BAD_REQUEST,
            Error::ParamError(_) => StatusCode::BAD_REQUEST,
            Error::BodyMissing => StatusCode::BAD_REQUEST,
            Error::ParseError(_) => StatusCode::BAD_REQUEST,
            Error::HttpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CryptoError(_) => StatusCode::BAD_REQUEST,
            Error::HexError(_) => StatusCode::BAD_REQUEST,
            Error::HttpClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SignatureValidationError(_) => StatusCode::BAD_REQUEST,
            Error::Base64Error(_) => StatusCode::BAD_REQUEST,
            Error::UuidParseError(_) => StatusCode::BAD_REQUEST,
        }
    }
}
