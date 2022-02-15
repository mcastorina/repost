pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DbError: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("ClapError: {0}")]
    ClapError(#[from] clap::Error),

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
}
