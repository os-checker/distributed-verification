use super::Functions;
use crate::Result;
use distributed_verification::db::{
    DbFunction,
    sql::{SQL_CREATE, SQL_INSERT, db_file},
};
use eyre::{Context, ContextCompat};
use rusqlite::{Connection, named_params};
use serde_json::to_string_pretty;

pub struct Db {
    db: Connection,
}

impl Db {
    /// Create a timestamp.sqlite3 file and db table.
    pub fn new() -> Result<Db> {
        let db_file = db_file();
        info!(db_file, "start sqlite db");
        let _span = error_span!("Db::new", db_file).entered();
        let db = Connection::open(db_file).context("Failed to open or create db file.")?;

        db.execute(SQL_CREATE, []).context("Failed to execute SQL_CREATE.")?;

        Ok(Db { db })
    }

    /// This function should be called after recursive hashes are computed for all functions.
    pub fn store(&mut self, map: &Functions, crate_name: &str) -> Result<()> {
        info!(db = ?self.db.path(), "data ready to be stored to sqlite db");
        let tx = self.db.transaction()?;
        let mut stmt = tx.prepare(SQL_INSERT)?;

        let mut count = 0usize;
        for func in cache_to_db_func(map, crate_name) {
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
                ":macro_backtrace_len": func.macro_backtrace_len,
                ":macro_backtrace": to_string_pretty(&func.macro_backtrace).unwrap(),
                ":callees_len": func.callees_len,
                ":callees": to_string_pretty(&func.callees).unwrap(),
                ":crate": &func.krate,
            };

            if let Err(err) = stmt.insert(params) {
                match &err {
                    rusqlite::Error::SqliteFailure(error, opt_str) => {
                        // skip if the same hash exists
                        // FIXME: need to figure out hash collision
                        if matches!(error.code, rusqlite::ffi::ErrorCode::ConstraintViolation)
                            && error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                        {
                            error!(%error, ?opt_str, func.file, func.name, func.hash, "No insert a duplicated PRIMARY KEY.");
                        } else {
                            bail!("Failed to insert {func:?}\nerr={err:?}")
                        }
                    }
                    _ => bail!("Failed to insert {func:?}\nerr={err:?}"),
                }
            }
            count += 1;
        }
        stmt.finalize().context("Faield to commit prepare statement.")?;
        tx.commit()?;

        info!(db = ?self.db.path(), count, "data successully stored to sqlite db");
        Ok(())
    }
}

fn cache_to_db_func(map: &Functions, crate_name: &str) -> impl Iterator<Item = Result<DbFunction>> {
    map.iter().map(|(inst, func)| {
        let _span = error_span!("cache_to_db_func", ?inst, ?func).entered();
        // skip func that has no body
        let f = func.inner.as_ref().context("The func has no body.")?;
        // skip func that has no recursive_hash
        let hash = func.recursive_hash.clone().context("No recursive_hash.")?;
        let src = &func.src;
        let callees: Vec<_> = func
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
            .collect();
        Ok(DbFunction {
            file: f.file.clone().into(),
            name: f.name.clone().into(),
            hash,
            hash_direct: f.hash.clone(),
            inst_kind: src.inst_kind,
            proof_kind: f.proof_kind,
            attrs: src.attrs.clone(),
            src: src.src.as_str().into(),
            macro_backtrace_len: src.macro_backtrace_len,
            macro_backtrace: src.macro_backtrace.clone(),
            callees_len: callees.len(),
            callees,
            krate: crate_name.into(),
        })
    })
}
