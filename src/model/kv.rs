use crate::error::Error;

#[derive(Queryable)]
pub struct KV {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub platform: String,
    pub identity: String,
    pub content: String, // FIXME: ???
}

pub fn find_or_create(platform: &str, identity: &str) -> Result<KV, Error> {

    todo!()
}
