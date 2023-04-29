use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use kv_server::{controller::lambda::entrypoint, model};
use lambda_http::{service_fn, Error as LambdaError};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let _ = env_logger::try_init();
    model::establish_connection().run_pending_migrations(MIGRATIONS).expect("Migration failed");

    lambda_http::run(service_fn(entrypoint)).await?;
    Ok(())
}
