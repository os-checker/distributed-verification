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
--   path TEXT
-- ) STRICT;
SELECT
  crate,
  name,
  path
FROM
  db
WHERE
  name LIKE 'convert::num::<impl convert::From<num::nonzero::NonZero<i32>> for num::nonzero::NonZero<i128>>::from'
  OR name='<ascii::ascii_char::AsciiChar as iter::range::Step>::backward_unchecked'
  OR name='<str::pattern::MatchOnly as str::pattern::TwoWayStrategy>::matching'
LIMIT
  10;

SELECT
  '_______________________________________' AS _;

SELECT
  crate,
  count() AS `Functions`,
  COUNT(
    CASE
      WHEN proof_kind IS NOT NULL THEN 1
    END
  ) AS `Proofs`
FROM
  db
GROUP BY
  crate;

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
