use sqlx::{Error, FromRow, SqlitePool};

pub type Environment = DbEnvironment;

impl Environment {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

#[derive(Debug, PartialEq, Eq, FromRow, Clone)]
pub struct DbEnvironment {
    pub name: String,
}

impl DbEnvironment {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query("INSERT INTO environments (name) VALUES (?)")
            .bind(self.name.as_str())
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl<T: Into<String>> From<T> for DbEnvironment {
    fn from(s: T) -> Self {
        Self { name: s.into() }
    }
}

impl AsRef<str> for DbEnvironment {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
