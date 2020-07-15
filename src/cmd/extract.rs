use crate::bastion::Bastion;
use crate::db::{DbObject, InputOption, Method, OutputOption, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use colored::*;
use regex::Regex;
use reqwest::blocking;
use reqwest::header::HeaderMap;
use serde_json::Value;

pub fn execute(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    if b.current_request().is_none() {
        return Err(Error::new(ErrorKind::ArgumentError(
            "Extract is only available in a request specific context. Try setting a request first.",
        )));
    }
    let request = b.current_request().unwrap();
    let extraction_source = matches.value_of("type").unwrap();
    let key = matches.value_of("key").unwrap();
    let var = matches.value_of("variable").unwrap();

    let opt = OutputOption::new(request, var, extraction_source, key);
    opt.create(b.conn())?;
    Ok(())
}