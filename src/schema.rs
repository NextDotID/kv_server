// @generated automatically by Diesel CLI.

diesel::table! {
    kv (id) {
        id -> Int4,
        uuid -> Nullable<Uuid>,
        platform -> Varchar,
        identity -> Varchar,
        content -> Jsonb,
        persona -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        arweave_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    kv_chains (id) {
        id -> Int4,
        uuid -> Uuid,
        persona -> Bytea,
        platform -> Varchar,
        identity -> Varchar,
        patch -> Jsonb,
        previous_id -> Nullable<Int4>,
        signature -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        signature_payload -> Varchar,
        arweave_id -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    kv,
    kv_chains,
);
