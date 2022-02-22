use crate::{error::Error, controller::lambda::json_response};
use lambda_http::{http::StatusCode, Body, Request, RequestExt, Response};
use serde::Serialize;

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
