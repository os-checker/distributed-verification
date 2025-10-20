WITH
  df AS (
    SELECT
      crate,
      name,
      proof_kind
    FROM
      db
    WHERE
      proof_kind IS NOT NULL
    ORDER BY
      crate,
      name,
      proof_kind
  ),
  g AS (
    SELECT
      crate,
      json_group_object (name, proof_kind) AS proof
    FROM
      df
    GROUP BY
      crate
  )
SELECT
  json_group_object (crate, json (proof)) AS result
FROM
  g
