use std::convert::Infallible;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DbError: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("JSONError: {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("InvalidBodyKind: {0}")]
    InvalidBodyKind(String),

    #[error("MissingBodyKind")]
    MissingBodyKind,

    #[error("UTF-8 Error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("InvalidMethod: {0}")]
    InvalidMethod(#[from] http::method::InvalidMethod),

    #[error("ConfigError: Non-UTF-8 data directory found")]
    ConfigDataToStr,

    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),

    #[error("ParseError: {0}")]
    ParseError(&'static str),

    #[error("Infallible")]
    Infallible,
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        Self::Infallible
    }
}
