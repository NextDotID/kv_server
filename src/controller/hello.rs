use crate::{
    controller::{json_response, query_parse, Request, Response},
    error::Error,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
struct HelloResponse {
    pub hello: String,
}

pub async fn controller(req: Request) -> Result<Response, Error> {
    let params = query_parse(req);
    let target = params
        .get("name")
        .ok_or(Error::ParamMissing("name".to_string()))?;

    json_response(
        StatusCode::OK,
        &HelloResponse {
            hello: target.to_string(),
        },
    )
}
