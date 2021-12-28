use super::Environment;
use crate::db::models as db;
use chrono::{DateTime, Local};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable<'n, 'e, 'v, 's> {
    pub name: Cow<'n, str>,
    pub env: Environment<'e>,
    pub value: Option<Cow<'v, str>>,
    pub source: Cow<'s, str>,
    pub timestamp: DateTime<Local>,
}

impl<'n, 'e, 'v, 's> Variable<'n, 'e, 'v, 's> {
    pub fn new<N, E, V, S>(name: N, env: E, value: V, source: S) -> Self
    where
        N: Into<Cow<'n, str>>,
        E: Into<Environment<'e>>,
        V: Into<Cow<'v, str>>,
        S: Into<Cow<'s, str>>,
    {
        Self {
            name: name.into(),
            env: env.into(),
            value: Some(value.into()),
            source: source.into(),
            timestamp: Local::now(),
        }
    }
}

impl<'n, 'e, 'v, 's> From<db::Variable> for Variable<'n, 'e, 'v, 's> {
    fn from(var: db::Variable) -> Self {
        Self {
            name: var.name.into(),
            env: var.env.into(),
            value: var.value.map(|s| s.into()),
            source: var.source.into(),
            timestamp: var.timestamp,
        }
    }
}
