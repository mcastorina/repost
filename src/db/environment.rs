use super::PrintableTableStruct;
use super::{db::DbObject, Db};
use crate::error::Result;
use chrono::Utc;
use comfy_table::Cell;
use rusqlite::{params, Connection, NO_PARAMS};

pub fn get_all(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT environment FROM variables;")?;

    let envs = stmt.query_map(NO_PARAMS, |row| Ok(row.get(0)?))?;

    // TODO: print a warning for errors
    Ok(envs.filter_map(|env| env.ok()).collect())
}
