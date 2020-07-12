use super::PrintableTableStruct;
use super::{db::DbObject, Db};
use crate::error::Result;
use chrono::Utc;
use comfy_table::Cell;
use rusqlite::{params, Connection, NO_PARAMS};

pub struct Environment {
    name: String,
}

impl Environment {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl DbObject for Environment {
    fn create(&self, conn: &Connection) -> Result<()> {
        // TODO: return an error, but these should never be called
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        Ok(0)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Environment>> {
        let mut stmt = conn.prepare("SELECT DISTINCT environment FROM variables;")?;
        let envs = stmt.query_map(NO_PARAMS, |row| Ok(Environment { name: row.get(0)? }))?;

        // TODO: print a warning for errors
        Ok(envs.filter_map(|env| env.ok()).collect())
    }
    fn name(&self) -> Option<&str> {
        Some(self.name())
    }
}

impl PrintableTableStruct for Environment {
    fn get_header() -> Vec<Cell> {
        vec![Cell::new("environment")]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![Cell::new(&self.name)]]
    }
}
