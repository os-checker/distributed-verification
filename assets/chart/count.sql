WITH
  cut AS (
    SELECT
      proof_kind,
      -- Should we handle function names leading with `<` or some monomorphized forms?
      -- cc https://github.com/os-checker/distributed-verification/issues/129
      CASE
        WHEN instr (name, '::')=0 THEN crate
        ELSE crate||'::'||substr (name, 1, instr (name, '::') - 1)
      END AS mod
    FROM
      db
    WHERE
      instr (name, '<')=0
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
        0
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
  CAST(percent AS INTEGER) AS pct
FROM
  df
