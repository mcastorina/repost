use super::DbObject;
use super::PrintableTableStruct;
use crate::error::Result;
use comfy_table::Cell;
use rusqlite::{params, Connection, NO_PARAMS};

#[derive(Debug, Clone)]
pub struct InputOption {
    request_name: String,
    option_name: String,
    values: Vec<String>,
}

impl InputOption {
    const VALUE_SEPARATOR: &'static str = "\n~\n";

    pub fn new(req_name: &str, opt_name: &str, values: Vec<String>) -> InputOption {
        InputOption {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            values,
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

    pub fn option_name(&self) -> &str {
        self.option_name.as_ref()
    }
    pub fn request_name(&self) -> &str {
        self.request_name.as_ref()
    }
    pub fn values(&self) -> Vec<&str> {
        self.values.iter().map(AsRef::as_ref).collect()
    }
    pub fn set_value(&mut self, value: Option<&str>) {
        self.values = match value {
            Some(x) => vec![String::from(x)],
            None => vec![],
        }
    }
    pub fn set_values(&mut self, values: Vec<&str>) {
        self.values = values.into_iter().map(String::from).collect();
    }

    fn stringify_values(values: Vec<&str>) -> Option<String> {
        match values.len() {
            0 => None,
            _ => Some(values.join(InputOption::VALUE_SEPARATOR)),
        }
    }
    fn unstringify_values(value: Option<String>) -> Vec<String> {
        match value {
            Some(value) => value
                .split(InputOption::VALUE_SEPARATOR)
                .map(String::from)
                .collect(),
            None => vec![],
        }
    }
}

impl DbObject for InputOption {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO input_options (request_name, option_name, value)
                  VALUES (?1, ?2, ?3);",
            params![
                self.request_name,
                self.option_name,
                InputOption::stringify_values(self.values())
            ],
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
            params![
                InputOption::stringify_values(self.values()),
                self.request_name,
                self.option_name
            ],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<InputOption>> {
        let mut stmt =
            conn.prepare("SELECT request_name, option_name, value FROM input_options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(InputOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                values: InputOption::unstringify_values(row.get(2)?),
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    fn name(&self) -> &str {
        self.request_name()
    }
}

impl PrintableTableStruct for InputOption {
    fn get_header() -> Vec<Cell> {
        vec![Cell::new("option_name"), Cell::new("values")]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(&self.option_name),
            Cell::new(self.values().join("\n")),
        ]]
    }
}

#[derive(Debug, Clone)]
pub struct OutputOption {
    request_name: String,
    option_name: String,
    // TODO: enum
    extraction_type: String,
    extraction_source: String,
}

impl OutputOption {
    pub fn new(req_name: &str, opt_name: &str, typ: &str, path: &str) -> OutputOption {
        OutputOption {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            extraction_type: String::from(typ),
            extraction_source: String::from(path),
        }
    }
    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS output_options (
                  request_name      TEXT NOT NULL,
                  option_name       TEXT NOT NULL,
                  extraction_type   TEXT NOT NULL,
                  extraction_source TEXT NOT NULL,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;
        Ok(())
    }

    pub fn option_name(&self) -> &str {
        self.option_name.as_ref()
    }
    pub fn request_name(&self) -> &str {
        self.request_name.as_ref()
    }
    pub fn extraction_type(&self) -> &str {
        self.extraction_type.as_ref()
    }
    pub fn extraction_source(&self) -> &str {
        self.extraction_source.as_ref()
    }
}

impl DbObject for OutputOption {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO output_options
                (request_name, option_name, extraction_type, extraction_source)
                VALUES (?1, ?2, ?3, ?4);",
            params![
                self.request_name,
                self.option_name,
                self.extraction_type,
                self.extraction_source,
            ],
        )?;
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "DELETE FROM output_options
                WHERE request_name = ?1 AND option_name = ?2;",
            params![self.request_name, self.option_name],
        )?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        let num = conn.execute(
            "UPDATE output_options SET
                extraction_type = ?1, extraction_source = ?2
            WHERE request_name = ?3 AND option_name = ?4;",
            params![
                self.extraction_type,
                self.extraction_source,
                self.request_name,
                self.option_name
            ],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<OutputOption>> {
        let mut stmt = conn.prepare(
            "SELECT
                request_name, option_name, extraction_type, extraction_source
            FROM output_options;",
        )?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(OutputOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                extraction_type: row.get(2)?,
                extraction_source: row.get(3)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    fn name(&self) -> &str {
        self.request_name()
    }
}

impl PrintableTableStruct for OutputOption {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("request_name"),
            Cell::new("output_variable"),
            Cell::new("extraction_type"),
            Cell::new("extraction_source"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(&self.request_name),
            Cell::new(&self.option_name),
            Cell::new(&self.extraction_type),
            Cell::new(&self.extraction_source),
        ]]
    }
}
