use super::variable::VarString;
use reqwest::Method;
use sqlx::{Error, FromRow, SqlitePool};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, FromRow, PartialEq)]
/// Database representation of a Request
pub struct DbRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: Vec<u8>,
}

impl DbRequest {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO requests
                (name, method, url, headers, body)
                VALUES (?, ?, ?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.method.as_str())
        .bind(self.url.as_str())
        .bind(self.headers.as_str())
        .bind(&self.body)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl<'a> From<Request<'a>> for DbRequest {
    fn from(req: Request<'a>) -> Self {
        // TODO: headers and body
        Self {
            name: req.name.into(),
            method: req.method.as_str().to_string(),
            url: req.url.to_string(),
            headers: String::new(),
            body: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    /// Name of the request
    pub name: String,
    /// HTTP method type
    pub method: Method,
    /// HTTP url string including protocol and parameters
    pub url: VarString,
    /// HTTP header key-value pairs
    pub headers: Vec<(String, VarString)>,
    /// HTTP request body
    pub body: Option<RequestBody<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestBody<'a> {
    /// A blob of bytes
    Blob(&'a [u8]),
    /// A body that contains a variable string
    Payload(VarString),
}

impl<'a> Request<'a> {
    pub fn new<N, M, U>(name: N, method: M, url: U) -> Result<Self, ()>
    where
        N: Into<String>,
        M: TryInto<Method>,
        U: Into<VarString>,
    {
        // TODO: headers and body
        Ok(Self {
            name: name.into(),
            method: method.try_into().map_err(|_| ())?,
            url: url.into(),
            headers: vec![],
            body: None,
        })
    }
}

impl<'a> TryFrom<DbRequest> for Request<'a> {
    type Error = ();
    fn try_from(req: DbRequest) -> Result<Self, Self::Error> {
        // TODO: headers and body
        Ok(Self {
            name: req.name.into(),
            method: req.method.parse().map_err(|_| ())?,
            url: req.url.into(),
            headers: vec![],
            body: None,
        })
    }
}
