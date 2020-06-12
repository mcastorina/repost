use crate::Repl;
use crate::db::{DbError};

pub enum CmdError {
    DbError(DbError),
    ArgsError(String),
    NotFound,
    NotImplemented,
}

pub trait Cmd {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError>;
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CmdError::DbError(x) => match x {
                DbError::Rusqlite(x) => write!(f, "{}", x),
            },
            CmdError::ArgsError(x) => write!(f, "{}", x),
            CmdError::NotFound => write!(f, "Command not found."),
            CmdError::NotImplemented => write!(f, "Command not implemented."),
        }
    }
}
impl From<CmdError> for String {
    fn from(err: CmdError) -> String {
        match err {
            CmdError::DbError(x) => match x {
                DbError::Rusqlite(x) => format!("{}", x),
            },
            CmdError::ArgsError(x) => x,
            CmdError::NotFound => String::from("Command not found."),
            CmdError::NotImplemented => String::from("Command not implemented."),
        }
    }
}
impl From<DbError> for CmdError {
    fn from(err: DbError) -> CmdError {
        CmdError::DbError(err)
    }
}
