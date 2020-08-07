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
    headers: Option<String>,
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
            headers: None, // TODO
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
    pub fn get_unique(conn: &Connection, name: &str) -> Result<Request> {
        todo!();
    }
    pub fn exists(conn: &Connection, name: &str) -> Result<bool> {
        todo!();
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
        if let Some(headers) = &self.headers {
            let headers = variable_names(headers);
            names.extend(headers);
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
    pub fn add_extraction(&mut self, opt: OutputOption) -> Result<()> {
        todo!();
    }
    pub fn input_options(&self) -> &Vec<InputOption> {
        todo!();
    }
    pub fn output_options(&self) -> &Vec<InputOption> {
        todo!();
    }
    pub fn create_requests(&self) -> Vec<RequestRunner> {
        todo!();
    }
    pub fn delete_option(&mut self) -> Result<()> {
        todo!();
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
                self.headers,
                self.body
            ],
        )?;
        for option in self.input_options.iter() {
            option.update(conn)?;
        }
        Ok(num)
    }
    fn get_all(conn: &Connection) -> Result<Vec<Request>> {
        let mut stmt = conn.prepare("SELECT name, method, url, headers, body FROM requests;")?;

        let requests = stmt.query_map(NO_PARAMS, |row| {
            let name: String = row.get(0)?;
            let input_opts = InputOption::get_by(conn, |i| i.option_name() == &name);
            let output_opts = OutputOption::get_by(conn, |o| o.option_name() == &name);
            // TODO: error checking
            Ok(Request {
                name,
                method: Method::from_bytes(row.get::<_, String>(1)?.as_bytes())
                    .unwrap_or(Method::GET),
                url: row.get(2)?,
                headers: row.get(3)?,
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
            Cell::new(self.headers.as_ref().unwrap_or(&String::from(""))),
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
