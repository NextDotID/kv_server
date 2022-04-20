-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS idx_kv_chains_uuid;
DROP INDEX IF EXISTS idx_signature;
DROP INDEX IF EXISTS idx_previous_id;
DROP INDEX IF EXISTS idx_persona;

DROP TABLE IF EXISTS kv_chains;
