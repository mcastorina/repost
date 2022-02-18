use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow, Clone)]
pub struct DbEnvironment {
    pub name: String,
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

pub type Environment = DbEnvironment;

impl Environment {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        // TODO: ignore conflicts
        sqlx::query("INSERT INTO environments (name) VALUES (?)")
            .bind(self.name.as_str())
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct Environments(pub Vec<Environment>);

use comfy_table::{Cell, Table};
use std::fmt::{self, Display, Formatter};
impl Display for Environments {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        // generate table
        let mut table = Table::new();
        table
            .load_preset(super::format::TABLE_FORMAT)
            .set_table_width(80)
            .set_header(vec!["environment"]);
        // add rows from the vector
        for env in self.0.iter() {
            table.add_row(vec![Cell::new(&env.name)]);
        }
        // print a blank line
        writeln!(f)?;
        // indent each row by two spaces
        for line in table.to_string().split('\n') {
            writeln!(f, "  {}", line)?;
        }
        Ok(())
    }
}
