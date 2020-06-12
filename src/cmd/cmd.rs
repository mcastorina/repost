use crate::db::DbError;
use crate::Repl;

pub enum CmdError {
    DbError(DbError),
    ArgsError(String),
    ArgParseError(clap_v3::Error),
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
            CmdError::ArgParseError(x) => write!(f, "{}", x),
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
            CmdError::ArgParseError(x) => format!("{}", x),
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
impl From<clap_v3::Error> for CmdError {
    fn from(err: clap_v3::Error) -> CmdError {
        CmdError::ArgParseError(err)
    }
}
