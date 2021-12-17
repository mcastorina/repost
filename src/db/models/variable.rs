use super::Environment;
use chrono::{DateTime, Local};
use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Variable {
    id: usize,
    name: String,
    env: Environment,
    value: Option<String>,
    source: String,
    timestamp: DateTime<Local>,
}

impl Variable {
    pub fn new<N, E, V, S>(name: N, env: E, value: V, source: S) -> Self
    where
        N: Into<String>,
        E: Into<Environment>,
        V: Into<String>,
        S: Into<String>,
    {
        Self {
            id: 0,
            name: name.into(),
            env: env.into(),
            value: Some(value.into()),
            source: source.into(),
            timestamp: Local::now(),
        }
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO variables
                (name, env, value, source, timestamp)
                VALUES (?, ?, ?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.env.as_ref())
        .bind(self.value.as_ref())
        .bind(self.source.as_str())
        .bind(self.timestamp)
        .execute(pool)
        .await?;
        Ok(())
    }
}
