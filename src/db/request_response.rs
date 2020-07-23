use super::request::{Method, Request};
use super::PrintableTableStruct;
use super::{DbObject, InputOption};
use crate::error::{Error, ErrorKind, Result};
use comfy_table::Cell;
use regex::Regex;
use reqwest::blocking;
use rusqlite::{params, Connection, NO_PARAMS};
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct RequestResponse {
    rowid: u32,
    request_url: String,
    request_method: Method,
    request_headers: Option<String>,
    request_body: Option<Vec<u8>>,
    response_status: Option<String>,
    response_headers: Option<String>,
    response_body: Option<Vec<u8>>,
    response_extractions: Vec<(String, String)>,
}

impl RequestResponse {
    const VALUE_SEPARATOR: &'static str = "\n~\n";

    pub fn new(req: &blocking::Request) -> RequestResponse {
        RequestResponse {
            rowid: 0,
            request_url: format!("{}", req.url()),
            request_method: Method::new(req.method().as_str()),
            request_headers: Some(
                req.headers()
                    .iter()
                    .map(|x| format!("{}: {}", x.0, x.1.to_str().unwrap()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            request_body: req.body().map(|x| x.as_bytes().unwrap().to_vec()),

            response_status: None,
            response_headers: None,
            response_body: None,
            response_extractions: vec![],
        }
    }
    pub fn set_response(&mut self, resp: &blocking::Response, body: &Vec<u8>) {
        self.response_status = Some(format!("{}", resp.status()));
        self.response_headers = Some(
            resp.headers()
                .iter()
                .map(|x| format!("{}: {}", x.0, x.1.to_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        self.response_body = Some(body.clone());
    }
    pub fn add_extraction(&mut self, key: &str, value: &str) {
        self.response_extractions
            .push((String::from(key), String::from(value)));
    }

    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_responses (
                  rowid                 INTEGER PRIMARY KEY,
                  request_url           TEXT NOT NULL,
                  request_method        TEXT NOT NULL,
                  request_headers       TEXT,
                  request_body          BLOB,
                  response_status       TEXT,
                  response_headers      TEXT,
                  response_body         BLOB,
                  response_extractions  TEXT
              )",
            NO_PARAMS,
        )?;
        Ok(())
    }

    fn stringify_extractions(v: &Vec<(String, String)>) -> Option<String> {
        match v.len() {
            0 => None,
            _ => Some(
                v.iter()
                    .map(|x| format!("{} <= {}", x.0, x.1))
                    .collect::<Vec<_>>()
                    .join(RequestResponse::VALUE_SEPARATOR),
            ),
        }
    }
    fn unstringify_extractions(v: Option<String>) -> Vec<(String, String)> {
        match v {
            Some(v) => v
                .split(RequestResponse::VALUE_SEPARATOR)
                .map(|x| {
                    let mut iter = x.split(" <= ");
                    (
                        String::from(iter.next().unwrap()),
                        String::from(iter.next().unwrap()),
                    )
                })
                .collect(),
            None => vec![],
        }
    }
}

impl DbObject for RequestResponse {
    fn create(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO request_responses (
                    request_url,
                    request_method,
                    request_headers,
                    request_body,
                    response_status,
                    response_headers,
                    response_body,
                    response_extractions
                  )
              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            params![
                &self.request_url,
                &self.request_method.to_string(),
                &self.request_headers,
                &self.request_body,
                &self.response_status,
                &self.response_headers,
                &self.response_body,
                RequestResponse::stringify_extractions(&self.response_extractions),
            ],
        )?;
        Ok(())
    }
    fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "DELETE FROM request_responses WHERE rowid = ?1;",
            params![self.rowid],
        )?;
        Ok(())
    }
    fn update(&self, conn: &Connection) -> Result<usize> {
        // TODO: not supported
        Ok(0)
    }
    fn get_all(conn: &Connection) -> Result<Vec<RequestResponse>> {
        let mut stmt = conn.prepare(
            "SELECT 
                    rowid,
                    request_url,
                    request_method,
                    request_headers,
                    request_body,
                    response_status,
                    response_headers,
                    response_body,
                    response_extractions
                FROM request_responses;",
        )?;

        let req_resps = stmt.query_map(NO_PARAMS, |row| {
            let s: String = row.get(2)?;
            Ok(RequestResponse {
                rowid: row.get(0)?,
                request_url: row.get(1)?,
                request_method: Method::new(s.as_ref()),
                request_headers: row.get(3)?,
                request_body: row.get(4)?,
                response_status: row.get(5)?,
                response_headers: row.get(6)?,
                response_body: row.get(7)?,
                response_extractions: RequestResponse::unstringify_extractions(row.get(8)?),
            })
        })?;

        // TODO: print a warning for errors
        Ok(req_resps.filter_map(|req| req.ok()).collect())
    }
    fn name(&self) -> &str {
        ""
    }
}

impl PrintableTableStruct for RequestResponse {
    fn get_header() -> Vec<Cell> {
        vec![Cell::new("id"), Cell::new("request"), Cell::new("status")]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        vec![vec![
            Cell::new(&self.rowid),
            Cell::new(&self.request_url),
            Cell::new(self.response_status.as_ref().unwrap_or(&String::from("-"))),
        ]]
    }
}
