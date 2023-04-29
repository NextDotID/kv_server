mod tests;

use crate::{
    error::Error,
    schema::kv::{self, dsl::*},
};
use ::uuid::Uuid;
use diesel::{prelude::*, PgConnection};
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name = kv)]
pub struct KV {
    pub id: i32,
    pub uuid: Option<Uuid>,
    pub platform: String,
    pub identity: String,
    pub content: serde_json::Value,
    pub persona: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = kv)]
pub struct NewKV {
    pub platform: String,
    pub identity: String,
    pub persona: Vec<u8>,
}

impl KV {
    /// Apply a patch JSON onto current record.
    pub fn patch(&self, conn: &mut PgConnection, patch: &serde_json::Value) -> Result<(), Error> {
        let mut patched_content = self.content.clone();
        json_patch::merge(&mut patched_content, patch);

        diesel::update(self)
            .set(content.eq(patched_content))
            .execute(conn)
            .map_err(|e| Error::from(e))?;
        Ok(())
    }

    /// `"0xHEXSTRING"` of persona (avatar). Uncompressed form.
    pub fn avatar(&self) -> String {
        format!("0x{}", hex::encode(self.persona.clone()))
    }
}

/// Find all KVs belong to given persona.
pub fn find_all_by_persona(
    conn: &mut PgConnection,
    persona_given: &PublicKey,
) -> Result<Vec<KV>, Error> {
    let persona_vec = persona_given.serialize().to_vec();
    let result: Vec<KV> = kv
        .filter(persona.eq(&persona_vec))
        .get_results(conn)
        .map_err(|e| Error::from(e))?;

    Ok(result)
}

/// Find all KVs belongs to given platform-identity pair.
pub fn find_all_by_identity(
    conn: &mut PgConnection,
    platform_given: &str,
    identity_given: &str,
) -> Result<Vec<KV>, Error> {
    let result: Vec<KV> = kv
        .filter(platform.eq(platform_given))
        .filter(identity.eq(identity_given))
        .get_results(conn)?;

    Ok(result)
}

/// Returns (KV, is_founded)
pub fn find_or_create(
    conn: &mut PgConnection,
    expected_platform: &str,
    expected_identity: &str,
    expected_persona: &PublicKey,
) -> Result<(KV, bool), Error> {
    let persona_vec: Vec<u8> = expected_persona.serialize().to_vec();
    let found: Option<KV> = kv
        .filter(platform.eq(expected_platform))
        .filter(identity.eq(expected_identity))
        .filter(persona.eq(&persona_vec))
        .first(conn)
        .optional()?;

    // Found
    if found.is_some() {
        return Ok((found.unwrap(), true));
    }

    // Create
    diesel::insert_into(kv::table)
        .values((
            platform.eq(expected_platform),
            identity.eq(expected_identity),
            persona.eq(&persona_vec),
        ))
        .get_result(conn)
        .map(|created| (created, false))
        .map_err(|e| e.into())
}
