use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection, NO_PARAMS};

pub enum DbError {
    Rusqlite(rusqlite::Error),
    NotFound,
}
pub struct Db {
    conn: Connection,
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
                  type            TEXT NOT NULL,
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
            .prepare("SELECT rowid, name, environment, value, source, timestamp FROM variables ORDER BY timestamp ASC;")?;

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
            .prepare("SELECT request_name, option_name, value, type FROM options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(RequestOption {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                value: row.get(2)?,
                option_type: row.get(3)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    pub fn get_unique_request_names_from_options(&self) -> Result<Vec<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT request_name FROM options;")?;

        let req_names = stmt.query_map(NO_PARAMS, |row| Ok(row.get(0)?))?;

        // TODO: print a warning for errors
        Ok(req_names.filter_map(|req_name| req_name.ok()).collect())
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
        self.conn.execute(
            "INSERT INTO options (request_name, option_name, value, type)
                  VALUES (?1, ?2, ?3, ?4);",
            params![opt.request_name, opt.option_name, opt.value, opt.option_type,],
        )?;
        Ok(())
    }

    pub fn update_option(&self, opt: RequestOption) -> Result<(), DbError> {
        let num = self.conn.execute(
            "UPDATE options SET value = ?1 WHERE request_name = ?2 AND option_name = ?3 AND type = ?4;",
            params![opt.value, opt.request_name, opt.option_name, opt.option_type])?;
        if num == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    pub fn delete_request_by_name(&self, request: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM options WHERE request_name = ?1;",
            params![request],
        )?;
        self.conn
            .execute("DELETE FROM requests WHERE name = ?1;", params![request])?;
        Ok(())
    }

    pub fn delete_variable_by_name(&self, variable: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM variables WHERE name = ?1;", params![variable])?;
        Ok(())
    }
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
    // TODO: enum
    option_type: String,
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
        let mut names: Vec<String> = re
            .captures_iter(&self.url)
            .map(|cap| String::from(cap.get(1).unwrap().as_str()))
            .collect();
        if let Some(headers) = &self.headers {
            let mut headers: Vec<String> = re
                .captures_iter(&headers)
                .map(|cap| String::from(cap.get(1).unwrap().as_str()))
                .collect();
            names.append(&mut headers);
        }
        if let Some(body) = &self.body {
            let re = regex::bytes::Regex::new(r"\{(.*?)\}").unwrap();
            let mut body: Vec<String> = re
                .captures_iter(&body)
                .map(|cap| String::from_utf8(cap.get(1).unwrap().as_bytes().to_vec()))
                .filter_map(|x| x.ok())
                .collect();
            names.append(&mut body);
        }

        // TODO: unique names
        Ok(names)
    }
    // TODO: return another type to ensure this does not get saved to the DB
    //       or used anywhere other than run
    pub fn substitute_options(&mut self, opts: Vec<RequestOption>) -> bool {
        // TODO: better replacement for all options
        //       this could result in some unexpected behavior
        //       will need to do a two pass approach:
        //          1. find all start/end indices
        //          2. iterate backwards to perform replacement
        // find all variables and replace with values in options
        for opt in opts.into_iter().filter(|opt| opt.option_type == "input") {
            if opt.value.is_none() {
                // All input options are required
                return false;
            }
            let old = format!("{{{}}}", &opt.option_name);
            let new = opt.value.unwrap();
            self.url = self.url.replace(&old, &new);
            if let Some(headers) = &self.headers {
                self.headers = Some(headers.replace(&old, &new));
            }
            if let Some(body) = &self.body {
                let old = format!(r"\{{{}\}}", &opt.option_name);
                let re = regex::bytes::Regex::new(&old).unwrap();
                self.body = Some(re.replace_all(&body, new.as_bytes()).to_vec());
            }
        }
        true
    }
    pub fn has_option(&self, opt: &RequestOption) -> bool {
        self.name == opt.request_name
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
    pub fn consume_body(&mut self) -> Option<Vec<u8>> {
        self.body.take()
    }
}
impl RequestOption {
    pub fn new(req_name: &str, opt_name: &str, value: Option<String>, opt_type: &str) -> RequestOption {
        RequestOption {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            value,
            option_type: String::from(opt_type),
        }
    }
    pub fn from_variable(req_name: &str, var_name: &str) -> RequestOption {
        RequestOption::new(req_name, var_name, None, "input")
    }

    pub fn request_name(&self) -> &str {
        self.request_name.as_ref()
    }
    pub fn option_name(&self) -> &str {
        self.option_name.as_ref()
    }

    pub fn update_value(&mut self, val: Option<String>) {
        self.value = val;
    }
}
impl Variable {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn environment(&self) -> &str {
        self.environment.as_ref()
    }
    pub fn consume_value(&mut self) -> Option<String> {
        self.value.take()
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> DbError {
        DbError::Rusqlite(err)
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
        row!["request_name", "option_name", "value", "type"]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        self.iter()
            .map(|opt| {
                row![
                    opt.request_name,
                    opt.option_name,
                    opt.value.as_ref().unwrap_or(&String::from("")),
                    opt.option_type,
                ]
            })
            .collect()
    }
}
impl PrintableTable for Vec<String> {
    fn column_names(&self) -> prettytable::Row {
        row![&self[0]]
    }
    fn rows(&self) -> Vec<prettytable::Row> {
        let mut iter = self.iter();
        iter.next();
        iter.map(|row| row![row]).collect()
    }
}
