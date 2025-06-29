SELECT
  name,
  hash
FROM
  db
WHERE
  callees_len==1
ORDER BY
  name;
