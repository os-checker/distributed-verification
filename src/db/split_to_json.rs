use crate::{
    Result,
    db::{DbFunction, sql::SQL_SELECT_ALL},
};
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use std::{fs, path::PathBuf};

pub fn split_to_json(db_file: &str, base: &str) -> Result<()> {
    let _span = debug_span!("split_to_json", db_file).entered();
    let db = Connection::open(db_file)?;

    let mut stmt = db.prepare(SQL_SELECT_ALL)?;
    let rows = stmt.query_map([], |row| {
        Ok(DbFunction {
            file: row.get(0)?,
            name: row.get(1)?,
            hash: row.get(2)?,
            hash_direct: row.get(3)?,
            inst_kind: row.get(4).ok().map(convert),
            proof_kind: row.get(5).ok().map(convert),
            attrs: row.get(6).map(convert)?,
            src: row.get(7)?,
            macro_backtrace_len: row.get(8)?,
            macro_backtrace: row.get(9).map(convert)?,
            callees_len: row.get(10)?,
            callees: row.get(11).map(convert)?,
        })
    })?;

    let mut path_buf = PathBuf::with_capacity(128);

    for row in rows {
        let row = row?;
        path_buf.push(base);
        row.json_path(&mut path_buf);
        let _json = error_span!("write json", path = %path_buf.display()).entered();

        let file = fs::File::create(&path_buf)?;
        serde_json::to_writer_pretty(file, &row)?;
        path_buf.clear();
    }

    Ok(())
}

/// Types that don't implement FromSql have to be converted from JSON string.
fn convert<T: DeserializeOwned>(s: String) -> T {
    serde_json::from_str(&s).unwrap()
}
