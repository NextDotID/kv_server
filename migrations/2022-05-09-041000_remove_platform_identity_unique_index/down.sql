-- This file should undo anything in `up.sql`

CREATE UNIQUE INDEX idx_platform_identity
ON kv((lower(platform)), (lower(identity)));
