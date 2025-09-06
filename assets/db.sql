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
--   callees_len INTEGER,
--   callees TEXT,
--   crate TEXT
-- ) STRICT;
SELECT
  crate,
  file,
  name,
  hash,
  hash_direct,
  proof_kind,
  callees_len
FROM
  db
WHERE
  proof_kind IS NOT NULL
LIMIT
  10;

SELECT
  count() AS `Total Proofs`
FROM
  db
WHERE
  proof_kind IS NOT NULL;

SELECT
  count() AS `Total Functions`
FROM
  db;

-- DROP TABLE db;
-- VACUUM;
