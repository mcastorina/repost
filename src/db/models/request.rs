use super::variable::VarString;
use reqwest::{Body, Method};
use serde_json;
use sqlx::{Error, FromRow, SqlitePool};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, FromRow, PartialEq)]
/// Database representation of a Request
pub struct DbRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: Option<Vec<u8>>,
    pub body_kind: Option<String>,
}

impl From<Request> for DbRequest {
    fn from(req: Request) -> Self {
        let kind = |rb: &RequestBody| {
            match rb {
                RequestBody::Blob(_) => "raw",
                RequestBody::Payload(_) => "var",
            }
            .to_string()
        };
        Self {
            name: req.name.into(),
            method: req.method.to_string(),
            url: req.url.to_string(),
            // Serialization can fail if `T`'s implementation of `Serialize` decides to fail, or if
            // `T` contains a map with non-string keys. We are serializing a vector of tuples with
            // known types, which should never fail and can be safely unwrapped.
            headers: serde_json::to_string(&req.headers).unwrap(),
            body_kind: req.body.as_ref().map(kind),
            body: req.body.map(|body| body.as_bytes()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    /// Name of the request
    pub name: String,
    /// HTTP method type
    pub method: Method,
    /// HTTP url string including protocol and parameters
    pub url: VarString,
    /// HTTP header key-value pairs
    pub headers: Vec<(VarString, VarString)>,
    /// HTTP request body
    pub body: Option<RequestBody>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestBody {
    /// A blob of bytes
    Blob(Vec<u8>),
    /// A body that contains a variable string
    Payload(VarString),
}

impl RequestBody {
    fn as_bytes(self) -> Vec<u8> {
        match self {
            Self::Blob(body) => body,
            Self::Payload(v) => v.to_string().as_bytes().to_owned(),
        }
    }
}

impl Request {
    /// Create a new request object. Please note that method is case sensitive.
    pub fn new<N, M, U>(name: N, method: M, url: U) -> Self
    where
        N: Into<String>,
        M: TryInto<Method>,
        <M as TryInto<Method>>::Error: std::fmt::Debug,
        U: Into<VarString>,
    {
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

    /// Add a single header key-value pair to the request.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<VarString>,
        V: Into<VarString>,
    {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Add many header key-value pairs to the request.
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

    /// Set the request body to exact data.
    /// This function is recommended if you don't intend to use variables in the request body.
    pub fn body_raw<B>(mut self, body: B) -> Self
    where
        B: Into<Body>,
    {
        self.body = body
            .into()
            .as_bytes()
            .map(|body| RequestBody::Blob(body.to_owned()));
        self
    }

    /// Set the request body to a variable string.
    /// Bodies set by this function will be subject to variable replacement.
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<VarString>,
    {
        self.body = Some(RequestBody::Payload(body.into()));
        self
    }

    pub async fn save(self, pool: &SqlitePool) -> Result<(), Error> {
        let db_req: DbRequest = self.into();
        sqlx::query(
            "INSERT INTO requests
                (name, method, url, headers, body_kind, body)
                VALUES (?, ?, ?, ?, ?, ?);",
        )
        .bind(db_req.name.as_str())
        .bind(db_req.method.as_str())
        .bind(db_req.url.as_str())
        .bind(db_req.headers.as_str())
        .bind(&db_req.body_kind)
        .bind(&db_req.body)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl TryFrom<DbRequest> for Request {
    type Error = ();
    fn try_from(req: DbRequest) -> Result<Self, Self::Error> {
        let headers = serde_json::from_str(&req.headers).map_err(|_| ())?;
        let body_kind = req.body_kind.as_deref();
        let body = req
            .body
            .map(|body| match body_kind {
                Some("raw") => Ok(RequestBody::Blob(body)),
                Some("var") => String::from_utf8(body)
                    .map(|b| RequestBody::Payload(b.into()))
                    .map_err(|_| "not UTF-8"),
                Some(_) => Err("expected 'raw' or 'var' body kind"),
                None => Err("found body, but missing it's kind"),
            })
            .transpose()
            .map_err(|_| ())?;
        Ok(Self {
            name: req.name.into(),
            method: req.method.parse().map_err(|_| ())?,
            url: req.url.into(),
            headers,
            body,
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
