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
}

impl DbObject for Environment {
    fn create(&self, _conn: &Connection) -> Result<()> {
        // TODO: return an error, but these should never be called
        Ok(())
    }
    fn delete(&self, _conn: &Connection) -> Result<()> {
        Ok(())
    }
    fn update(&self, _conn: &Connection) -> Result<usize> {
        Ok(0)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Environment>> {
        let mut stmt = conn.prepare("SELECT DISTINCT environment FROM variables;")?;
        let envs = stmt.query_map(NO_PARAMS, |row| Ok(Environment { name: row.get(0)? }))?;

        // TODO: print a warning for errors
        Ok(envs.filter_map(|env| env.ok()).collect())
    }
    fn name(&self) -> &str {
        self.name()
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
