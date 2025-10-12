WITH
  cut AS (
    SELECT
      proof_kind,
      crate||'::'||CASE
        WHEN instr (name, '::')>0 THEN substr (name, 1, instr (name, '::') - 1)
        ELSE name
      END AS name_category
    FROM
      db
  ),
  a AS (
    SELECT
      name_category,
      proof_kind,
      COUNT(*) AS cnt,
      ROUND(
        100.0*COUNT(*)/SUM(COUNT(*)) OVER (
          PARTITION BY
            name_category
        ),
        1
      ) AS percent
    FROM
      cut
    GROUP BY
      name_category,
      proof_kind
    ORDER BY
      name_category,
      percent DESC
  )
SELECT
  name_category AS mod,
  proof_kind,
  cnt,
  CAST(percent AS TEXT) AS pct
FROM
  a
WHERE
  NOT (
    proof_kind IS NULL
    AND percent==100.0
  );
