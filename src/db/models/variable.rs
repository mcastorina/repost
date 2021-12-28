use crate::cmd::models as cmd;
use chrono::{DateTime, Local};
use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, FromRow, PartialEq)]
pub struct Variable {
    pub id: i32,
    pub name: String,
    pub env: String,
    pub value: Option<String>,
    pub source: String,
    pub timestamp: DateTime<Local>,
}

impl Variable {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO variables
                (name, env, value, source, timestamp)
                VALUES (?, ?, ?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.env.as_str())
        .bind(self.value.as_ref())
        .bind(self.source.as_str())
        .bind(self.timestamp)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl<'a> From<cmd::Variable<'a>> for Variable {
    fn from(var: cmd::Variable<'a>) -> Self {
        Self {
            id: 0,
            name: var.name.into(),
            env: var.env.name.into(),
            value: var.value.map(|cow| cow.into()),
            source: var.source.into(),
            timestamp: var.timestamp,
        }
    }
}
