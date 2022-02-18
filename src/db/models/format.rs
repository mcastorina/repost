use comfy_table::Cell;
use std::fmt::{self, Display, Formatter};

pub const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub trait DisplayTable: Sized {
    const HEADER: &'static [&'static str];

    fn fmt(&self) -> Vec<Cell>;
    fn as_table(&self) -> Wrapper<Self> {
        todo!()
    }
}

// let r = Request::new();
// r.as_table() ???

pub struct Wrapper<T: DisplayTable>(T);
// T is Request, Environment, or Variable

impl<T: DisplayTable> Display for Wrapper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "lol")
    }
}

impl DisplayTable for Vec<super::Request> {
    const HEADER: &'static [&'static str] = &[];

    fn fmt(&self) -> Vec<Cell> {
        todo!()
    }
}

// impl DisplayTable for Request {}
// usage: Vec<Request>
// create the table for each Request in the vector

// possible solution: use a generic wrapper struct

// impl<T: DisplayTable + ?Sized> Display for T {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
//         todo!()
//     }
// }
