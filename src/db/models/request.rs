use super::variable::VarString;
use reqwest::Method;
use sqlx::{Error, FromRow, SqlitePool};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, FromRow, PartialEq)]
/// Database representation of a Request
pub struct DbRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: Vec<u8>,
}

impl DbRequest {
    pub async fn save(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO requests
                (name, method, url, headers, body)
                VALUES (?, ?, ?, ?, ?);",
        )
        .bind(self.name.as_str())
        .bind(self.method.as_str())
        .bind(self.url.as_str())
        .bind(self.headers.as_str())
        .bind(&self.body)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl<'a> From<Request<'a>> for DbRequest {
    fn from(req: Request<'a>) -> Self {
        // TODO: headers and body
        Self {
            name: req.name.into(),
            method: req.method.to_string(),
            url: req.url.to_string(),
            headers: String::new(),
            body: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    /// Name of the request
    pub name: String,
    /// HTTP method type
    pub method: Method,
    /// HTTP url string including protocol and parameters
    pub url: VarString,
    /// HTTP header key-value pairs
    pub headers: Vec<(VarString, VarString)>,
    /// HTTP request body
    pub body: Option<RequestBody<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestBody<'a> {
    /// A blob of bytes
    Blob(&'a [u8]),
    /// A body that contains a variable string
    Payload(VarString),
}

impl<'a> Request<'a> {
    /// Create a new request object. Please note that method is case sensitive.
    pub fn new<N, M, U>(name: N, method: M, url: U) -> Self
    where
        N: Into<String>,
        M: TryInto<Method>,
        <M as TryInto<Method>>::Error: std::fmt::Debug,
        U: Into<VarString>,
    {
        // TODO: headers and body
        Self {
            name: name.into(),
            // Method::TryInto returns Infallible so it's okay to unwrap
            // unfortunately, From<&str> is not implemented for Method
            method: method.try_into().unwrap(),
            url: url.into(),
            headers: vec![],
            body: None,
        }
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<VarString>,
        V: Into<VarString>,
    {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn headers<K, V>(mut self, headers: Vec<(K, V)>) -> Self
    where
        K: Into<VarString>,
        V: Into<VarString>,
    {
        for (k, v) in headers {
            self = self.header(k, v)
        }
        self
    }
}

impl<'a> TryFrom<DbRequest> for Request<'a> {
    type Error = ();
    fn try_from(req: DbRequest) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod test {
    use super::{Method, Request};
    use std::str::FromStr;

    macro_rules! method {
        ($m:expr) => {
            Request::new("name", $m, "url").method
        };
    }

    #[test]
    fn method() {
        assert_eq!(method!(Method::GET), Method::GET);
        assert_eq!(method!("GET"), Method::GET);
        assert_eq!(method!("get"), Method::from_str("get").unwrap());
        assert_eq!(method!("foo"), Method::from_str("foo").unwrap());
    }

    #[test]
    fn headers() {
        assert_eq!(
            Request::new("foo", "bar", "baz")
                .header("foo", "bar")
                .header("bar", "baz")
                .headers
                .len(),
            2
        );
        assert_eq!(
            Request::new("foo", "bar", "baz")
                .header("foo", "bar")
                .headers(vec![("foo", "bar"), ("bar", "baz")])
                .headers
                .len(),
            3
        );
    }
}
