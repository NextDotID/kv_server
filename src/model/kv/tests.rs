#[cfg(test)]
mod tests {
    use diesel::{PgConnection, QueryDsl, RunQueryDsl};
    use serde_json::json;
    const PUBKEY: &str = "0x04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575";

    use crate::{
        error::Error,
        model::{establish_connection, kv::find_or_create},
        schema::kv::dsl::*,
    };

    fn before_each(connection: &PgConnection) -> Result<(), Error> {
        env_logger::init();
        // Clear DB
        diesel::delete(kv).execute(connection)?;
        assert_eq!(Ok(0), kv.count().get_result(connection));
        Ok(())
    }

    #[test]
    fn test_find_or_create_success() -> Result<(), Error> {
        let c = establish_connection();
        before_each(&c)?;
        let (kv_created, is_found) = find_or_create(&c, "twitter", "yeiwb", &PUBKEY.to_string(),).unwrap();
        assert_eq!(is_found, false);
        assert_eq!(kv_created.platform, "twitter".to_string());
        assert_eq!(kv_created.identity, "yeiwb".to_string());
        assert_eq!(kv_created.uuid, None);
        assert!(kv_created.content.is_object());
        assert_eq!(Ok(1), kv.count().get_result(&c));

        let (_new_kv, is_found_2) = find_or_create(&c, "twitter", "yeiwb", &PUBKEY.to_string(),).unwrap();
        assert!(is_found_2);
        assert_eq!(Ok(1), kv.count().get_result(&c));
        Ok(())
    }

    #[test]
    fn test_patch() -> Result<(), Error> {
        let c = establish_connection();
        before_each(&c)?;

        let (kv_created, _) = find_or_create(&c, "twitter", "yeiwb", &PUBKEY.to_string())?;
        kv_created.patch(&c, &json!({"test": "abc"}))?;

        let (kv_found, _) = find_or_create(&c, "twitter", "yeiwb", &PUBKEY.to_string())?;
        assert_eq!(kv_found.content, json!({"test": "abc"}));

        kv_found.patch(&c, &json!({"test": null}))?;

        let (kv_found_2, _) = find_or_create(&c, "twitter", "yeiwb", &PUBKEY.to_string())?;
        assert_eq!(kv_found_2.content, json!({}));

        Ok(())
    }
}
