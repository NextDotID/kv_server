use diesel::prelude::*;
use diesel::PgConnection;

pub mod kv;

pub fn establish_connection() -> PgConnection {
    let database_url = crate::config::C.database_url();
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn do_migration() {
    todo!()
}
