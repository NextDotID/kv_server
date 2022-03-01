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
    ParseError {
        #[from]
        source: serde_json::error::Error,
    },
    #[error("HTTP general error")]
    HttpError {
        #[from]
        source: lambda_http::http::Error,
    },
    #[error("Config error: {source:?}")]
    ConfigError {
        #[from]
        source: config::ConfigError,
    },
    #[error("Database error: {source:?}")]
    DatabaseError {
        #[from]
        source: diesel::result::Error,
    },
    #[error("Crypto error: {source:?}")]
    CryptoError{
        #[from]
        source: libsecp256k1::Error,
    },
    #[error("Parse hex error: {source:?}")]
    HexError {
        #[from]
        source: hex::FromHexError,
    },
    #[error("Error when calling remote server: {source:?}")]
    HttpClientError {
        #[from]
        source: hyper::Error,
    }
    // #[error("Crypto error: {source: ?}")]
    // CryptoCoreError {
    //     #[from]
    //     source: libsecp256k1::Error,
    // }
}

impl Error {
    pub fn http_status(&self) -> StatusCode {
        match self {
            Error::General(_, status) => *status,
            Error::ParamMissing(_) => StatusCode::BAD_REQUEST,
            Error::ParamError(_) => StatusCode::BAD_REQUEST,
            Error::BodyMissing => StatusCode::BAD_REQUEST,
            Error::ParseError { source: _ } => StatusCode::BAD_REQUEST,
            Error::HttpError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ConfigError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DatabaseError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CryptoError { source: _ } => StatusCode::BAD_REQUEST,
            Error::HexError { source: _ } => StatusCode::BAD_REQUEST,
            Error::HttpClientError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
