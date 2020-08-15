use crate::bastion::Bastion;
use crate::db::{DbObject, Request, RequestResponse, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use colored::*;
use regex::Regex;
use reqwest::blocking;
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn execute(b: &mut Bastion, matches: &ArgMatches, req: Option<&str>) -> Result<()> {
    let req = match req {
        Some(s) => Request::get_unique(b.conn(), s)?,
        None => b.request()?,
    };
    // TODO: parse arguments and pass as options to create_requests()

    let mut runner = req.create_requests()?;
    println!("{:?}", runner);
    runner.run()
}

fn get_json_values(root: &Value, query: &str) -> Result<Vec<Value>> {
    let mut v: Value = root.clone();
    let mut result: &mut Value = &mut v;

    let re = Regex::new(r"\[(\d+|\*)\]")?;
    let tokens = query.split(".");
    for token in tokens.clone() {
        let name = token.splitn(2, "[").next().unwrap();
        let mr = result.get_mut(name);
        if mr.is_none() {
            return Err(Error::new(ErrorKind::ParseError));
        }
        result = mr.unwrap();
        for cap in re.captures_iter(token) {
            let index = &cap[1];
            if index == "*" {
                let mut tokens = tokens.skip_while(|x| x != &token);
                tokens.next();
                let rest = tokens.collect::<Vec<&str>>().join(".");

                let mut results = vec![];
                if !result.is_array() {
                    return Err(Error::new(ErrorKind::ParseError));
                }
                for value in result.as_array().unwrap() {
                    results.extend(get_json_values(value, &rest)?);
                }
                return Ok(results);
            } else {
                let num: usize = index.parse()?;
                let mr = result.get_mut(num);
                if mr.is_none() {
                    return Err(Error::new(ErrorKind::ParseError));
                }
                result = mr.unwrap();
            }
        }
    }
    Ok(vec![result.take()])
}

pub fn display_body(text: &str, no_pager: bool) {
    let v: serde_json::Result<serde_json::Value> = serde_json::from_str(text);
    let text = match v {
        Ok(v) => format!("{}\n", serde_json::to_string_pretty(&v).unwrap()),
        _ => String::from(text),
    };

    let mut used_pager = false;
    if !no_pager && text.lines().count() > 80 {
        // try to invoke $PAGER
        // TODO: support args in $PAGER
        if let Ok(pager) = env::var("PAGER") {
            if let Ok(mut child) = Command::new(pager).stdin(Stdio::piped()).spawn() {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                used_pager = true;
            }
        }
    }
    if !used_pager {
        print!("{}", text);
        if !(text).ends_with('\n') {
            println!("{}", "%".bold().reversed());
        }
    }
}
