#[macro_use]
extern crate diesel_migrations;

use kv_server::{controller::lambda::entrypoint, model};
use lambda_http::{service_fn, Error as LambdaError};

embed_migrations!("./migrations");

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let _ = env_logger::try_init();
    embedded_migrations::run(&model::establish_connection()).unwrap();

    lambda_http::run(service_fn(entrypoint)).await?;
    Ok(())
}
