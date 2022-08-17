use crate::{
    controller::{json_response, Request, Response},
    error::Error,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
struct HealthzResponse {
    pub hello: String,
    pub build_at: String,
    pub commit_version: String,
}

pub async fn controller(_req: Request) -> Result<Response, Error> {
    json_response(
        StatusCode::OK,
        &HealthzResponse {
            hello: "kv server".to_string(),
            build_at: option_env!("KV_SERVER_BUILD_AT")
                .unwrap_or("UNKNOWN")
                .to_string(),
            commit_version: option_env!("KV_SERVER_CURRENT_COMMIT_ID")
                .unwrap_or("UNKNOWN")
                .to_string(),
        },
    )
}
