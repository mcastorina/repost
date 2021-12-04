use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct Environment {
    pub name: String,
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
