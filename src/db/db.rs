use crate::error::Result;
use rusqlite::{Connection, NO_PARAMS};
use super::request::Request;

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
        Request::create_table(&self.conn)?;

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

pub trait PrintableTable {
    fn get_header() -> Vec<String>;
    fn get_rows(&self) -> Vec<Vec<String>>;
}
impl <T: PrintableTable>PrintableTable for Vec<T> {
    fn get_header() -> Vec<String> {
        T::get_header()
    }
    fn get_rows(&self) -> Vec<Vec<String>> {
        self.iter().map(|x| x.get_rows().concat()).collect::<Vec<Vec<String>>>()
    }
}
impl PrintableTable for String {
    fn get_header() -> Vec<String> {
        vec![String::from("string")]
    }
    fn get_rows(&self) -> Vec<Vec<String>> {
        vec![vec![self.clone()]]
    }
}

pub trait DbObject {
    fn create(&self, conn: &Connection) -> Result<()>;
    fn delete(&self, conn: &Connection) -> Result<()>;
    fn update(&self, conn: &Connection) -> Result<()>;
    fn get_all(&self, conn: &Connection) -> Result<Vec<Self>>
        where Self: std::marker::Sized;
    fn name(&self) -> Option<&str> {
        None
    }
    fn get_by_name(&self, conn: &Connection, name: &str) -> Result<Vec<Self>>
        where Self: std::marker::Sized {
        Ok(
            self.get_all(conn)?
            .into_iter()
            .filter(|x|
                match x.name() {
                    Some(x) => x == name,
                    None => false,
                }
             )
            .collect()
        )
    }
    fn delete_by_name(&self, conn: &Connection, name: &str) -> Result<()>
        where Self: std::marker::Sized {
        for x in self.get_by_name(conn, name)? {
            x.delete(conn)?;
        }
        Ok(())
    }
    fn upsert(&self, conn: &Connection) -> Result<()> {
        // default implementation: try to update, then try to create
        match self.update(conn) {
            Err(_) => self.create(conn),
            _ => Ok(()),
        }
    }
}
