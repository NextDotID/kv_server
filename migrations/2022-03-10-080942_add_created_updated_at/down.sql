-- This file should undo anything in `up.sql`

ALTER TABLE kv_chains
DROP COLUMN created_at;
ALTER TABLE kv_chains
DROP COLUMN updated_at;

ALTER TABLE kv
DROP COLUMN created_at;
ALTER TABLE kv
DROP COLUMN updated_at;
