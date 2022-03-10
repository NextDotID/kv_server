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

table! {
    kv_chains (id) {
        id -> Int4,
        uuid -> Uuid,
        persona -> Bytea,
        platform -> Varchar,
        identity -> Varchar,
        patch -> Jsonb,
        previous_id -> Nullable<Int4>,
        signature -> Bytea,
    }
}

allow_tables_to_appear_in_same_query!(kv, kv_chains,);
