use crate::error::Result;
use rusqlite::{Connection, NO_PARAMS};

pub struct Db {
    conn: Connection,
}
impl Db {
    pub fn new(path: &str) -> Result<Db> {
        let db = Db {
            conn: Connection::open(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                  name            TEXT PRIMARY KEY,
                  method          TEXT NOT NULL,
                  url             TEXT NOT NULL,
                  headers         TEXT,
                  body            BLOB
              )",
            NO_PARAMS,
        )?;

        // TODO: multiple of the same variable name / environment
        self.conn.execute(
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

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS input_options (
                  request_name    TEXT NOT NULL,
                  option_name     TEXT NOT NULL,
                  value           TEXT,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS output_options (
                  request_name      TEXT NOT NULL,
                  option_name       TEXT NOT NULL,
                  extraction_source TEXT NOT NULL,
                  extraction_path   TEXT NOT NULL,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;

        self.conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;

        Ok(())
    }
}
