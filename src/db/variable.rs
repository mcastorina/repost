use super::DbObject;
use super::PrintableTableStruct;
use crate::error::Result;
use chrono::Utc;
use comfy_table::Cell;
use rusqlite::{params, Connection, NO_PARAMS};

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
        let value = value.map(|x| String::from(x));
        let source = source.map(|x| String::from(x));
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
                  timestamp       TEXT
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
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
    pub fn set_value(&mut self, value: Option<&str>) {
        self.value = value.map(|x| String::from(x));
    }
    pub fn set_source(&mut self, source: Option<&str>) {
        self.source = source.map(|x| String::from(x));
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
        conn.execute(
            "DELETE FROM variables WHERE rowid = ?1;",
            params![self.rowid],
        )?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        // first try by rowid
        let mut num = conn.execute(
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
        // then try by name, environment
        if num == 0 {
            num = conn.execute(
                "UPDATE variables SET
                    value = ?3,
                    source = ?4,
                    timestamp = ?5
                WHERE name = ?1 AND environment = ?2;",
                params![
                    self.name,
                    self.environment,
                    self.value,
                    self.source,
                    format!("{}", Utc::now().format("%Y-%m-%d %T %Z")),
                ],
            )?;
        }
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Variable>> {
        let mut stmt = conn.prepare(
            "SELECT rowid, name, environment, value, source, timestamp
                FROM variables ORDER BY timestamp ASC;",
        )?;

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
    fn name(&self) -> &str {
        self.name()
    }
}

impl PrintableTableStruct for Variable {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("id"),
            Cell::new("name"),
            Cell::new("environment"),
            Cell::new("value"),
            Cell::new("source"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(self.rowid),
            Cell::new(self.name()),
            Cell::new(self.environment()),
            Cell::new(self.value.as_ref().unwrap_or(&String::from(""))),
            Cell::new(self.source.as_ref().unwrap_or(&String::from(""))),
        ]]
    }
}
