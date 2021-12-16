use super::Environment;
use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Variable {
    id: usize,
    name: String,
    env: Environment,
    value: String,
    // TODO: source, timestamp
}

impl Variable {
    pub fn new<T, U, E>(name: T, env: E, value: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
        E: Into<Environment>,
    {
        Self {
            id: 0,
            name: name.into(),
            env: env.into(),
            value: value.into(),
        }
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO variables
                (name, env, value)
                VALUES (?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.env.as_ref())
        .bind(self.value.as_str())
        .execute(pool)
        .await?;
        Ok(())
    }
}
