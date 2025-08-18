pub const DB_FILE: &str = "db.sqlite3";
pub const SQL_DROP: &str = "DROP TABLE IF EXISTS db;";
// cannot VACUUM from within a transaction
pub const SQL_VACUUM: &str = "VACUUM;";
pub const SQL_CREATE: &str = "\
CREATE TABLE IF NOT EXISTS db (
  file TEXT NOT NULL,
  name TEXT NOT NULL,
  hash TEXT NOT NULL PRIMARY KEY,
  hash_direct TEXT NOT NULL,
  inst_kind TEXT,
  proof_kind TEXT,
  attrs TEXT,
  src TEXT,
  macro_backtrace_len INTEGER,
  macro_backtrace TEXT,
  callees_len INTEGER,
  callees TEXT
) STRICT;
";
pub const SQL_INSERT: &str = "\
INSERT INTO db (file, name, hash, hash_direct, inst_kind, proof_kind, attrs, src, macro_backtrace_len, macro_backtrace, callees_len, callees) 
VALUES (:file, :name, :hash, :hash_direct, :inst_kind, :proof_kind, :attrs, :src, :macro_backtrace_len, :macro_backtrace, :callees_len, :callees)
";

pub fn db_file() -> String {
    const VAR_DB_FILE: &str = "DB_FILE";
    let db_file = std::env::var(VAR_DB_FILE);
    db_file.unwrap_or_else(|_| DB_FILE.to_owned())
}
