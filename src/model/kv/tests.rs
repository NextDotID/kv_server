#[cfg(test)]
mod tests {
    use diesel::{PgConnection, QueryDsl, RunQueryDsl};
    use serde_json::json;
    const PUBKEY: &str = "0x04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575";
    use fake::{Fake, Faker};

    use crate::{
        crypto::secp256k1::Secp256k1KeyPair,
        error::Error,
        model::{
            establish_connection,
            kv::{find_all_by_persona, find_or_create},
        },
        schema::kv::dsl::*,
    };

    fn before_each(connection: &PgConnection) -> Result<(), Error> {
        let _ = env_logger::try_init();
        // Clear DB
        diesel::delete(kv).execute(connection)?;
        assert_eq!(Ok(0), kv.count().get_result(connection));
        Ok(())
    }

    #[test]
    fn test_find_or_create_success() -> Result<(), Error> {
        let c = establish_connection();
        before_each(&c)?;
        let username: String = Faker.fake();

        let Secp256k1KeyPair {
            public_key: pubkey,
            secret_key: _,
        } = Secp256k1KeyPair::from_pubkey_hex(&PUBKEY.to_string())?;
        let (kv_created, is_found) = find_or_create(&c, "twitter", &username, &pubkey).unwrap();
        assert_eq!(is_found, false);
        assert_eq!(kv_created.platform, "twitter".to_string());
        assert_eq!(kv_created.identity, username);
        assert_eq!(kv_created.uuid, None);
        assert!(kv_created.content.is_object());

        let (_new_kv, is_found_2) = find_or_create(&c, "twitter", &username, &pubkey).unwrap();
        assert!(is_found_2);
        Ok(())
    }

    #[test]
    fn test_patch() -> Result<(), Error> {
        let c = establish_connection();
        before_each(&c)?;
        let username: String = Faker.fake();
        let Secp256k1KeyPair {
            public_key: pubkey,
            secret_key: _,
        }: Secp256k1KeyPair = Secp256k1KeyPair::from_pubkey_hex(&PUBKEY.to_string())?;

        let (kv_created, _) = find_or_create(&c, "twitter", &username, &pubkey)?;
        kv_created.patch(&c, &json!({"test": "abc"}))?;

        let (kv_found, _) = find_or_create(&c, "twitter", &username, &pubkey)?;
        assert_eq!(kv_found.content, json!({"test": "abc"}));

        kv_found.patch(&c, &json!({ "test": null }))?;

        let (kv_found_2, _) = find_or_create(&c, "twitter", &username, &pubkey)?;
        assert_eq!(kv_found_2.content, json!({}));

        Ok(())
    }

    #[test]
    fn test_find_all_by_persona() -> Result<(), Error> {
        let c = establish_connection();
        before_each(&c)?;
        let username: String = Faker.fake();
        let Secp256k1KeyPair {
            public_key: pubkey,
            secret_key: _,
        } = Secp256k1KeyPair::generate();

        find_or_create(&c, "twitter", &username, &pubkey).unwrap();

        let result = find_all_by_persona(&c, &pubkey).unwrap();
        assert_eq!(result.len(), 1);
        Ok(())
    }
}
