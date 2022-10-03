use super::Environment;
use chrono::{DateTime, Local};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, SqlitePool};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, FromRow, PartialEq)]
pub struct DbVariable {
    pub id: i32,
    pub name: String,
    pub env: String,
    pub value: Option<String>,
    pub source: String,
    pub timestamp: DateTime<Local>,
}

impl<'a> From<Variable> for DbVariable {
    fn from(var: Variable) -> Self {
        Self {
            id: var.id.unwrap_or(0),
            name: var.name.into(),
            env: var.env.name.into(),
            value: var.value.map(|cow| cow.into()),
            source: var.source.into(),
            timestamp: var.timestamp,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable {
    pub id: Option<i32>,
    pub name: String,
    pub env: Environment,
    pub value: Option<String>,
    pub source: String,
    pub timestamp: DateTime<Local>,
}

impl Variable {
    pub fn new<N, E, V, S>(name: N, env: E, value: V, source: S) -> Self
    where
        N: Into<String>,
        E: Into<Environment>,
        V: Into<String>,
        S: Into<String>,
    {
        Self {
            id: None,
            name: name.into(),
            env: env.into(),
            value: Some(value.into()),
            source: source.into(),
            timestamp: Local::now(),
        }
    }

    pub async fn save(self, pool: &SqlitePool) -> Result<(), Error> {
        match self.id {
            Some(id) => self.update(id, pool).await?,
            None => self.create(pool).await?,
        };
        Ok(())
    }

    async fn create(self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO variables
                (name, env, value, source, timestamp)
                VALUES (?, ?, ?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.env.as_ref())
        .bind(self.value.as_ref())
        .bind(self.source.as_str())
        .bind(self.timestamp)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update(self, id: i32, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "UPDATE variables SET
                (name, env, value, source, timestamp) = (?, ?, ?, ?, ?)
                WHERE id = ?;",
        )
        .bind(self.name.as_str())
        .bind(self.env.as_ref())
        .bind(self.value.as_ref())
        .bind(self.source.as_str())
        .bind(self.timestamp)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl From<DbVariable> for Variable {
    fn from(var: DbVariable) -> Self {
        Self {
            id: Some(var.id),
            name: var.name.into(),
            env: var.env.into(),
            value: var.value.map(|s| s.into()),
            source: var.source.into(),
            timestamp: var.timestamp,
        }
    }
}

/// A string that contains {variables}.
/// Variables may not be nested, and variable names begin with an alphanumeric character and
/// contains alphanumeric, and `-` characters.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VarString {
    /// Source string containing variables
    source: String,

    /// Set of variables found in source
    vars: HashSet<String>,
}

impl VarString {
    pub fn as_str(&self) -> &str {
        self.source.as_str()
    }

    pub fn variables(&self) -> HashSet<&str> {
        let mut vars = HashSet::new();
        for var in &self.vars {
            vars.insert(var.as_str());
        }
        vars
    }
}

impl<T> From<T> for VarString
where
    T: Into<String>,
{
    fn from(source: T) -> Self {
        let source = source.into();
        // TODO: lazy static
        let re = Regex::new(r"\{(\w[\w\d-]*)\}").unwrap();
        let mut vars = HashSet::new();
        for cap in re.captures_iter(&source) {
            vars.insert(cap[1].to_string());
        }

        Self { source, vars }
    }
}

use std::convert::Infallible;
use std::str::FromStr;
impl FromStr for VarString {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(s.into())
    }
}

impl Display for VarString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.source)
    }
}

#[cfg(test)]
mod test {
    use super::VarString;

    #[test]
    fn var_string() {
        let test = |s: &str, expected: Vec<&str>| {
            let vs: VarString = s.into();
            assert_eq!(vs.source, s);
            assert_eq!(
                vs.vars.len(),
                expected.len(),
                "{}: parsed {:?}, expected {:?}",
                vs.source,
                vs.vars,
                expected
            );
            for var in expected {
                assert!(vs.vars.contains(var));
            }
        };
        test("foo{bar}{bar}", vec!["bar"]);
        test("foo{bar{baz}}", vec!["baz"]);
        test("foo{bar", vec![]);
        test("foo {} bar", vec![]);
        test("foo {-} bar", vec![]);
        test("foo {bar-3} baz", vec!["bar-3"]);
        test("{foo}{bar} {baz}", vec!["foo", "bar", "baz"]);
        test("{foo{bar}", vec!["bar"]);
        test("{9}", vec!["9"]);
        test("{foo_bar}{_bar}", vec!["foo_bar", "_bar"]);
    }
}
