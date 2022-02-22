use lambda_http::{service_fn, Error as LambdaError};
use aws_lambda_rust_template::controller::entrypoint;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    lambda_http::run(service_fn(entrypoint)).await?;
    Ok(())
}
