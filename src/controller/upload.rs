use crate::error::Error;
use lambda_http::{Request, Response, Body, http::StatusCode};
use serde::{Deserialize, Serialize};

use super::json_response;

#[derive(Debug, Clone, Deserialize)]
struct UploadRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
struct UploadResponse {
    pub response: String,
}

pub async fn controller(req: Request) -> Result<Response<Body>, Error> {
    let body: UploadRequest = super::json_parse_body(&req)?;
    let response = UploadResponse {
        response: format!("Hello, {}!", body.name),
    };

    json_response(StatusCode::OK, &response)
}
