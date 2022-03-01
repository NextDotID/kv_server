-- Your SQL goes here

CREATE TABLE kv (
       id SERIAL PRIMARY KEY,
       uuid UUID,
       platform VARCHAR NOT NULL,
       identity VARCHAR NOT NULL,
       content JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE UNIQUE INDEX idx_uuid ON kv (uuid);

CREATE UNIQUE INDEX idx_platform_identity
ON kv((lower(platform)), (lower(identity)));
