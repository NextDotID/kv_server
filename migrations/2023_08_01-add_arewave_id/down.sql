-- THis file should undo anything in `up.sql`

ALTER TABLE kv_chains
DROP COLUMN areweave_id;

ALTER TABLE kv
DROP COLUMN areweave_id;

