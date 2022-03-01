table! {
    kv (id) {
        id -> Int4,
        uuid -> Nullable<Uuid>,
        platform -> Varchar,
        identity -> Varchar,
        content -> Jsonb,
        persona -> Bytea,
    }
}
