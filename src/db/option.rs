use crate::error::Result;
use super::PrintableTable;
use super::{Db, db::DbObject};
use rusqlite::{Connection, NO_PARAMS, params};
use comfy_table::Cell;
use chrono::Utc;

pub struct InputOption {
    request_name: String,
    option_name: String,
    value: Option<String>,
}

impl InputOption {
    pub fn new(req_name: &str, opt_name: &str, value: Option<String>) -> InputOption {
        InputOption {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            value,
        }
    }
    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS input_options (
                  request_name    TEXT NOT NULL,
                  option_name     TEXT NOT NULL,
                  value           TEXT,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        self.option_name.as_ref()
    }
}

impl DbObject for InputOption {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO input_options (request_name, option_name, value)
                  VALUES (?1, ?2, ?3);",
            params![self.request_name, self.option_name, self.value],
        )?;
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "DELETE FROM input_options
                WHERE request_name = ?1 AND option_name = ?2;",
            params![self.request_name, self.option_name],
        )?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        let num = conn.execute(
            "UPDATE input_options SET
                value = ?1
            WHERE request_name = ?2 AND option_name = ?3;",
            params![self.value, self.request_name, self.option_name],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<InputOption>> {
        let mut stmt = conn
            .prepare("SELECT request_name, option_name, value FROM input_options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(InputOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                value: row.get(2)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    fn name(&self) -> Option<&str> {
        Some(self.name())
    }
}

impl PrintableTable for InputOption {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("request_name"),
            Cell::new("option_name"),
            Cell::new("value"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(&self.request_name),
            Cell::new(&self.option_name),
            Cell::new(self.value.as_ref().unwrap_or(&String::from(""))),
        ]]
    }
}

pub struct OutputOption {
    request_name: String,
    option_name: String,
    // TODO: enum
    extraction_type: String,
    extraction_path: String,
}

impl OutputOption {
    pub fn new(req_name: &str, opt_name: &str, typ: &str, path: &str) -> OutputOption {
        OutputOption {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            extraction_type: String::from(typ),
            extraction_path: String::from(path),
        }
    }
    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS output_options (
                  request_name      TEXT NOT NULL,
                  option_name       TEXT NOT NULL,
                  extraction_type   TEXT NOT NULL,
                  extraction_path   TEXT NOT NULL,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        self.option_name.as_ref()
    }
}

impl DbObject for OutputOption {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO output_options
                (request_name, option_name, extraction_type, extraction_path)
                VALUES (?1, ?2, ?3, ?4);",
            params![
                self.request_name,
                self.option_name,
                self.extraction_type,
                self.extraction_path,
            ],
        )?;
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        conn
            .execute("DELETE FROM output_options
                WHERE request_name = ?1 AND option_name = ?2;",
                params![self.request_name, self.option_name])?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        let num = conn.execute(
            "UPDATE output_options SET
                extraction_type = ?1, extraction_path = ?2
            WHERE request_name = ?3 AND option_name = ?4;",
            params![
                self.extraction_type,
                self.extraction_path,
                self.request_name,
                self.option_name
            ],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<OutputOption>> {
        let mut stmt = conn
            .prepare("SELECT
                request_name, option_name, extraction_type, extraction_path
            FROM output_options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(OutputOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                extraction_type: row.get(2)?,
                extraction_path: row.get(3)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    fn name(&self) -> Option<&str> {
        Some(self.name())
    }
}

impl PrintableTable for OutputOption {
    fn get_header() -> Vec<Cell> {
        vec![
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
        ]]
    }
}

