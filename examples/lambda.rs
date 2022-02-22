use lambda_http::{service_fn, Error as LambdaError};
use kv_server::controller::lambda::entrypoint;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    lambda_http::run(service_fn(entrypoint)).await?;
    Ok(())
}
