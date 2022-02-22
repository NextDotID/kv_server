use lambda_http::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // http
    #[error("Param missing: {0}")]
    ParamMissing(String),
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
}

impl Error {
    pub fn http_status(&self) -> StatusCode {
        match self {
            Error::ParamMissing(_) => StatusCode::BAD_REQUEST,
            Error::BodyMissing => StatusCode::BAD_REQUEST,
            Error::ParseError { source: _ } => StatusCode::BAD_REQUEST,
            Error::HttpError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ConfigError { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
