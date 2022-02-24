use crate::{
    controller::{json_response, Request, Response},
    error::Error,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
struct HealthzResponse {
    pub hello: String,
}

pub async fn controller(_req: Request) -> Result<Response, Error> {
    json_response(
        StatusCode::OK,
        &HealthzResponse {
            hello: "kv server".to_string(),
        },
    )
}
