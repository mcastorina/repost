use super::{DbObject, PrintableTableStruct};
use crate::error::{Error, ErrorKind, Result};
use comfy_table::{Attribute, Cell, Color};
use reqwest::blocking;
use reqwest::Method;
use rusqlite::{params, Connection, NO_PARAMS};

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
            request_method: req.method().clone(),
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

    pub fn url(&self) -> &str {
        self.request_url.as_ref()
    }
    pub fn method(&self) -> String {
        self.request_method.to_string()
    }
    pub fn request_headers(&self) -> Vec<&str> {
        match &self.request_headers {
            None => vec![],
            Some(headers) => headers.split("\n").collect(),
        }
    }
    pub fn request_body(&self) -> Option<&Vec<u8>> {
        self.request_body.as_ref()
    }
    pub fn status(&self) -> Option<&str> {
        self.response_status.as_deref()
    }
    pub fn response_headers(&self) -> Vec<&str> {
        match &self.response_headers {
            None => vec![],
            Some(headers) => headers.split("\n").collect(),
        }
    }
    pub fn response_body(&self) -> Option<&Vec<u8>> {
        self.response_body.as_ref()
    }
    pub fn extractions(&self) -> Vec<(&str, &str)> {
        self.response_extractions
            .iter()
            .map(|x| (x.0.as_ref(), x.1.as_ref()))
            .collect()
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

    pub fn delete_all(conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM request_responses;", NO_PARAMS)?;
        Ok(())
    }
    pub fn get_by_id(conn: &Connection, id: u32) -> Result<RequestResponse> {
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
                FROM request_responses WHERE rowid = ?1;",
        )?;

        let req_resps = stmt.query_map(params![id], |row| {
            let s: String = row.get(2)?;
            Ok(RequestResponse {
                rowid: row.get(0)?,
                request_url: row.get(1)?,
                request_method: Method::from_bytes(s.as_bytes()).unwrap_or(Method::GET),
                request_headers: row.get(3)?,
                request_body: row.get(4)?,
                response_status: row.get(5)?,
                response_headers: row.get(6)?,
                response_body: row.get(7)?,
                response_extractions: RequestResponse::unstringify_extractions(row.get(8)?),
            })
        })?;

        // TODO: print a warning for errors
        let mut v: Vec<_> = req_resps.filter_map(|req| req.ok()).collect();
        if v.len() == 0 {
            Err(Error::new(ErrorKind::NotFound))
        } else {
            Ok(v.remove(0))
        }
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
    fn update(&self, _conn: &Connection) -> Result<usize> {
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
                request_method: Method::from_bytes(s.as_bytes()).unwrap_or(Method::GET),
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
        let status = self.response_status.clone().unwrap_or(String::from("-"));
        let status = match status.chars().next().unwrap_or('-') {
            '2' => Cell::new(status).fg(Color::Green),
            '3' => Cell::new(status).fg(Color::Yellow),
            '4' => Cell::new(status).fg(Color::Red),
            '5' => Cell::new(status)
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
            _ => Cell::new(status),
        };
        vec![vec![
            Cell::new(&self.rowid),
            Cell::new(format!(
                "{} {}",
                self.request_method.to_string(),
                self.request_url
            )),
            status,
        ]]
    }
}
