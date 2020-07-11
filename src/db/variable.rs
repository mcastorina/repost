use crate::error::Result;
use super::PrintableTable;
use super::{Db, db::DbObject};
use rusqlite::{Connection, NO_PARAMS, params};
use comfy_table::Cell;
use chrono::Utc;

pub struct Variable {
    rowid: u32,
    name: String,
    environment: String,
    value: Option<String>,
    source: Option<String>,
    timestamp: Option<String>,
}

impl Variable {
    pub fn new(name: &str, env: &str, value: Option<&str>, source: Option<&str>) -> Variable {
        let value = match value {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        let source = match source {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        Variable {
            rowid: 0,
            name: String::from(name),
            environment: String::from(env),
            value,
            source,
            timestamp: None,
        }
    }
    pub fn create_table(conn: &Connection) -> Result<()> {
        // TODO: multiple of the same variable name / environment
        conn.execute(
            "CREATE TABLE IF NOT EXISTS variables (
                  rowid           INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  environment     TEXT NOT NULL,
                  value           TEXT,
                  source          TEXT,
                  timestamp       TEXT,
                  UNIQUE(name, environment)
              )",
            NO_PARAMS,
        )?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn environment(&self) -> &str {
        self.environment.as_ref()
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn consume_value(&mut self) -> Option<String> {
        self.value.take()
    }
}

impl DbObject for Variable {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO variables (name, environment, value, source, timestamp)
                  VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                self.name,
                self.environment,
                self.value,
                self.source,
                format!("{}", Utc::now().format("%Y-%m-%d %T %Z"))
            ],
        )?;
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM variables WHERE rowid = ?1;", params![self.rowid])?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        let num = conn.execute(
            "UPDATE variables SET
                name = ?1,
                environment = ?2,
                value = ?3,
                source = ?4,
                timestamp = ?5
            WHERE rowid = ?6;",
            params![
                self.name,
                self.environment,
                self.value,
                self.source,
                format!("{}", Utc::now().format("%Y-%m-%d %T %Z")),
                self.rowid,
            ],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Variable>> {
        let mut stmt = conn
            .prepare("SELECT rowid, name, environment, value, source, timestamp
                FROM variables ORDER BY timestamp ASC;")?;

        let vars = stmt.query_map(NO_PARAMS, |row| {
            Ok(Variable {
                rowid: row.get(0)?,
                name: row.get(1)?,
                environment: row.get(2)?,
                value: row.get(3)?,
                source: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(vars.filter_map(|var| var.ok()).collect())
    }
    fn name(&self) -> Option<&str> {
        Some(self.name())
    }
}

impl PrintableTable for Variable {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("name"),
            Cell::new("environment"),
            Cell::new("value"),
            Cell::new("source"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(self.name()),
            Cell::new(self.environment()),
            Cell::new(self.value.as_ref().unwrap_or(&String::from(""))),
            Cell::new(self.source.as_ref().unwrap_or(&String::from(""))),
        ]]
    }
}
