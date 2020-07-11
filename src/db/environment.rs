use crate::error::Result;
use super::PrintableTable;
use super::{Db, db::DbObject};
use rusqlite::{Connection, NO_PARAMS, params};
use comfy_table::Cell;
use chrono::Utc;

pub struct Environment {
    name: String,
}

impl Environment {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn get_all(conn: &Connection) -> Result<Vec<Environment>> {
        let mut stmt = conn
            .prepare("SELECT DISTINCT environment FROM variables;")?;

        let envs = stmt.query_map(NO_PARAMS, |row| {
            Ok(Environment {
                name: row.get(0)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(envs.filter_map(|env| env.ok()).collect())
    }
}

impl PrintableTable for Environment {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("environment"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(self.name()),
        ]]
    }
}
