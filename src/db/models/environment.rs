use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow, Clone)]
pub struct Environment {
    name: String,
}

impl Environment {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }

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
        Environment::new(s)
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
