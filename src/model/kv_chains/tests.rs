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
            kv_chains::{KVChain, NewKVChain, find_kv_chain_by_id},
        },
        schema::kv_chains::dsl::*,
        util::{naive_now, vec_to_base64},
    };

    fn before_each(connection: &mut PgConnection) -> Result<(), Error> {
        let _ = env_logger::try_init();
        // Clear DB
        diesel::delete(kv_chains).execute(connection)?;
        diesel::delete(crate::schema::kv::dsl::kv).execute(connection)?;
        Ok(())
    }

    fn create_link_and_insert(conn: &mut PgConnection, persona_pubkey: &PublicKey, other_arweave_id: Option<String>) -> Result<KVChain, Error> {
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
                arweave_id: other_arweave_id,
            })
            .get_result(conn)
            .map_err(|e| e.into())
    }

    #[test]
    fn test_find_last_link() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key: pk,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = create_link_and_insert(&mut conn, &pk, None)?;

        let found = KVChain::find_last_link(&mut conn, &pk)?.unwrap();
        assert_eq!(found.id, link.id);
        assert_eq!(found.uuid, link.uuid);
        Ok(())
    }

    #[test]
    fn test_newkv_finalize() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key: pk,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = create_link_and_insert(&mut conn, &pk, None)?;

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
            arweave_id: None,
        };
        let new_link = new_kvchain.finalize(&mut conn)?;
        assert_eq!(new_link.previous_id.unwrap(), link.id);
        assert_eq!(new_link.platform, "facebook");
        assert_eq!(new_link.identity, new_identity);
        Ok(())
    }

    #[test]
    fn test_newkv_for_persona() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = create_link_and_insert(&mut conn, &public_key, None)?;

        let new_kv = NewKVChain::for_persona(&mut conn, &public_key)?;
        assert_eq!(new_kv.persona, public_key.serialize().to_vec());
        assert_eq!(new_kv.previous_id, Some(link.id));
        Ok(())
    }

    #[test]
    fn test_newkv_sign_body() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let link = create_link_and_insert(&mut conn, &public_key, None)?;
        let new_kv = NewKVChain::for_persona(&mut conn, &public_key)?;

        let sign_body = new_kv.generate_signature_payload()?;
        assert!(sign_body.previous.unwrap() == vec_to_base64(&link.signature));
        assert!(sign_body.uuid == new_kv.uuid);
        Ok(())
    }

    #[test]
    fn test_newkv_sign_and_verify() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let keypair = Secp256k1KeyPair::generate();
        create_link_and_insert(&mut conn, &keypair.public_key, None)?;
        let mut new_kv = NewKVChain::for_persona(&mut conn, &keypair.public_key)?;
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

    #[test]
    fn test_insert_arweave_id() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let insert_arweave_id = Some(String::from("xahZGuFbiayYiIuEG4dSpMS3XWcNH02vAXYGt4t2WFA"));

        let link = create_link_and_insert(&mut conn, &public_key, None)?;
        assert_eq!(link.arweave_id, None);

        let _ = link.insert_arweave_id(&mut conn, insert_arweave_id.clone());
        let (find_link, is_found) = find_kv_chain_by_id(&mut conn, link.id)?;
        assert!(is_found);
        assert_eq!(find_link.arweave_id, insert_arweave_id);

        Ok(())
    }

    #[test]
    fn test_find_last_chain_arweave() -> Result<(), Error> {
        let mut conn = establish_connection();
        before_each(&mut conn)?;
        let Secp256k1KeyPair {
            public_key: pk,
            secret_key: _,
        } = Secp256k1KeyPair::generate();
        let first_link = create_link_and_insert(&mut conn, &pk, Some("first".into()))?;

        let new_identity: String = Faker.fake();
        let second_link = NewKVChain {
            uuid: ::uuid::Uuid::new_v4(),
            persona: pk.serialize().to_vec(),
            platform: "facebook".into(),
            identity: new_identity.clone(),
            patch: json!({"test": "def"}),
            previous_id: Some(first_link.id),
            signature: vec![2],
            signature_payload: "".into(),
            created_at: naive_now(),
            arweave_id: Some("second".into()),
        };

        let found_arweave_id = second_link.find_last_chain_arweave(&mut conn)?;
        assert_eq!(found_arweave_id, Some("first".into()));
        Ok(())
    }
}
