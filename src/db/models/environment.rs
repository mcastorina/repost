use crate::cmd::models as cmd;
use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow, Clone)]
pub struct Environment {
    name: String,
}

impl Environment {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query("INSERT INTO environments (name) VALUES (?)")
            .bind(self.name.as_str())
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl<T: Into<String>> From<T> for Environment {
    fn from(s: T) -> Self {
        Self { name: s.into() }
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> From<cmd::Environment<'a>> for Environment {
    fn from(env: cmd::Environment<'a>) -> Self {
        Self {
            name: env.name.into(),
        }
    }
}
