use super::Environment;
use crate::db::models as db;
use chrono::{DateTime, Local};
use std::borrow::Cow;
use std::collections::HashSet;
use std::str::FromStr;

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

#[derive(Debug, PartialEq, Eq, Clone)]
/// A string that contains {variables}
pub struct VarString<'a> {
    /// Source string containing variables
    source: Cow<'a, str>,

    /// Set of variables found in source
    vars: HashSet<Cow<'a, str>>,
}

impl<'a, T> From<T> for VarString<'a>
where
    T: Into<Cow<'a, str>>,
{
    fn from(t: T) -> Self {
        // TODO: fill hashset
        Self {
            source: t.into(),
            vars: HashSet::new(),
        }
    }
}

impl From<VarString<'_>> for String {
    fn from(v: VarString<'_>) -> Self {
        v.source.into()
    }
}
