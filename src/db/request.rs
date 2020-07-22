use super::PrintableTableStruct;
use super::{DbObject, InputOption};
use crate::error::{Error, ErrorKind, Result};
use comfy_table::Cell;
use regex::Regex;
use rusqlite::{params, Connection, NO_PARAMS};
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Request {
    name: String,
    method: Method,
    url: String,
    headers: Option<String>,
    body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
}
impl Method {
    // TODO: implement display for method
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
        // TODO: case insensitive
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

impl Request {
    pub fn new(name: &str, method: Option<Method>, url: &str) -> Request {
        let method = method.unwrap_or(Request::name_to_method(name));
        Request {
            name: String::from(name),
            method: method,
            url: String::from(url),
            headers: None,
            body: None,
        }
    }
    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                  name            TEXT PRIMARY KEY,
                  method          TEXT NOT NULL,
                  url             TEXT NOT NULL,
                  headers         TEXT,
                  body            BLOB
              )",
            NO_PARAMS,
        )?;
        Ok(())
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
    pub fn add_query_param(&mut self, query: &str) {
        if self.url.contains('?') {
            self.url.push('&');
        } else {
            self.url.push('?');
        }
        self.url.push_str(query);
    }
    pub fn set_body(&mut self, body: Option<Vec<u8>>) {
        self.body = body;
    }

    fn name_to_method(name: &str) -> Method {
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
    pub fn variable_names(&self) -> HashSet<String> {
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

        HashSet::from_iter(names.into_iter())
    }
    pub fn replace_input_options(&mut self, options: &Vec<InputOption>) -> Result<()> {
        // TODO: better replacement for all options
        //       this could result in some unexpected behavior
        //       will need to do a two pass approach:
        //          1. find all start/end indices
        //          2. iterate backwards to perform replacement
        // find all variables and replace with values in options
        let missing_opts: Vec<_> = options
            .iter()
            .filter(|opt| opt.values().len() == 0)
            .map(|opt| String::from(opt.option_name()))
            .collect();
        if missing_opts.len() > 0 {
            // All input options are required
            return Err(Error::new(ErrorKind::MissingOptions(missing_opts)));
        }
        for opt in options {
            if opt.values().len() == 0 {}
            let old = format!("{{{}}}", opt.option_name());
            let new = opt.values().remove(0);
            self.url = self.url.replace(&old, &new);
            self.headers = self.headers.as_ref().map(|h| h.replace(&old, &new));
            if let Some(body) = &self.body {
                let old = format!(r"\{{{}\}}", opt.option_name());
                let re = regex::bytes::Regex::new(&old).unwrap();
                self.body = Some(re.replace_all(&body, new.as_bytes()).to_vec());
            }
        }
        Ok(())
    }
}

impl DbObject for Request {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO requests (name, method, url, headers, body)
                  VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                self.name,
                self.method.to_string(),
                self.url,
                self.headers,
                self.body
            ],
        )?;
        // create input options
        for var_name in self.variable_names().iter() {
            InputOption::new(self.name(), var_name, vec![]).create(conn)?;
        }
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        // TODO: should this get the options for the request, then call delete()
        //       on the objects?
        conn.execute(
            "DELETE FROM input_options WHERE request_name = ?1;",
            params![self.name],
        )?;
        conn.execute(
            "DELETE FROM output_options WHERE request_name = ?1;",
            params![self.name],
        )?;
        conn.execute("DELETE FROM requests WHERE name = ?1;", params![self.name])?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        // TODO: update input/output options
        let num = conn.execute(
            "UPDATE requests SET method = ?2, url = ?3, headers = ?4, body = ?5 WHERE name = ?1;",
            params![
                self.name,
                self.method.to_string(),
                self.url,
                self.headers,
                self.body
            ],
        )?;
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Request>> {
        let mut stmt = conn.prepare("SELECT name, method, url, headers, body FROM requests;")?;

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

        // TODO: print a warning for errors
        Ok(requests.filter_map(|req| req.ok()).collect())
    }
    fn name(&self) -> &str {
        self.name()
    }
}

impl PrintableTableStruct for Request {
    fn get_header() -> Vec<Cell> {
        vec![
            Cell::new("name"),
            Cell::new("method"),
            Cell::new("url"),
            Cell::new("headers"),
            Cell::new("body?"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        let has_body = {
            if self.body.is_some() {
                "true"
            } else {
                "false"
            }
        };
        vec![vec![
            Cell::new(&self.name),
            Cell::new(self.method.to_string()),
            Cell::new(&self.url),
            Cell::new(self.headers.as_ref().unwrap_or(&String::from(""))),
            Cell::new(has_body),
        ]]
    }
}
