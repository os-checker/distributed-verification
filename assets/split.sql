SELECT
  file,
  hash,
  JSON_PATCH (
    '{}',
    JSON_OBJECT (
      'file',
      file,
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
  ) AS data
FROM
  db
ORDER BY
  file,
  hash;
