use super::{InputOption, OutputOption, Request, Variable};
use crate::error::Result;
use comfy_table::Cell;
use rusqlite::{Connection, NO_PARAMS};
use std::collections::HashMap;

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

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
    fn create_tables(&self) -> Result<()> {
        Request::create_table(&self.conn)?;
        Variable::create_table(&self.conn)?;
        InputOption::create_table(&self.conn)?;
        OutputOption::create_table(&self.conn)?;
        self.conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;

        Ok(())
    }
}

pub trait PrintableTableStruct {
    fn get_header() -> Vec<Cell>;
    fn get_rows(&self) -> Vec<Vec<Cell>>;
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
        self.iter()
            .map(|x| x.get_rows().concat())
            .collect::<Vec<Vec<Cell>>>()
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
    fn name(&self) -> &str;

    fn get_by<F>(conn: &Connection, f: F) -> Result<Vec<Self>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> bool,
    {
        Ok(Self::get_all(conn)?.into_iter().filter(f).collect())
    }
    fn get_by_name(conn: &Connection, name: &str) -> Result<Vec<Self>>
    where
        Self: std::marker::Sized,
    {
        Self::get_by(conn, |x| x.name() == name)
    }
    fn delete_by_name(conn: &Connection, name: &str) -> Result<()>
    where
        Self: std::marker::Sized,
    {
        for x in Self::get_by_name(conn, name)? {
            x.delete(conn)?;
        }
        Ok(())
    }
    fn upsert(&self, conn: &Connection) -> Result<()> {
        // default implementation: try to update, then try to create
        match self.update(conn) {
            Err(_) | Ok(0) => self.create(conn),
            _ => Ok(()),
        }
    }
    fn exists(conn: &Connection, name: &str) -> Result<bool>
    where
        Self: std::marker::Sized,
    {
        // default implementation: get_by_name
        Ok(Self::get_by_name(conn, name)?.len() > 0)
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
    fn get_by_name_map<F, T>(conn: &Connection, name: &str, f: F) -> Result<HashMap<T, Self>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> T,
        T: std::cmp::Eq + std::hash::Hash,
    {
        let v = Self::get_by_name(conn, name)?;
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
    {
        // TODO: unique values
        Ok(Self::get_all(conn)?.into_iter().map(|x| f(&x)).collect())
    }
    fn collect_by<F, T>(conn: &Connection, f: F) -> Result<Vec<T>>
    where
        Self: std::marker::Sized,
        F: Fn(&Self) -> Option<T>,
    {
        // TODO: unique values
        Ok(Self::get_all(conn)?
            .into_iter()
            .filter_map(|x| f(&x))
            .collect())
    }
}