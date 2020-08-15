use super::PrintableTableStruct;
use super::{DbObject, InputOption, OutputOption, RequestRunner};
use crate::error::{Error, ErrorKind, Result};
use comfy_table::{Cell, Color};
use regex::Regex;
use reqwest::Method;
use rusqlite::{params, Connection, NO_PARAMS};
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Request {
    name: String,
    method: Method,
    url: String,
    headers: Vec<String>,
    body: Option<Vec<u8>>,

    input_options: Vec<InputOption>,
    output_options: Vec<OutputOption>,
}

impl Request {
    pub fn new(
        name: &str,
        method: Option<Method>,
        url: &str,
        headers: Vec<(&str, &str)>,
        body: Option<Vec<u8>>,
    ) -> Request {
        let method = method.unwrap_or(Request::name_to_method(name));
        let mut r = Request {
            name: String::from(name),
            method: method,
            url: String::from(url),
            headers: headers
                .into_iter()
                .map(|x| format!("{}: {}", x.0, x.1))
                .collect(),
            body: body,

            input_options: vec![],
            output_options: vec![],
        };
        r.input_options = r
            .variable_names()
            .iter()
            .map(|var_name| InputOption::new(r.name(), var_name, vec![]))
            .collect();
        r
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
    pub fn get_unique(conn: &Connection, name: &str) -> Result<Request> {
        // TODO: direct call to db
        let mut req: Vec<_> = Request::get_by(conn, |r| r.name() == name)?;
        if req.len() == 0 {
            Err(Error::new(ErrorKind::NotFound))
        } else {
            Ok(req.remove(0))
        }
    }
    pub fn exists(conn: &Connection, name: &str) -> Result<bool> {
        match Request::get_unique(conn, name) {
            Ok(_) => Ok(true),
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
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

    fn variable_names(&self) -> HashSet<String> {
        // find all variables in the request
        // TODO: lazy static
        let mut names = vec![];
        let url = variable_names(&self.url);
        names.extend(url);
        for header in &self.headers {
            names.extend(variable_names(header));
        }
        if let Some(body) = &self.body {
            let body = variable_names(&String::from_utf8(body.clone()).unwrap());
            names.extend(body);
        }

        HashSet::from_iter(names.into_iter())
    }
    pub fn set_option(&mut self, opt: &str, values: Vec<&str>) -> Result<()> {
        let opt = self
            .input_options
            .iter_mut()
            .find(|x| x.option_name() == opt);
        if opt.is_none() {
            return Err(Error::new(ErrorKind::NotFound));
        }
        opt.unwrap().set_values(values);
        Ok(())
    }
    pub fn add_extraction(&mut self, var_name: &str, typ: &str, key: &str) {
        self.output_options
            .push(OutputOption::new(self.name(), var_name, typ, key));
    }
    pub fn input_options(&self) -> &Vec<InputOption> {
        &self.input_options
    }
    pub fn output_options(&self) -> &Vec<OutputOption> {
        &self.output_options
    }
    pub fn create_requests(self) -> Result<RequestRunner> {
        RequestRunner::new(self)
    }
    pub fn delete_input_option(&mut self, name: &str) {
        self.input_options.retain(|x| x.option_name() != name);
    }
    pub fn delete_output_option(&mut self, name: &str) {
        self.output_options.retain(|x| x.option_name() != name);
    }

    fn stringify_headers(h: &Vec<String>) -> Option<String> {
        if h.len() == 0 {
            None
        } else {
            Some(h.join("\n"))
        }
    }
    fn unstringify_headers(h: Option<String>) -> Vec<String> {
        if h.is_none() {
            vec![]
        } else {
            h.unwrap().split("\n").map(String::from).collect()
        }
    }

    pub fn headers(&self) -> &Vec<String> {
        &self.headers
    }
    pub fn take_body(&mut self) -> Option<Vec<u8>> {
        self.body.take()
    }
    pub fn take_method(&mut self) -> Method {
        self.method.clone()
    }
    // TODO: make this private / function of RequestRunner
    pub fn replace_input_options(&mut self) -> Result<()> {
        // TODO: better replacement for all options
        //       this could result in some unexpected behavior
        //       will need to do a two pass approach:
        //          1. find all start/end indices
        //          2. iterate backwards to perform replacement
        // find all variables and replace with values in options
        let missing_opts: Vec<_> = self
            .input_options
            .iter()
            .filter(|opt| opt.values().len() == 0)
            .map(|opt| String::from(opt.option_name()))
            .collect();
        if missing_opts.len() > 0 {
            // All input options are required
            return Err(Error::new(ErrorKind::MissingOptions(missing_opts)));
        }
        for opt in self.input_options.iter() {
            if opt.values().len() == 0 {}
            let old = format!("{{{}}}", opt.option_name());
            let new = opt.values().remove(0);
            self.url = self.url.replace(&old, &new);
            for mut h in self.headers.iter_mut() {
                *h = h.replace(&old, &new);
            }
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
                Request::stringify_headers(&self.headers),
                self.body
            ],
        )?;
        // create input options
        for option in self.input_options.iter() {
            option.create(conn)?;
        }
        // create output options
        for option in self.output_options.iter() {
            option.create(conn)?;
        }
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        for option in self.input_options.iter() {
            option.delete(conn)?;
        }
        for option in self.output_options.iter() {
            option.delete(conn)?;
        }
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
                Request::stringify_headers(&self.headers),
                self.body
            ],
        )?;
        for option in self.input_options.iter() {
            option.upsert(conn)?;
        }
        for option in self.output_options.iter() {
            option.upsert(conn)?;
        }
        for opt in InputOption::get_by(conn, |opt| opt.request_name() == self.name())?.iter() {
            if !self.input_options.contains(opt) {
                opt.delete(conn)?;
            }
        }
        for opt in OutputOption::get_by(conn, |opt| opt.request_name() == self.name())?.iter() {
            if !self.output_options.contains(opt) {
                opt.delete(conn)?;
            }
        }
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Request>> {
        let mut stmt = conn.prepare("SELECT name, method, url, headers, body FROM requests;")?;

        let requests = stmt.query_map(NO_PARAMS, |row| {
            let name: String = row.get(0)?;
            let input_opts = InputOption::get_by(conn, |i| i.request_name() == &name);
            let output_opts = OutputOption::get_by(conn, |o| o.request_name() == &name);
            // TODO: error checking
            Ok(Request {
                name,
                method: Method::from_bytes(row.get::<_, String>(1)?.as_bytes())
                    .unwrap_or(Method::GET),
                url: row.get(2)?,
                headers: Request::unstringify_headers(row.get(3)?),
                body: row.get(4)?,

                input_options: input_opts.unwrap(),
                output_options: output_opts.unwrap(),
            })
        })?;

        // TODO: print a warning for errors
        Ok(requests.filter_map(|req| req.ok()).collect())
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
    fn get_row(&self) -> Vec<Cell> {
        let has_body = {
            if self.body.is_some() {
                "true"
            } else {
                "false"
            }
        };
        let can_run = self.input_options.iter().all(|x| x.values().len() > 0);
        let mut name = Cell::new(&self.name);
        if can_run {
            name = name.fg(Color::Green);
        }
        vec![
            name,
            Cell::new(self.method.to_string()),
            Cell::new(&self.url),
            Cell::new(Request::stringify_headers(&self.headers).unwrap_or(String::new())),
            Cell::new(has_body),
        ]
    }
}

fn variable_names(s: &str) -> HashSet<String> {
    // find all variables in the request
    // TODO: lazy static
    let re = Regex::new(r"\{(.*?)\}").unwrap();
    let names: Vec<String> = re
        .captures_iter(s)
        .map(|cap| String::from(cap.get(1).unwrap().as_str()))
        .collect();
    HashSet::from_iter(names.into_iter())
}
