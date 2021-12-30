use super::variable::VarString;
use crate::db::models as db;
use reqwest::Method;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    /// Name of the request
    pub name: Cow<'a, str>,
    /// HTTP method type
    pub method: Method,
    /// HTTP url string including protocol and parameters
    pub url: VarString<'a>,
    /// HTTP header key-value pairs
    pub headers: Vec<(String, VarString<'a>)>,
    /// HTTP request body
    pub body: Option<RequestBody<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestBody<'a> {
    /// A blob of bytes
    Blob(&'a [u8]),
    /// A body that contains a variable string
    Payload(VarString<'a>),
}

impl<'a> Request<'a> {
    pub fn new<N, M, U>(name: N, method: M, url: U) -> Result<Self, ()>
    where
        N: Into<Cow<'a, str>>,
        M: TryInto<Method>,
        U: Into<VarString<'a>>,
    {
        // TODO: headers and body
        Ok(Self {
            name: name.into(),
            method: method.try_into().map_err(|_| ())?,
            url: url.into(),
            headers: vec![],
            body: None,
        })
    }
}

impl<'a> TryFrom<db::Request> for Request<'a> {
    type Error = ();
    fn try_from(req: db::Request) -> Result<Self, Self::Error> {
        // TODO: headers and body
        Ok(Self {
            name: req.name.into(),
            method: req.method.parse().map_err(|_| ())?,
            url: req.url.into(),
            headers: vec![],
            body: None,
        })
    }
}
