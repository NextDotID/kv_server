#[cfg(test)]
mod tests {
    use diesel::{QueryDsl, RunQueryDsl, PgConnection};

    use crate::{
        schema::kv::dsl::*,
        error::Error,
        model::{establish_connection, kv::find_or_create},
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
        let (kv_created, is_found) = find_or_create(&c, "twitter", "yeiwb").unwrap();
        assert_eq!(is_found, false);
        assert_eq!(kv_created.platform, "twitter".to_string());
        assert_eq!(kv_created.identity, "yeiwb".to_string());
        assert_eq!(kv_created.uuid, None);
        assert!(kv_created.content.is_object());
        assert_eq!(Ok(1), kv.count().get_result(&c));

        let (_new_kv, is_found_2) = find_or_create(&c, "twitter", "yeiwb").unwrap();
        assert!(is_found_2);
        assert_eq!(Ok(1), kv.count().get_result(&c));
        Ok(())
    }
}
