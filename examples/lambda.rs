use kv_server::controller::lambda::entrypoint;
use lambda_http::{service_fn, Error as LambdaError};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    lambda_http::run(service_fn(entrypoint)).await?;
    Ok(())
}
