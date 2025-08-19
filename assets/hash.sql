-- sqlite3 ../artifacts/artifact-libcore/core.sqlite3 <hash.sql | jq -s '.' >../ui/verify-rust-std_data/hash.json
WITH
  df AS (
    SELECT
      file,
      name,
      proof_kind,
      hash
    FROM
      db
    ORDER BY
      file,
      name
  )
SELECT
  JSON_PATCH (
    '{}',
    JSON_OBJECT (
      'file',
      file,
      'name',
      name,
      'proof_kind',
      proof_kind,
      'hash',
      hash
    )
  )
FROM
  df;
