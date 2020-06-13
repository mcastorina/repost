use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection, NO_PARAMS};

pub struct Db {
    conn: Connection,
}

pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
}

impl Method {
    fn to_string(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
        }
    }
    pub fn new(s: &str) -> Method {
        match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            _ => Method::GET,
        }
    }
}

pub struct Request {
    name: String,
    method: Method,
    url: String,
    headers: Option<String>,
    body: Option<Vec<u8>>,
}

pub struct RequestOption {
    request_name: String,
    option_name: String,
    value: Option<String>,
    required: bool,
}

pub struct Variable {
    pub rowid: u32,
    pub name: String,
    pub environment: String,
    pub value: Option<String>,
    pub source: Option<String>,
    pub timestamp: Option<String>,
}

pub struct Environment {
    environment: String,
}

impl Request {
    pub fn new(name: &str, method: Option<Method>, url: &str) -> Request {
        Request {
            name: String::from(name),
            method: Request::name_to_method(name, method),
            url: String::from(url),
            headers: None,
            body: None,
        }
    }
    pub fn add_header(&mut self, key: &str, value: &str) {
        let mut headers = {
            match &self.headers {
                Some(x) => format!("{}\n", x),
                None => String::new(),
            }
        };
        headers.push_str(format!("{}: {}", key, value).as_ref());
        self.headers = Some(headers);
    }
    pub fn set_body(&mut self, body: Option<Vec<u8>>) {
        self.body = body;
    }

    fn name_to_method(name: &str, method: Option<Method>) -> Method {
        if let Some(x) = method {
            return x;
        }
        let name = name.to_lowercase();
        if name.starts_with("create") || name.starts_with("post") {
            Method::POST
        } else if name.starts_with("delete") {
            Method::DELETE
        } else if name.starts_with("replace") || name.starts_with("put") {
            Method::PUT
        } else if name.starts_with("update") || name.starts_with("patch") {
            Method::PATCH
        } else if name.starts_with("head") {
            Method::HEAD
        } else {
            Method::GET
        }
    }
    pub fn get_variable_names(&self) -> Result<Vec<String>, DbError> {
        // find all variables in the request
        // TODO: lazy static
        let re = Regex::new(r"\{(.*?)\}").unwrap();
        Ok(re
            .captures_iter(&self.url)
            .map(|cap| String::from(cap.get(1).unwrap().as_str()))
            .collect())
    }
    pub fn substitute_variables(&self, vars: Vec<Variable>) -> bool {
        false
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
    pub fn headers(&self) -> &Option<String> {
        &self.headers
    }
    pub fn body(&self) -> &Option<Vec<u8>> {
        &self.body
    }
}

impl RequestOption {
    pub fn from_variable(req_name: &str, var_name: &str) -> RequestOption {
        RequestOption {
            request_name: String::from(req_name),
            option_name: String::from(var_name),
            value: None,
            required: true,
        }
    }
}

pub enum DbError {
    Rusqlite(rusqlite::Error),
    NotFound,
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> DbError {
        DbError::Rusqlite(err)
    }
}

impl Db {
    pub fn new(path: &str) -> Result<Db, DbError> {
        let db = Db {
            conn: Connection::open(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                  name            TEXT PRIMARY KEY,
                  method          TEXT NOT NULL,
                  url             TEXT NOT NULL,
                  headers         TEXT,
                  body            BLOB
              )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS variables (
                  rowid           INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  environment     TEXT NOT NULL,
                  value           TEXT,
                  source          TEXT,
                  timestamp       TEXT
              )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS options (
                  request_name    TEXT,
                  option_name     TEXT NOT NULL,
                  value           TEXT,
                  required        INTEGER,
                  FOREIGN KEY(request_name) REFERENCES requests(name)
              )",
            NO_PARAMS,
        )?;

        self.conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;

        Ok(())
    }

    pub fn get_requests(&self) -> Result<Vec<Request>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, method, url, headers, body FROM requests;")?;

        let requests = stmt.query_map(NO_PARAMS, |row| {
            let s: String = row.get(1)?;
            Ok(Request {
                name: row.get(0)?,
                method: Method::new(s.as_ref()),
                url: row.get(2)?,
                headers: row.get(3)?,
                body: row.get(4)?,
            })
        })?;

        Ok(requests.filter_map(|req| req.ok()).collect())
    }
    pub fn get_request(&self, name: &str) -> Result<Request, DbError> {
        // TODO: single call
        for req in self.get_requests()? {
            if req.name == name {
                return Ok(req);
            }
        }
        Err(DbError::NotFound)
    }

    pub fn get_variables(&self) -> Result<Vec<Variable>, DbError> {
        let mut stmt = self.conn
            .prepare("SELECT rowid, name, environment, value, source, timestamp FROM variables ORDER BY timestamp DESC;")?;

        let vars = stmt.query_map(NO_PARAMS, |row| {
            Ok(Variable {
                rowid: row.get(0)?,
                name: row.get(1)?,
                environment: row.get(2)?,
                value: row.get(3)?,
                source: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(vars.filter_map(|var| var.ok()).collect())
    }

    pub fn get_options(&self) -> Result<Vec<RequestOption>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT request_name, option_name, value, required FROM options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(RequestOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                value: row.get(2)?,
                required: row.get(3)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }

    pub fn get_environments(&self) -> Result<Vec<Environment>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT environment FROM variables;")?;

        let envs = stmt.query_map(NO_PARAMS, |row| {
            Ok(Environment {
                environment: row.get(0)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(envs.filter_map(|env| env.ok()).collect())
    }
    pub fn environment_exists(&self, name: &str) -> Result<bool, DbError> {
        // TODO: single query
        Ok(self
            .get_environments()?
            .iter()
            .any(|x| x.environment == name))
    }

    pub fn create_request(&self, req: Request) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO requests (name, method, url, headers, body)
                  VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                req.name,
                req.method.to_string(),
                req.url,
                req.headers,
                req.body
            ],
        )?;
        for var_name in req.get_variable_names()?.iter() {
            self.create_option(RequestOption::from_variable(&req.name, var_name))?;
        }
        Ok(())
    }
    pub fn create_variable(&self, var: Variable) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO variables (name, environment, value, source, timestamp)
                  VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                var.name,
                var.environment,
                var.value,
                var.source,
                format!("{}", Utc::now().format("%Y-%m-%d %T %Z"))
            ],
        )?;
        Ok(())
    }
    pub fn create_option(&self, opt: RequestOption) -> Result<(), DbError> {
        println!("{} : {}", &opt.request_name, &opt.option_name);
        self.conn.execute(
            "INSERT INTO options (request_name, option_name, value, required)
                  VALUES (?1, ?2, ?3, ?4);",
            params![opt.request_name, opt.option_name, opt.value, opt.required,],
        )?;
        Ok(())
    }
}

pub trait PrintableTable {
    fn column_names(&self) -> prettytable::Row;
    fn rows(&self) -> Vec<prettytable::Row>;
}

impl PrintableTable for Vec<Request> {
    fn column_names(&self) -> prettytable::Row {
        row!["name", "method", "url", "headers", "body?"]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        self.iter()
            .map(|req| {
                let has_body = {
                    if req.body.is_some() {
                        "true"
                    } else {
                        "false"
                    }
                };
                row![
                    req.name,
                    req.method,
                    req.url,
                    req.headers.as_ref().unwrap_or(&String::from("")),
                    has_body,
                ]
            })
            .collect()
    }
}

impl PrintableTable for Vec<Variable> {
    fn column_names(&self) -> prettytable::Row {
        row![
            "rowid",
            "name",
            "environment",
            "value",
            "source",
            "timestamp"
        ]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        self.iter()
            .map(|var| {
                row![
                    var.rowid,
                    var.name,
                    var.environment,
                    var.value.as_ref().unwrap_or(&String::from("")),
                    var.source.as_ref().unwrap_or(&String::from("")),
                    var.timestamp.as_ref().unwrap_or(&String::from("")),
                ]
            })
            .collect()
    }
}

impl PrintableTable for Vec<Environment> {
    fn column_names(&self) -> prettytable::Row {
        row!["environment"]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        self.iter().map(|env| row![env.environment,]).collect()
    }
}

impl PrintableTable for Vec<RequestOption> {
    fn column_names(&self) -> prettytable::Row {
        row!["request_name", "option_name", "value", "required"]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        self.iter()
            .map(|opt| {
                row![
                    opt.request_name,
                    opt.option_name,
                    opt.value.as_ref().unwrap_or(&String::from("")),
                    opt.required
                ]
            })
            .collect()
    }
}
