-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS idx_uuid;
DROP INDEX IF EXISTS idx_platform_identity;

DROP TABLE IF EXISTS kv;
