-- Your SQL goes here
CREATE TABLE kv_chains (
       id SERIAL PRIMARY KEY,
       uuid UUID NOT NULL,
       persona bytea NOT NULL,
       platform VARCHAR NOT NULL,
       identity VARCHAR NOT NULL,
       patch JSONB NOT NULL DEFAULT '{}'::jsonb,
       previous_id INTEGER,
       signature bytea NOT NULL
);

CREATE UNIQUE INDEX idx_kv_chains_uuid ON kv_chains (uuid);
CREATE INDEX idx_persona ON kv_chains (persona);
CREATE INDEX idx_signature ON kv_chains (signature);
CREATE INDEX idx_previous_id ON kv_chains (previous_id);
