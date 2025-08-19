WITH
  file_obj AS (
    SELECT
      *
    FROM
      db
    ORDER BY
      file,
      name
  )
SELECT
  file,
  JSON_GROUP_ARRAY (
    JSON_PATCH (
      '{}',
      JSON_OBJECT (
        'name',
        name,
        'hash',
        hash,
        'hash_direct',
        hash_direct,
        'inst_kind',
        inst_kind,
        'proof_kind',
        proof_kind,
        'attrs',
        JSON (attrs),
        'src',
        src,
        'macro_backtrace',
        JSON (macro_backtrace),
        'callees',
        JSON (callees)
      )
    )
  ) AS data
FROM
  file_obj
GROUP BY
  file;
