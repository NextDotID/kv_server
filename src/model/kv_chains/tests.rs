#[cfg(test)]
mod tests {
    use diesel::{insert_into, PgConnection, QueryDsl, RunQueryDsl};
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
    };

    fn before_each(connection: &PgConnection) -> Result<(), Error> {
        let _ = env_logger::try_init();
        // Clear DB
        diesel::delete(kv_chains).execute(connection)?;
        diesel::delete(crate::schema::kv::dsl::kv).execute(connection)?;
        assert_eq!(Ok(0), kv_chains.count().get_result(connection));
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
    fn test_newkv_append() -> Result<(), Error> {
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
        };
        let new_link = link.append(&conn, &new_kvchain)?;
        assert_eq!(new_link.previous_id.unwrap(), link.id);
        assert_eq!(new_link.platform, "facebook");
        assert_eq!(new_link.identity, new_identity);
        Ok(())
    }
}
