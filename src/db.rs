use chrono::Utc;
use comfy_table::Cell;
use regex::Regex;
use rusqlite::{params, Connection, NO_PARAMS};

pub enum DbError {
    Rusqlite(rusqlite::Error),
    NotFound,
}
pub struct Db {
    conn: Connection,
}
// TODO: organize this code
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

        // TODO: multiple of the same variable name / environment
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS variables (
                  rowid           INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  environment     TEXT NOT NULL,
                  value           TEXT,
                  source          TEXT,
                  timestamp       TEXT,
                  UNIQUE(name, environment)
              )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS input_options (
                  request_name    TEXT NOT NULL,
                  option_name     TEXT NOT NULL,
                  value           TEXT,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
              )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS output_options (
                  request_name      TEXT NOT NULL,
                  option_name       TEXT NOT NULL,
                  extraction_source TEXT NOT NULL,
                  extraction_path   TEXT NOT NULL,
                  FOREIGN KEY(request_name) REFERENCES requests(name),
                  UNIQUE(request_name, option_name)
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

    pub fn get_input_options(&self) -> Result<Vec<RequestInput>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT request_name, option_name, value FROM input_options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(RequestInput {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                value: row.get(2)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    pub fn get_output_options(&self) -> Result<Vec<RequestOutput>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT request_name, option_name, extraction_source, extraction_path FROM output_options;")?;

        let opts = stmt.query_map(NO_PARAMS, |row| {
            Ok(RequestOutput {
                request_name: row.get(0)?,
                option_name: row.get(1)?,
                extraction_source: row.get(2)?,
                extraction_path: row.get(3)?,
            })
        })?;

        // TODO: print a warning for errors
        Ok(opts.filter_map(|opt| opt.ok()).collect())
    }
    pub fn get_unique_request_names_from_options(&self) -> Result<Vec<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT request_name FROM input_options;")?;

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
            self.create_input_option(RequestInput::from_variable(&req.name, var_name))?;
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
    pub fn upsert_variable(&self, var: Variable) -> Result<(), DbError> {
        let vars: Vec<Variable> = self
            .get_variables()?
            .into_iter()
            .filter(|x| x == &var)
            .collect();
        if vars.len() >= 1 {
            // update
            self.conn.execute(
                "UPDATE variables SET value = ?1, timestamp = ?2, source = ?3 WHERE rowid = ?4;",
                params![
                    var.value,
                    format!("{}", Utc::now().format("%Y-%m-%d %T %Z")),
                    var.source,
                    vars[0].rowid,
                ],
            )?;
        } else {
            // create
            self.create_variable(var)?;
        }
        Ok(())
    }
    pub fn create_input_option(&self, opt: RequestInput) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO input_options (request_name, option_name, value)
                  VALUES (?1, ?2, ?3);",
            params![opt.request_name, opt.option_name, opt.value,],
        )?;
        Ok(())
    }
    pub fn create_output_option(&self, opt: RequestOutput) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO output_options (request_name, option_name, extraction_source, extraction_path)
                  VALUES (?1, ?2, ?3, ?4);",
            params![
                opt.request_name,
                opt.option_name,
                opt.extraction_source,
                opt.extraction_path,
            ],
        )?;
        Ok(())
    }

    pub fn update_input_option(&self, opt: RequestInput) -> Result<(), DbError> {
        let num = self.conn.execute(
            "UPDATE input_options SET value = ?1 WHERE request_name = ?2 AND option_name = ?3;",
            params![opt.value, opt.request_name, opt.option_name],
        )?;
        if num == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    pub fn delete_request_by_name(&self, request: &str) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM input_options WHERE request_name = ?1;",
            params![request],
        )?;
        self.conn.execute(
            "DELETE FROM output_options WHERE request_name = ?1;",
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
    pub fn delete_input_option_by_name(
        &self,
        request_name: &str,
        option_name: &str,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM input_options WHERE request_name = ?1 AND option_name = ?2;",
            params![request_name, option_name],
        )?;
        Ok(())
    }
    pub fn delete_output_option_by_name(
        &self,
        request_name: &str,
        option_name: &str,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM output_options WHERE request_name = ?1 AND option_name = ?2;",
            params![request_name, option_name],
        )?;
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
    pub fn to_string(&self) -> &str {
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
pub struct RequestInput {
    request_name: String,
    option_name: String,
    value: Option<String>,
}
pub struct RequestOutput {
    request_name: String, // get-token
    option_name: String,  // token
    // TODO: enum
    extraction_source: String, // body
    extraction_path: String,   // .access_token
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
    pub fn substitute_options(&mut self, opts: &Vec<RequestInput>) -> bool {
        // TODO: better replacement for all options
        //       this could result in some unexpected behavior
        //       will need to do a two pass approach:
        //          1. find all start/end indices
        //          2. iterate backwards to perform replacement
        // find all variables and replace with values in options
        for opt in opts {
            if opt.value().is_none() {
                // All input options are required
                return false;
            }
            let old = format!("{{{}}}", &opt.option_name);
            let new = opt.value().unwrap();
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
impl RequestInput {
    pub fn new(req_name: &str, opt_name: &str, value: Option<String>) -> RequestInput {
        RequestInput {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            value,
        }
    }
    pub fn from_variable(req_name: &str, var_name: &str) -> RequestInput {
        RequestInput::new(req_name, var_name, None)
    }

    pub fn request_name(&self) -> &str {
        self.request_name.as_ref()
    }
    pub fn option_name(&self) -> &str {
        self.option_name.as_ref()
    }
    pub fn value(&self) -> Option<&str> {
        match &self.value {
            Some(x) => Some(x.as_str()),
            None => None,
        }
    }

    pub fn update_value(&mut self, val: Option<String>) {
        self.value = val;
    }
}
impl RequestOutput {
    pub fn new(req_name: &str, opt_name: &str, ext_src: &str, ext_path: &str) -> RequestOutput {
        RequestOutput {
            request_name: String::from(req_name),
            option_name: String::from(opt_name),
            extraction_source: String::from(ext_src),
            extraction_path: String::from(ext_path),
        }
    }
    pub fn request_name(&self) -> &str {
        self.request_name.as_ref()
    }
    pub fn option_name(&self) -> &str {
        self.option_name.as_ref()
    }
    pub fn extraction_source(&self) -> &str {
        self.extraction_source.as_ref()
    }
    pub fn path(&self) -> &str {
        self.extraction_path.as_ref()
    }
}
impl Variable {
    pub fn new(name: &str, env: &str, value: Option<&str>, source: Option<&str>) -> Variable {
        let value = match value {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        let source = match source {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        Variable {
            rowid: 0,
            name: String::from(name),
            environment: String::from(env),
            value,
            source,
            timestamp: None,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn environment(&self) -> &str {
        self.environment.as_ref()
    }
    pub fn value(&self) -> Option<&str> {
        match &self.value {
            Some(x) => Some(x.as_ref()),
            None => None,
        }
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
    fn column_names(&self) -> Vec<Cell>;
    // TODO: iterator
    fn rows(&self) -> Vec<Vec<Cell>>;
}
impl PrintableTable for Vec<Request> {
    fn column_names(&self) -> Vec<Cell> {
        vec![
            Cell::new("name"),
            Cell::new("method"),
            Cell::new("url"),
            Cell::new("headers"),
            Cell::new("body?"),
        ]
    }
    fn rows(&self) -> Vec<Vec<Cell>> {
        self.iter()
            .map(|req| {
                let has_body = {
                    if req.body.is_some() {
                        "true"
                    } else {
                        "false"
                    }
                };
                vec![
                    Cell::new(&req.name),
                    Cell::new(req.method.to_string()),
                    Cell::new(&req.url),
                    Cell::new(req.headers.as_ref().unwrap_or(&String::from(""))),
                    Cell::new(has_body),
                ]
            })
            .collect()
    }
}
impl PrintableTable for Vec<Variable> {
    fn column_names(&self) -> Vec<Cell> {
        vec![
            // Cell::new("rowid"),
            Cell::new("name"),
            Cell::new("environment"),
            Cell::new("value"),
            Cell::new("source"),
            // Cell::new("timestamp"),
        ]
    }
    fn rows(&self) -> Vec<Vec<Cell>> {
        self.iter()
            .map(|var| {
                vec![
                    // Cell::new(var.rowid),
                    Cell::new(&var.name),
                    Cell::new(&var.environment),
                    Cell::new(var.value.as_ref().unwrap_or(&String::from(""))),
                    Cell::new(var.source.as_ref().unwrap_or(&String::from(""))),
                    // Cell::new(var.timestamp.as_ref().unwrap_or(&String::from(""))),
                ]
            })
            .collect()
    }
}
impl PrintableTable for Vec<Environment> {
    fn column_names(&self) -> Vec<Cell> {
        vec![Cell::new("environment")]
    }
    fn rows(&self) -> Vec<Vec<Cell>> {
        self.iter()
            .map(|env| vec![Cell::new(&env.environment)])
            .collect()
    }
}
impl PrintableTable for Vec<RequestInput> {
    fn column_names(&self) -> Vec<Cell> {
        vec![
            Cell::new("request_name"),
            Cell::new("option_name"),
            Cell::new("value"),
        ]
    }
    fn rows(&self) -> Vec<Vec<Cell>> {
        self.iter()
            .map(|opt| {
                vec![
                    Cell::new(&opt.request_name),
                    Cell::new(&opt.option_name),
                    Cell::new(opt.value.as_ref().unwrap_or(&String::from(""))),
                ]
            })
            .collect()
    }
}
impl PrintableTable for Vec<String> {
    fn column_names(&self) -> Vec<Cell> {
        vec![Cell::new(&self[0])]
    }
    fn rows(&self) -> Vec<Vec<Cell>> {
        let mut iter = self.iter();
        iter.next();
        iter.map(|row| vec![Cell::new(row)]).collect()
    }
}

impl std::cmp::PartialEq for Variable {
    fn eq(&self, other: &Variable) -> bool {
        self.name == other.name && self.environment == other.environment
        // && self.source == other.source
    }
}
