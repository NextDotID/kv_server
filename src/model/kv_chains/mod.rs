mod tests;

use ::uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::{insert_into, prelude::*, PgConnection};
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    crypto::{secp256k1::Secp256k1KeyPair, util::hex_public_key},
    error::Error,
    model::{establish_connection, kv::KV},
    schema::{kv_chains, kv_chains::dsl::*},
    util::{naive_now, vec_to_base64},
};

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = kv_chains, belongs_to(KVChain, foreign_key = previous_id))]
pub struct KVChain {
    pub id: i32,
    pub uuid: Uuid,
    pub persona: Vec<u8>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub previous_id: Option<i32>,
    pub signature: Vec<u8>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub signature_payload: String,
    pub arweave_id: Option<String>,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = kv_chains)]
pub struct NewKVChain {
    pub uuid: Uuid,
    pub persona: Vec<u8>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub previous_id: Option<i32>,
    pub signature: Vec<u8>,
    pub signature_payload: String,
    pub created_at: NaiveDateTime,
    pub arweave_id: Option<String>, 
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignPayload {
    pub version: String,
    pub uuid: Uuid,
    pub avatar: String,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub created_at: i64,
    pub previous: Option<String>,
}

impl NewKVChain {
    /// Generate a new KVChain append request for given persona.
    pub fn for_persona(
        conn: &mut PgConnection,
        persona_given: &PublicKey,
    ) -> Result<NewKVChain, Error> {
        let last_link = KVChain::find_last_link(conn, persona_given)?;
        let persona_vec = persona_given.serialize().to_vec();

        Ok(NewKVChain {
            uuid: ::uuid::Uuid::new_v4(),
            persona: persona_vec,
            platform: "".into(),
            identity: "".into(),
            patch: json!({}),
            previous_id: if let Some(last_link_instance) = last_link {
                Some(last_link_instance.id)
            } else {
                None
            },
            signature: vec![],
            signature_payload: "".into(),
            created_at: naive_now(),
            arweave_id: None,
        })
    }

    /// Convert persona byte vec into `PublicKey` instance.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::parse_slice(self.persona.as_slice(), None).unwrap()
    }

    /// Generate signature body for this KVChain request.
    pub fn generate_signature_payload(&self) -> Result<SignPayload, Error> {
        let mut previous_sig: Option<String> = None;
        if let Some(prev_id) = self.previous_id {
            let mut conn = establish_connection();
            let prev_kv_sig_bytes = kv_chains
                .select(signature)
                .filter(id.eq(prev_id))
                .get_result::<Vec<u8>>(&mut conn)
                .map_err(|e| Error::from(e))?;
            previous_sig = Some(vec_to_base64(&prev_kv_sig_bytes));
        }

        Ok(SignPayload {
            version: "1".into(),
            uuid: self.uuid.clone(),
            avatar: hex_public_key(&self.public_key()),
            platform: self.platform.clone(),
            identity: self.identity.clone(),
            patch: self.patch.clone(),
            previous: previous_sig,
            created_at: self.created_at.timestamp(),
        })
    }

    /// Generate a signature using given keypair.
    /// For development and test only.
    pub fn sign(&self, keypair: &Secp256k1KeyPair) -> Result<Vec<u8>, Error> {
        let body = self.generate_signature_payload()?;
        keypair.personal_sign(&serde_json::to_string(&body).unwrap())
    }

    /// Validate if this KVChain has valid signature.  It'll read
    /// `self.signature_payload` as signature body, so make sure it is
    /// prepared before calling this.
    pub fn validate(&self) -> Result<(), Error> {
        let recovered_pk = Secp256k1KeyPair::recover_from_personal_signature(
            &self.signature,
            &self.signature_payload,
        )?;

        if recovered_pk != self.public_key() {
            Err(Error::SignatureValidationError(
                "Public key mismatch".into(),
            ))
        } else {
            Ok(())
        }
    }

    /// Save myself into DB.
    pub fn finalize(&self, conn: &mut PgConnection) -> Result<KVChain, Error> {
        insert_into(kv_chains)
            .values(self)
            .get_result(conn)
            .map_err(|e| e.into())
    }

    /// Find last chain arweave id.
    pub fn find_last_chain_arweave(
        self,
        conn: &mut PgConnection,
    ) -> Result<Option<String>, Error> {

        if self.previous_id.is_none() {
            return Ok(None);
        }

        let found: Option<KVChain> = kv_chains
            .filter(id.eq(self.previous_id.unwrap()))
            .get_result(conn)
            .optional()?;

        if let Some(kv_chain) = found {
            Ok(kv_chain.arweave_id)
        } else {
            Ok(None)
        }
    }
}

impl KVChain {
    /// Find last link of given persona.
    /// `None` if not found.
    pub fn find_last_link(
        conn: &mut PgConnection,
        persona_pubkey: &PublicKey,
    ) -> Result<Option<KVChain>, Error> {
        let persona_bytes = persona_pubkey.serialize().to_vec();
        let found: Option<KVChain> = kv_chains
            .filter(persona.eq(persona_bytes))
            .get_result(conn)
            .optional()?;

        Ok(found)
    }

    /// Perform patch on KV record.
    pub fn perform_patch(&self, conn: &mut PgConnection) -> Result<KV, Error> {
        use crate::model::kv;

        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_vec(&self.persona)?;

        let (kv_record, _is_new) =
            kv::find_or_create(conn, &self.platform, &self.identity, &public_key)?;
        kv_record.patch(conn, &self.patch)?;
        
        Ok(kv_record)
    }

    /// Insert arweave id into kv and kv_chains.
    pub fn insert_arweave_id(&self, conn: &mut PgConnection, new_arweave: Option<String>) -> Result<(), Error> {
        
        use crate::model::kv;
        
        // insert arweave id into table kv
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_vec(&self.persona)?;

        let (kv_record, _is_new) =
            kv::find_or_create(conn, &self.platform, &self.identity, &public_key)?;
        kv_record.update_arweave(conn, new_arweave.clone())?;
        
        // insert arweave id into table kv_chains
        diesel::update(self)
            .set(arweave_id.eq(new_arweave))
            .execute(conn)
            .map_err(|e| Error::from(e))?;

        Ok(())
    }
}


/// Returns (KVChain, is_founded)
pub fn find_kv_chain_by_id(
    conn: &mut PgConnection,
    other_id: i32,
) -> Result<(KVChain, bool), Error> {

    let found = kv_chains
        .filter(id.eq(other_id))
        .first(conn)
        .optional()?;

    // Found
    if found.is_some() {
        return Ok((found.unwrap(), true));
    } 
    
    // Not found
    Ok((found.unwrap(), false))
}

/// Find all KVChains belongs to given platform-identity pair.
pub fn find_all_by_identity(
    conn: &mut PgConnection,
    platform_given: &str,
    identity_given: &str,
) -> Result<Vec<KVChain>, Error> {

    let result: Vec<KVChain> = kv_chains
        .filter(platform.eq(platform_given))
        .filter(identity.eq(identity_given))
        .get_results(conn)?;

    Ok(result)
}