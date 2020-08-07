use super::DbObject;
use super::PrintableTableStruct;
use crate::error::Result;
use comfy_table::Cell;
use rusqlite::{Connection, NO_PARAMS};

pub struct Environment {
    name: String,
}

impl Environment {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn exists(conn: &Connection, env: &str) -> Result<bool> {
        todo!();
    }
    pub fn get_all(conn: &Connection) -> Result<Vec<Environment>> {
        let mut stmt = conn.prepare("SELECT DISTINCT environment FROM variables;")?;
        let envs = stmt.query_map(NO_PARAMS, |row| Ok(Environment { name: row.get(0)? }))?;

        Ok(envs.filter_map(|env| env.ok()).collect())
    }
}

impl PrintableTableStruct for Environment {
    fn get_header() -> Vec<Cell> {
        vec![Cell::new("environment")]
    }
    fn get_row(&self) -> Vec<Cell> {
        vec![Cell::new(&self.name)]
    }
}
