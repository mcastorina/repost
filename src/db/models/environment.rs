use sqlx::{Error, FromRow, SqlitePool};

#[derive(FromRow)]
pub struct Environment {
    pub name: String,
}

impl Environment {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query("INSERT INTO environments (name) VALUES (?)")
            .bind(&self.name)
            .execute(pool)
            .await?;
        Ok(())
    }
}
