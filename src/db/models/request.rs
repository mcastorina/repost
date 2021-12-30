use crate::cmd::models as cmd;
use chrono::{DateTime, Local};
use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, FromRow, PartialEq)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: Vec<u8>,
}

impl Request {
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

impl<'a> From<cmd::Request<'a>> for Request {
    fn from(req: cmd::Request<'a>) -> Self {
        // TODO: headers and body
        Self {
            name: req.name.into(),
            method: req.method.as_str().to_string(),
            url: req.url.into(),
            headers: String::new(),
            body: vec![],
        }
    }
}
