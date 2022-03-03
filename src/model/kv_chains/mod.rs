mod tests;

use serde_json::json;
use ::uuid::Uuid;
use diesel::{insert_into, prelude::*, PgConnection};
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    schema::{kv_chains, kv_chains::dsl::*}, crypto::{secp256k1::Secp256k1KeyPair, util::compress_public_key},
};

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, Debug)]
#[table_name = "kv_chains"]
#[belongs_to(KVChain, foreign_key = "previous_id")]
pub struct KVChain {
    pub id: i32,
    pub uuid: Uuid,
    pub persona: Vec<u8>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub previous_id: Option<i32>,
    pub signature: Vec<u8>,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "kv_chains"]
pub struct NewKVChain {
    pub uuid: Uuid,
    pub persona: Vec<u8>,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
    pub previous_id: Option<i32>,
    pub signature: Vec<u8>,
}

impl NewKVChain {
    /// Generate a new KVChain append request for given persona.
    pub fn for_persona(conn: &PgConnection, persona_given: &PublicKey) -> Result<NewKVChain, Error> {
        let last_link = KVChain::find_last_link(conn, persona_given)?;
        let persona_vec = persona_given.serialize().to_vec();
        let mut new_kvchain = NewKVChain{
            uuid: ::uuid::Uuid::new_v4(),
            persona: persona_vec,
            platform: "".into(),
            identity: "".into(),
            patch: json!({}),
            previous_id: None,
            signature: vec![],
        };

        if let Some(last_link_instance) = last_link {
            new_kvchain.previous_id = Some(last_link_instance.id);
        }

        Ok(new_kvchain)
    }

    /// Convert persona byte vec into `PublicKey` instance.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::parse_slice(self.persona.as_slice(), None).unwrap()
    }

    /// Generate signature body for this KVChain request.
    pub fn sign_body(&self) -> String {
        let body: serde_json::Value = json!({
            "version": "1",
            "uuid": self.uuid.to_string(),
            "persona": compress_public_key(&self.public_key()),
            "platform": self.platform,
            "identity": self.identity,
            "patch": self.patch,
            "previous": "", // TODO
        });

        serde_json::to_string(&body).unwrap()
    }

    /// Generate a signature using given keypair.
    /// For development and test only.
    pub fn sign(&self, keypair: &Secp256k1KeyPair) -> Result<Vec<u8>, Error> {
        let body = self.sign_body();
        keypair.personal_sign(&body)
    }

    /// Validate if this KVChain has valid signature
    pub fn validate(&self) -> Result<(), Error> {
        let sign_body = self.sign_body();
        let key_pair = Secp256k1KeyPair::from_pubkey_vec(&self.persona)?;
        let pk = Secp256k1KeyPair::recover_from_personal_signature(&self.signature, &sign_body)?;

        if pk != key_pair.public_key {
            Err(Error::SignatureValidationError("Public key mismatch".into()))
        } else {
            Ok(())
        }
    }
}

impl KVChain {
    /// Find last link of given persona.
    /// `None` if not found.
    pub fn find_last_link(
        conn: &PgConnection,
        persona_pubkey: &PublicKey,
    ) -> Result<Option<KVChain>, Error> {
        let persona_bytes = persona_pubkey.serialize().to_vec();
        let found: Option<KVChain> = kv_chains
            .filter(persona.eq(persona_bytes))
            .get_result(conn)
            .optional()?;

        Ok(found)
    }

    /// Append a new link from current link.
    /// No check of validity is performed.
    pub fn append(
        &self,
        conn: &PgConnection,
        new_kvchain: &NewKVChain,
    ) -> Result<KVChain, Error> {
        insert_into(kv_chains)
            .values(new_kvchain)
            .get_result(conn)
            .map_err(|e| e.into())
    }
}
