-- sqlite3 -line 2025-06-29T04:45:18.455186748Z.sqlite3 < assets/db.sql
--
-- CREATE TABLE IF NOT EXISTS db (
--   file TEXT NOT NULL,
--   name TEXT NOT NULL,
--   hash TEXT NOT NULL PRIMARY KEY,
--   hash_direct TEXT NOT NULL,
--   inst_kind TEXT,
--   proof_kind TEXT,
--   attrs TEXT,
--   src TEXT,
--   macro_backtrace_len INTEGER,
--   macro_backtrace TEXT,
--   callees TEXT
-- ) STRICT;
SELECT
  file,
  name,
  hash,
  hash_direct,
  inst_kind,
  proof_kind,
  callees
FROM
  db
WHERE
  proof_kind IS NOT NULL
LIMIT
  10;

SELECT
  count()
FROM
  db;

-- DROP TABLE db;
-- VACUUM;
