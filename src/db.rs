use chrono::Utc;
use rusqlite::{params, Connection, NO_PARAMS};

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

pub struct Db {
    conn: Connection,
}

pub struct Request {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<String>,
    pub body: Option<Vec<u8>>,
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
    pub environment: String,
}

impl Request {
    // TODO: use Option<enum> for method
    pub fn new(name: &str, method: Option<&str>, url: &str) -> Request {
        let method = Request::name_to_method(name, method);
        Request {
            name: String::from(name),
            method,
            url: String::from(url),
            headers: None,
            body: None,
        }
    }

    fn name_to_method(name: &str, method: Option<&str>) -> String {
        if let Some(x) = method {
            return String::from(x);
        }
        let name = name.to_lowercase();
        if name.starts_with("create") || name.starts_with("post") {
            String::from("POST")
        } else if name.starts_with("delete") {
            String::from("DELETE")
        } else if name.starts_with("replace") || name.starts_with("put") {
            String::from("PUT")
        } else if name.starts_with("update") || name.starts_with("patch") {
            String::from("PATCH")
        } else {
            String::from("GET")
        }
    }
}

pub enum DbError {
    Rusqlite(rusqlite::Error),
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

        Ok(())
    }

    pub fn get_requests(&self) -> Result<Vec<Request>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, method, url, headers, body FROM requests;")?;

        let requests = stmt.query_map(NO_PARAMS, |row| {
            Ok(Request {
                name: row.get(0)?,
                method: row.get(1)?,
                url: row.get(2)?,
                headers: row.get(3)?,
                body: row.get(4)?,
            })
        })?;

        Ok(requests.map(|req| req.unwrap()).collect())
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

        Ok(vars.map(|var| var.unwrap()).collect())
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

        Ok(envs.map(|env| env.unwrap()).collect())
    }

    pub fn create_request(&self, req: Request) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO requests (name, method, url, headers, body)
                  VALUES (?1, ?2, ?3, ?4, ?5);",
            params![req.name, req.method, req.url, req.headers, req.body],
        )?;
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
}
