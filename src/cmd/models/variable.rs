use super::Environment;
use crate::db::models as db;
use chrono::{DateTime, Local};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable<'a> {
    pub name: Cow<'a, str>,
    pub env: Environment<'a>,
    pub value: Option<Cow<'a, str>>,
    pub source: Cow<'a, str>,
    pub timestamp: DateTime<Local>,
}

impl<'a> Variable<'a> {
    pub fn new<N, E, V, S>(name: N, env: E, value: V, source: S) -> Self
    where
        N: Into<Cow<'a, str>>,
        E: Into<Environment<'a>>,
        V: Into<Cow<'a, str>>,
        S: Into<Cow<'a, str>>,
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

impl<'a> From<db::Variable> for Variable<'a> {
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
