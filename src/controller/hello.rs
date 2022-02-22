use crate::error::Error;
use lambda_http::{http::StatusCode, Body, Request, RequestExt, Response};
use serde::Serialize;

use super::json_response;

#[derive(Serialize)]
struct HelloResponse {
    pub hello: String,
}

pub async fn controller(req: Request) -> Result<Response<Body>, Error> {
    let params = req.query_string_parameters();
    let target = params
        .first("name")
        .ok_or(Error::ParamMissing("name".to_string()))?;

    json_response(
        StatusCode::OK,
        &HelloResponse {
            hello: target.to_string(),
        },
    )
}
