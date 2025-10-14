WITH
  cut AS (
    SELECT
      proof_kind,
      -- Should we handle function names leading with `<` or some monomorphized forms?
      -- cc https://github.com/os-checker/distributed-verification/issues/129
      crate||'::'||CASE
        WHEN instr (name, '::')>0 THEN substr (name, 1, instr (name, '::') - 1)
        ELSE name
      END AS mod
    FROM
      db
  ),
  df AS (
    SELECT
      mod,
      proof_kind,
      COUNT(*) AS cnt,
      ROUND(
        100.0*COUNT(*)/SUM(COUNT(*)) OVER (
          PARTITION BY
            mod
        ),
        1
      ) AS percent
    FROM
      cut
    GROUP BY
      mod,
      proof_kind
    ORDER BY
      mod,
      percent DESC
  )
SELECT
  mod,
  proof_kind,
  cnt,
  CAST(percent AS TEXT) AS pct
FROM
  df
WHERE
  NOT (
    proof_kind IS NULL
    AND percent==100.0
  );
