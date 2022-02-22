mod hello;
mod upload;

use std::future::Future;

use lambda_http::{
    http::{Method, StatusCode},
    Body, Error as LambdaError, IntoResponse, Request, Response,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    pub message: String,
}

async fn entry<F>(req: Request, controller: fn(Request) -> F) -> Response<Body>
where
    F: Future<Output = Result<Response<Body>, Error>>,
{
    match controller(req).await {
        Ok(resp) => resp,
        Err(err) => error_response(err),
    }
}

pub async fn entrypoint(req: Request) -> Result<impl IntoResponse, LambdaError> {
    info!(
        "{} {}",
        req.method().to_string(),
        req.uri().path().to_string()
    );

    Ok(match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => entry(req, hello::controller).await,
        (&Method::POST, "/upload") => entry(req, upload::controller).await,

        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .expect("Failed to render response"),
    })
}

fn json_parse_body<T>(req: &Request) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de>,
{
    match req.body() {
        Body::Empty => Err(Error::BodyMissing),
        Body::Text(text) => serde_json::from_str(text.as_str()).map_err(|e| e.into()),
        Body::Binary(bin) => serde_json::from_slice(bin.as_slice()).map_err(|e| e.into()),
    }
}

fn json_response<T>(status: StatusCode, resp: &T) -> Result<Response<Body>, Error>
where
    T: Serialize,
{
    let body = serde_json::to_string(resp).unwrap();

    Response::builder()
        .status(status)
        // CORS
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type,Authorization,X-Amz-Date,X-Api-Key,X-Amz-Security-Token",
        )
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .body(body.into())
        .map_err(|e| e.into())
}

fn error_response(err: Error) -> Response<Body> {
    let resp = ErrorResponse {
        message: err.to_string(),
    };
    let body: String = serde_json::to_string(&resp).unwrap();

    Response::builder()
        .status(err.http_status())
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type,Authorization,X-Amz-Date,X-Api-Key,X-Amz-Security-Token",
        )
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .body(body.into())
        .expect("failed to render response")
}
