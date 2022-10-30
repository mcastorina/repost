use super::variable::VarString;
use super::DisplayTable;
use crate::error::Error;
use reqwest::{Body, Method};
use serde_json;
use sqlx::{FromRow, SqlitePool};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, FromRow, PartialEq)]
/// Database representation of an Action
pub struct DbAction {
    pub id: i32,
    pub name: String,
    pub kind: String,
    pub spec: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Action {
    pub id: Option<i32>,
    pub name: String,
    pub kind: ActionKind,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize, serde::Serialize)]
pub enum ActionKind {
    #[serde(rename = "response_into_variable")]
    ResponseIntoVariable(ResponseIntoVariable),
}

impl ActionKind {
    pub fn header<R, H, V>(request: R, header: H, variable: V) -> Self
    where
        R: Into<String>,
        H: Into<String>,
        V: Into<String>,
    {
        Self::ResponseIntoVariable(ResponseIntoVariable {
            request: request.into(),
            variable: variable.into(),
            spec: ParseResponseSpec::Header(header.into()),
        })
    }

    pub fn json_path<R, P, V>(request: R, path: P, variable: V) -> Self
    where
        R: Into<String>,
        P: Into<String>,
        V: Into<String>,
    {
        Self::ResponseIntoVariable(ResponseIntoVariable {
            request: request.into(),
            variable: variable.into(),
            spec: ParseResponseSpec::JSONPath(path.into()),
        })
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ResponseIntoVariable(_) => "response-into-variable",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize, serde::Serialize)]
pub struct ResponseIntoVariable {
    request: String,
    variable: String,
    spec: ParseResponseSpec,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize, serde::Serialize)]
pub enum ParseResponseSpec {
    #[serde(rename = "header")]
    Header(String),
    #[serde(rename = "json_path")]
    JSONPath(String),
}

impl Action {
    /// Create a new Action.
    pub fn new<N, K>(name: N, kind: K) -> Self
    where
        N: Into<String>,
        K: Into<ActionKind>,
    {
        Self {
            id: None,
            name: name.into(),
            kind: kind.into(),
        }
    }

    /// Save the action to a sqlite database.
    pub async fn save(self, pool: &SqlitePool) -> Result<(), Error> {
        match self.id {
            Some(id) => self.update(id, pool).await?,
            None => self.create(pool).await?,
        };
        Ok(())
    }

    async fn update(self, id: i32, pool: &SqlitePool) -> Result<(), Error> {
        let spec = serde_json::to_string(&self.kind)?;
        sqlx::query(
            "UPDATE actions SET
                (name, kind, spec) = (?, ?, ?)
                WHERE id = ?;",
        )
        .bind(self.name.as_str())
        .bind(self.kind.as_str())
        .bind(spec)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn create(self, pool: &SqlitePool) -> Result<(), Error> {
        let spec = serde_json::to_string(&self.kind)?;
        sqlx::query("INSERT INTO actions (name, kind, spec) VALUES (?, ?, ?);")
            .bind(self.name.as_str())
            .bind(self.kind.as_str())
            .bind(spec)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl TryFrom<DbAction> for Action {
    type Error = Error;
    fn try_from(action: DbAction) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(action.id),
            name: action.name,
            kind: serde_json::from_slice(&action.spec)?,
        })
    }
}
