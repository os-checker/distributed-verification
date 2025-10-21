WITH
  df_mod AS (
    SELECT
      crate,
      name,
      -- path must not contain generics (i.e. `<...>`), 
      -- and must start with a root.
      CASE
        WHEN path LIKE '%::%::%' -- 确保至少有 2 个 "::"
        THEN substr (
          path,
          1,
          instr (
            substr (path, instr (path, '::')+2), -- 跳过第一个 "::"
            '::'
          )+instr (path, '::')
        ) -- 加上之前跳过的长度，再回退 1
        ELSE path -- 不足 2 个 "::" 就原样返回
      END AS mod
    FROM
      db
    ORDER BY
      crate,
      name,
      mod
  ),
  obj_name_mod AS (
    SELECT
      crate,
      json_group_object (name, mod) AS name_mod
    FROM
      df_mod
    GROUP BY
      crate
  )
SELECT
  json_group_object (crate, json (name_mod)) AS name_mod
FROM
  obj_name_mod
