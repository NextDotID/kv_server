use kv_server::{config::C, error::Error};

fn main() -> Result<(), Error> {
    let config = C.clone();
    println!("{}", config.web.port);
    Ok(())
}
