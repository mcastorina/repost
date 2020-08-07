use super::{InputOption, OutputOption, Request, RequestResponse, Variable};
use crate::error::Result;
use comfy_table::Cell;
use rusqlite::{Connection, NO_PARAMS};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

pub struct Db {
    root: PathBuf,
    conn: Connection,
}
impl Db {
    pub fn new<P: AsRef<Path>>(root: P, path: &str) -> Result<Db> {
        let db = Db {
            root: root.as_ref().to_path_buf(),
            conn: Connection::open(root.as_ref().join(path))?,
        };
        db.create_tables()?;
        Ok(db)
    }
    pub fn set_db(&mut self, path: &str) -> Result<()> {
        self.conn = Connection::open(self.root.join(path))?;
        self.create_tables()?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn get_dbs(&self) -> Result<Vec<String>> {
        // TODO: option for config directory; set default to $XDG_CONFIG_DIR/repost
        let mut result = vec![];
        let paths = fs::read_dir(&self.root)?;
        for path in paths {
            let path = path?.path();
            // filter out .db extensions
            match path.extension() {
                Some(x) => {
                    if x != "db" {
                        continue;
                    }
                }
                _ => continue,
            }
            let ws = path.file_stem().unwrap();
            if let Some(x) = ws.to_str() {
                result.push(String::from(x));
            }
        }
        Ok(result)
    }

    fn create_tables(&self) -> Result<()> {
        Request::create_table(&self.conn)?;
        Variable::create_table(&self.conn)?;
        InputOption::create_table(&self.conn)?;
        OutputOption::create_table(&self.conn)?;
        RequestResponse::create_table(&self.conn)?;
        self.conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;

        Ok(())
    }
}

pub trait PrintableTableStruct {
    fn get_header() -> Vec<Cell>;
    fn get_row(&self) -> Vec<Cell>;
}
pub trait PrintableTable {
    fn get_header(&self) -> Vec<Cell>;
    fn get_rows(&self) -> Vec<Vec<Cell>>;
}
impl<T: PrintableTableStruct> PrintableTable for Vec<T> {
    fn get_header(&self) -> Vec<Cell> {
        T::get_header()
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        self.iter().map(T::get_row).collect()
    }
}
impl<T: PrintableTableStruct> PrintableTable for &Vec<T> {
    fn get_header(&self) -> Vec<Cell> {
        T::get_header()
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        self.iter().map(T::get_row).collect()
    }
}
impl PrintableTable for (String, Vec<String>) {
    fn get_header(&self) -> Vec<Cell> {
        vec![Cell::new(&self.0)]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        self.1.iter().map(|x| vec![Cell::new(x)]).collect()
    }
}

pub trait DbObject {
    fn create(&self, conn: &Connection) -> Result<()>;
    fn delete(&self, conn: &Connection) -> Result<()>;
    fn update(&self, conn: &Connection) -> Result<usize>;
    fn get_all(conn: &Connection) -> Result<Vec<Self>>
    where
        Self: std::marker::Sized;

    fn get_by<F>(conn: &Connection, f: F) -> Result<Vec<Self>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> bool,
    {
        Ok(Self::get_all(conn)?.into_iter().filter(f).collect())
    }
    fn upsert(&self, conn: &Connection) -> Result<()> {
        // default implementation: try to update, then try to create
        match self.update(conn) {
            Err(_) | Ok(0) => self.create(conn),
            _ => Ok(()),
        }
    }
    fn get_all_map<F, T>(conn: &Connection, f: F) -> Result<HashMap<T, Self>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> T,
        T: std::cmp::Eq + std::hash::Hash,
    {
        let v = Self::get_all(conn)?;
        let mut m = HashMap::new();
        for e in v.into_iter() {
            m.insert(f(&e), e);
        }
        Ok(m)
    }
    fn collect_all<F, T>(conn: &Connection, f: F) -> Result<Vec<T>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> T,
        T: std::cmp::Eq + std::hash::Hash,
    {
        Ok(
            HashSet::<T>::from_iter(Self::get_all(conn)?.into_iter().map(|x| f(&x)))
                .into_iter()
                .collect(),
        )
    }
    fn collect_by<F, T>(conn: &Connection, f: F) -> Result<Vec<T>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> Option<T>,
        T: std::cmp::Eq + std::hash::Hash,
    {
        Ok(
            HashSet::<T>::from_iter(Self::get_all(conn)?.into_iter().filter_map(|x| f(&x)))
                .into_iter()
                .collect(),
        )
    }
}
