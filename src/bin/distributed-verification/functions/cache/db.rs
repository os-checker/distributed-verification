use super::Functions;
use crate::Result;
use distributed_verification::db::DbFunction;
use eyre::{Context, ContextCompat};
use rusqlite::{Connection, named_params};
use serde_json::to_string_pretty;

const FILE_NAME: &str = "db.sqlite3";
const SQL_DROP: &str = "DROP TABLE IF EXISTS db;";
// cannot VACUUM from within a transaction
const SQL_VACUUM: &str = "VACUUM;";
const SQL_CREATE: &str = "\
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
  callees TEXT
) STRICT;
";
const SQL_INSERT: &str = "\
INSERT INTO db (file, name, hash, hash_direct, inst_kind, proof_kind, attrs, src, macro_backtrace_len, macro_backtrace, callees) 
VALUES (:file, :name, :hash, :hash_direct, :inst_kind, :proof_kind, :attrs, :src, :macro_backtrace_len, :macro_backtrace, :callees)
";

pub struct Db {
    db: Connection,
}

impl Db {
    /// Create a timestamp.sqlite3 file and db table.
    pub fn new() -> Result<Db> {
        // let now = jiff::Timestamp::now();
        let _span = error_span!("Db::new", FILE_NAME).entered();
        let db = Connection::open(FILE_NAME).context("Failed to open or create db file.")?;

        db.execute(SQL_DROP, []).context("Failed to drop db.")?;
        db.execute(SQL_VACUUM, []).context("Failed to VACUUM.")?;
        db.execute(SQL_CREATE, []).context("Failed to execute SQL_CREATE.")?;

        Ok(Db { db })
    }

    /// This function should be called after recursive hashes are computed for all functions.
    pub fn store(&self, map: &Functions) -> Result<()> {
        let mut stmt = self.db.prepare(SQL_INSERT)?;
        for func in cache_to_db_func(map) {
            let func = match func {
                Ok(func) => func,
                Err(err) => {
                    error!(?err);
                    continue;
                }
            };

            let params = named_params! {
                ":file": &func.file,
                ":name": &func.name,
                ":hash": &func.hash,
                ":hash_direct": &func.hash_direct,
                ":inst_kind":  func.inst_kind.map(|k| to_string_pretty(&k).unwrap()),
                ":proof_kind": func.proof_kind.map(|k| to_string_pretty(&k).unwrap()),
                ":attrs": to_string_pretty(&func.attrs).unwrap(),
                ":src": &func.src,
                ":macro_backtrace_len": &func.macro_backtrace_len,
                ":macro_backtrace": to_string_pretty(&func.macro_backtrace).unwrap(),
                ":callees": to_string_pretty(&func.callees).unwrap()
            };
            if let Err(err) = stmt.insert(params) {
                match &err {
                    rusqlite::Error::SqliteFailure(error, opt_str) => {
                        // skip if the same hash exists
                        // FIXME: need to figure out hash collision
                        if matches!(error.code, rusqlite::ffi::ErrorCode::ConstraintViolation)
                            && error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                        {
                            error!(%error, ?opt_str, ?func, "No insert a duplicated PRIMARY KEY.");
                        } else {
                            bail!("Failed to insert {func:?}\nerr={err:?}")
                        }
                    }
                    _ => bail!("Failed to insert {func:?}\nerr={err:?}"),
                }
            }
        }
        Ok(())
    }
}

fn cache_to_db_func(map: &Functions) -> impl Iterator<Item = Result<DbFunction>> {
    map.iter().map(|(inst, func)| {
        let _span = error_span!("cache_to_db_func", ?inst, ?func).entered();
        // skip func that has no body
        let f = func.inner.as_ref().context("The func has no body.")?;
        // skip func that has no recursive_hash
        let hash = func.recursive_hash.clone().context("No recursive_hash.")?;
        let src = &func.src;
        Ok(DbFunction {
            file: f.file.clone().into(),
            name: f.name.clone().into(),
            hash,
            hash_direct: f.hash.clone(),
            inst_kind: src.inst_kind,
            proof_kind: f.proof_kind,
            attrs: src.attrs.clone(),
            src: src.src.clone(),
            macro_backtrace_len: src.macro_backtrace_len,
            macro_backtrace: src.macro_backtrace.clone(),
            callees: func
                .callees
                .iter()
                .filter_map(|inst| match map.get(inst) {
                    Some(callee) => callee.recursive_hash.clone().or_else(|| {
                        error!(callee = ?inst, "The callee donesn't exist.");
                        None
                    }),
                    None => {
                        error!(callee = ?inst, "The callee donesn't exist.");
                        None
                    }
                })
                .collect(),
        })
    })
}
