pub type Result<T> = std::result::Result<T, Error>;
pub struct Error {
    // message: String,
    kind: ErrorKind,
}
pub enum ErrorKind {
    DbError(rusqlite::Error),
    ClapError(clap_v3::Error),
    NotFound,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::DbError(x) => write!(f, "{}", x),
            ErrorKind::ClapError(x) => write!(f, "{}", x),
            ErrorKind::NotFound => write!(f, "Not found"),
        }
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::DbError(x) => write!(f, "DbError({})", x),
            ErrorKind::ClapError(x) => write!(f, "DbError({})", x),
            ErrorKind::NotFound => write!(f, "Not found"),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error {
            kind: ErrorKind::DbError(err),
        }
    }
}
impl From<clap_v3::Error> for Error {
    fn from(err: clap_v3::Error) -> Error {
        Error {
            kind: ErrorKind::ClapError(err),
        }
    }
}
