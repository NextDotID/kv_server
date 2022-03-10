mod tests;

use ::uuid::Uuid;
use diesel::{insert_into, prelude::*, PgConnection};
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    crypto::{secp256k1::Secp256k1KeyPair, util::compress_public_key},
    error::Error,
    model::{establish_connection, kv::KV},
    schema::{kv_chains, kv_chains::dsl::*},
    util::vec_to_base64,
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
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub signature_payload: String,
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
    pub signature_payload: String,
}

impl NewKVChain {
    /// Generate a new KVChain append request for given persona.
    pub fn for_persona(
        conn: &PgConnection,
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
        })
    }

    /// Convert persona byte vec into `PublicKey` instance.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::parse_slice(self.persona.as_slice(), None).unwrap()
    }

    /// Generate signature body for this KVChain request.
    pub fn generate_signature_payload(&self) -> Result<String, Error> {
        let mut previous_sig: Option<String> = None;
        if let Some(prev_id) = self.previous_id {
            let conn = establish_connection();
            let prev_kv_sig_bytes = kv_chains
                .select(signature)
                .filter(id.eq(prev_id))
                .get_result::<Vec<u8>>(&conn)
                .map_err(|e| Error::from(e))?;
            previous_sig = Some(vec_to_base64(&prev_kv_sig_bytes));
        }

        let body: serde_json::Value = json!({
            "version": "1",
            "uuid": self.uuid.to_string(),
            "persona": compress_public_key(&self.public_key()),
            "platform": self.platform,
            "identity": self.identity,
            "patch": self.patch,
            "previous": previous_sig,
        });

        let body_string = serde_json::to_string(&body).unwrap();
        Ok(body_string)
    }

    /// Generate a signature using given keypair.
    /// For development and test only.
    pub fn sign(&self, keypair: &Secp256k1KeyPair) -> Result<Vec<u8>, Error> {
        let body = self.generate_signature_payload()?;
        keypair.personal_sign(&body)
    }

    /// Validate if this KVChain has valid signature.  It'll read
    /// `self.signature_payload` as signature body, so make sure it is
    /// prepared before calling this.
    pub fn validate(&self) -> Result<(), Error> {
        let recovered_pk =
            Secp256k1KeyPair::recover_from_personal_signature(&self.signature, &self.signature_payload)?;

        if recovered_pk != self.public_key() {
            Err(Error::SignatureValidationError(
                "Public key mismatch".into(),
            ))
        } else {
            Ok(())
        }
    }

    /// Save myself into DB.
    pub fn finalize(&self, conn: &PgConnection) -> Result<KVChain, Error> {
        insert_into(kv_chains)
            .values(self)
            .get_result(conn)
            .map_err(|e| e.into())
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

    /// Perform patch on KV record.
    pub fn perform_patch(&self, conn: &PgConnection) -> Result<KV, Error> {
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
}
