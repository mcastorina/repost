pub type Result<T> = std::result::Result<T, Error>;
pub struct Error {
    // message: String,
    kind: ErrorKind,
}
pub enum ErrorKind {
    DbError(rusqlite::Error),
    ClapError(clap_v3::Error),
    IOError(std::io::Error),
    ArgumentError(&'static str),
    MissingOption(String),
    ReqwestError(reqwest::Error),
    ParseError,
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
            ErrorKind::IOError(x) => write!(f, "{}", x),
            ErrorKind::ArgumentError(x) => write!(f, "{}", x),
            ErrorKind::MissingOption(x) => write!(f, "The following option is missing: {}", x),
            ErrorKind::ReqwestError(x) => write!(f, "{}", x),
            ErrorKind::NotFound => write!(f, "Not found."),
            ErrorKind::ParseError => write!(f, "Parse error."),
        }
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            ErrorKind::DbError(x) => write!(f, "DbError({})", x),
            ErrorKind::ClapError(x) => write!(f, "ClapError({})", x),
            ErrorKind::IOError(x) => write!(f, "IOError({})", x),
            ErrorKind::ArgumentError(x) => write!(f, "ArgumentError({})", x),
            ErrorKind::MissingOption(x) => write!(f, "MissingOption({})", x),
            ErrorKind::ReqwestError(x) => write!(f, "ReqwestError({})", x),
            ErrorKind::NotFound => write!(f, "Not found."),
            ErrorKind::ParseError => write!(f, "Parse error."),
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
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error {
            kind: ErrorKind::IOError(err),
        }
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error {
            kind: ErrorKind::ReqwestError(err),
        }
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error {
            kind: ErrorKind::ParseError,
        }
    }
}
impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error {
            kind: ErrorKind::ParseError,
        }
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error {
            kind: ErrorKind::ParseError,
        }
    }
}
