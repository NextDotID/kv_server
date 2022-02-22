use crate::error::Error;
use crate::controller::{
    hello, upload, error_response,
    Request as OurRequest,
    Response as OurResponse,
    Body as OurBody,
};
use http::{Method, StatusCode};
use std::future::Future;
use lambda_http::{
    Body as LambdaBody, Error as LambdaError, IntoResponse, Request as LambdaRequest, Response as LambdaResponse,
};
use log::info;

/// Translate between `lambda_http` `Body` and our `Body`.
async fn parse<F>(req: LambdaRequest, controller: fn(OurRequest) -> F) -> LambdaResponse<LambdaBody>
where
    F: Future<Output = Result<OurResponse, Error>>,
{
    let (parts, old_body) = req.into_parts();
    let body: OurBody = crate::controller::LambdaBody(old_body).into();
    let new_req: OurRequest = http::Request::from_parts(parts, body);

    match controller(new_req).await {
        Ok(resp) => {
            let (parts, our_resp) = resp.into_parts();
            let resp = lambda_http::Body::Text(our_resp);
            LambdaResponse::from_parts(parts, resp)
        },
        Err(err) => {
            let (parts, our_resp) = error_response(err).into_parts();
            let resp = lambda_http::Body::Text(our_resp);
            LambdaResponse::from_parts(parts, resp)
        }
    }
}

/// Main entrypoint for `lambda_http`.
pub async fn entrypoint(req: LambdaRequest) -> Result<impl IntoResponse, LambdaError> {
    info!(
        "{} {}",
        req.method().to_string(),
        req.uri().path().to_string()
    );

    Ok(match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => parse(req, hello::controller).await,
        (&Method::POST, "/upload") => parse(req, upload::controller).await,

        _ => LambdaResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .expect("Failed to render response"),
    })
}
