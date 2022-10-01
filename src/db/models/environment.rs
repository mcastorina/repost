use sqlx::{Error, FromRow, SqlitePool};

#[derive(Debug, PartialEq, Eq, FromRow, Clone)]
pub struct Environment {
    pub name: String,
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
