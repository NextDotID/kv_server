#[cfg(test)]
mod tests {
    use diesel::{insert_into, PgConnection, RunQueryDsl};
    use fake::{Fake, Faker};
    use libsecp256k1::PublicKey;
    use serde_json::json;

    use crate::{
        crypto::secp256k1::Secp256k1KeyPair,
        error::Error,
        model::{
            establish_connection,
            kv_chains::{KVChain, NewKVChain},
        },
        schema::kv_chains::dsl::*,
        util::{naive_now, vec_to_base64},
    };

    fn before_each(connection: &PgConnection) -> Result<(), Error> {
        let _ = env_logger::try_init();
        // Clear DB
        diesel::delete(kv_chains).execute(connection)?;
        diesel::delete(crate::schema::kv::dsl::kv).execute(connection)?;
        Ok(())
    }

    fn generate_data(conn: &PgConnection, persona_pubkey: &PublicKey) -> Result<KVChain, Error> {
        let new_uuid = ::uuid::Uuid::new_v4();
        let persona_bytes = persona_pubkey.serialize().to_vec();
        let new_platform: String = Faker.fake();
        let new_identity: String = Faker.fake();
        insert_into(kv_chains)
            .values(&NewKVChain {
                uuid: new_uuid,
                persona: persona_bytes,
                platform: new_platform,
                identity: new_identity,
                patch: json!({ "test": "abc" }),
                previous_id: None,
                signature: vec![1],
                signature_payload: "".into(),
                created_at: naive_now(),
            })
            .get_result(conn)
            .map_err(|e| e.into())
    }

    #[test]
    fn test_find_last_link() -> Result<(), Error> {
        let conn = establish_connection();
        before_each(&conn)?;
        let Secp256k1KeyPair {
            public_key: pk,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = generate_data(&conn, &pk)?;

        let found = KVChain::find_last_link(&conn, &pk)?.unwrap();
        assert_eq!(found.id, link.id);
        assert_eq!(found.uuid, link.uuid);
        Ok(())
    }

    #[test]
    fn test_newkv_finalize() -> Result<(), Error> {
        let conn = establish_connection();
        before_each(&conn)?;
        let Secp256k1KeyPair {
            public_key: pk,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = generate_data(&conn, &pk)?;

        let new_identity: String = Faker.fake();
        let new_kvchain = NewKVChain {
            uuid: ::uuid::Uuid::new_v4(),
            persona: pk.serialize().to_vec(),
            platform: "facebook".into(),
            identity: new_identity.clone(),
            patch: json!({"test": "def"}),
            previous_id: Some(link.id),
            signature: vec![2],
            signature_payload: "".into(),
            created_at: naive_now(),
        };
        let new_link = new_kvchain.finalize(&conn)?;
        assert_eq!(new_link.previous_id.unwrap(), link.id);
        assert_eq!(new_link.platform, "facebook");
        assert_eq!(new_link.identity, new_identity);
        Ok(())
    }

    #[test]
    fn test_newkv_for_persona() -> Result<(), Error> {
        let conn = establish_connection();
        before_each(&conn)?;
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = generate_data(&conn, &public_key)?;

        let new_kv = NewKVChain::for_persona(&conn, &public_key)?;
        assert_eq!(new_kv.persona, public_key.serialize().to_vec());
        assert_eq!(new_kv.previous_id, Some(link.id));
        Ok(())
    }

    #[test]
    fn test_newkv_sign_body() -> Result<(), Error> {
        let conn = establish_connection();
        before_each(&conn)?;
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = generate_data(&conn, &public_key)?;
        let new_kv = NewKVChain::for_persona(&conn, &public_key)?;

        let sign_body = new_kv.generate_signature_payload()?;
        assert!(sign_body.previous.unwrap() == vec_to_base64(&link.signature));
        assert!(sign_body.uuid == new_kv.uuid);
        Ok(())
    }

    #[test]
    fn test_newkv_sign_and_verify() -> Result<(), Error> {
        let conn = establish_connection();
        before_each(&conn)?;
        let keypair = Secp256k1KeyPair::generate();
        generate_data(&conn, &keypair.public_key)?;
        let mut new_kv = NewKVChain::for_persona(&conn, &keypair.public_key)?;
        new_kv.platform = "facebook".into();
        new_kv.identity = Faker.fake();
        new_kv.patch = json!({"test": ["abc"]});

        let sig = new_kv.sign(&keypair)?;
        new_kv.signature = sig;
        new_kv.signature_payload =
            serde_json::to_string(&new_kv.generate_signature_payload()?).unwrap();
        assert!(new_kv.validate().is_ok());

        Ok(())
    }
}
