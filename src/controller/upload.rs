use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    controller::{json_parse_body, json_response, Request, Response},
    error::Error,
};

#[derive(Debug, Clone, Deserialize)]
struct UploadRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
struct UploadResponse {
    pub response: String,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let body: UploadRequest = json_parse_body(&req)?;
    let response = UploadResponse {
        response: format!("Hello, {}!", body.name),
    };

    json_response(StatusCode::OK, &response)
}
