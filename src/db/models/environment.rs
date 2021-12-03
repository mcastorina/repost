use sqlx::database::HasValueRef;
use sqlx::{ColumnIndex, Database, Decode, Error, FromRow, Row, Sqlite, SqlitePool, Type};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct Environment<'a> {
    pub name: Cow<'a, str>,
}

impl<'a> Environment<'a> {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self { name: name.into() }
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query("INSERT INTO environments (name) VALUES (?)")
            .bind(self.name.as_ref())
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl<'r, R> FromRow<'r, R> for Environment<'_>
where
    R: Row,
    usize: ColumnIndex<R>,
    String: Decode<'r, <R as Row>::Database> + Type<<R as Row>::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, Error> {
        let name: String = row.try_get(0)?;
        Ok(Environment::new(name))
    }
}
