pub type Result<T> = std::result::Result<T, Error>;
pub struct Error {
    // message: String,
    kind: ErrorKind,
}
pub enum ErrorKind {
    DbError(rusqlite::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::DbError(x) => write!(f, "{}", x),
        }
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::DbError(x) => write!(f, "DbError({})", x),
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
