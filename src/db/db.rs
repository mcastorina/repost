use crate::error::Result;
use rusqlite::{Connection, NO_PARAMS};
use super::{Request, Variable, InputOption, OutputOption};
use comfy_table::Cell;

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

pub trait PrintableTable {
    fn get_header() -> Vec<Cell>;
    fn get_rows(&self) -> Vec<Vec<Cell>>;
}
impl <T: PrintableTable>PrintableTable for Vec<T> {
    fn get_header() -> Vec<Cell> {
        T::get_header()
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        self.iter().map(|x| x.get_rows().concat()).collect::<Vec<Vec<Cell>>>()
    }
}
impl PrintableTable for String {
    fn get_header() -> Vec<Cell> {
        vec![Cell::new("string")]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![Cell::new(self)]]
    }
}

pub trait DbObject {
    fn create(&self, conn: &Connection) -> Result<()>;
    fn delete(&self, conn: &Connection) -> Result<()>;
    fn update(&self, conn: &Connection) -> Result<usize>;
    fn get_all(conn: &Connection) -> Result<Vec<Self>>
        where Self: std::marker::Sized;
    fn name(&self) -> Option<&str> {
        None
    }
    fn get_by_name(conn: &Connection, name: &str) -> Result<Vec<Self>>
        where Self: std::marker::Sized {
        Ok(
            Self::get_all(conn)?
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
    fn delete_by_name(conn: &Connection, name: &str) -> Result<()>
        where Self: std::marker::Sized {
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
}
