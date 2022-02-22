use std::{fmt, error};
use lambda_http::http::StatusCode;

#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Internal,
    BadRequest,
    NotFound,
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub category: ErrorCategory,
    pub module: String,
    pub description: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.module, self.description)
    }
}
impl error::Error for Error {}

impl Error {
    pub fn http_status(&self) -> StatusCode {
        match self.category {
            ErrorCategory::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCategory::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCategory::NotFound => StatusCode::NOT_FOUND,
            ErrorCategory::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

// pub type Result<T> = core::result::Result<T, Error>;
