use crate::bastion::Bastion;
use crate::db::{DbObject, Method, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use std::fs;

pub fn request(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    // TODO: add validator to yaml once available
    let headers = matches.values_of("headers").unwrap_or_default();
    if !headers.clone().all(|s| s.contains(':')) {
        return Err(Error::new(ErrorKind::ArgumentError(
            "Found argument that does not contain ':'",
        )));
    }

    // We can unwrap because name and url are required
    let name = matches.value_of("name").unwrap();
    let url = matches.value_of("url").unwrap();
    let method: Option<Method>;
    let body = match matches.value_of("data") {
        Some(x) => {
            if x.starts_with('@') {
                let mut filename = x.chars();
                filename.next(); // discard @
                Some(fs::read(filename.collect::<String>())?)
            } else {
                Some(x.as_bytes().to_vec())
            }
        }
        None => None,
    };
    let headers: Vec<(&str, &str)> = headers
        .map(|h| {
            let mut items = h.splitn(2, ":");
            // We can unwrap because this argument is guaranteed to have one ':'
            (items.next().unwrap().trim(), items.next().unwrap().trim())
        })
        .collect();
    method = matches.value_of("method").map(|x| Method::new(x));

    let mut request = Request::new(name, method, url);
    for header in headers {
        request.add_header(header.0, header.1);
    }

    request.set_body(body);
    request.create(b.conn())?;
    // TODO: update options for request
    Ok(())
}

pub fn variable(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let name = matches.value_of("name").unwrap();
    let env_vals = matches.values_of("environment=value").unwrap();
    // TODO: add validator to yaml once available
    if !env_vals.clone().all(|s| s.contains('=')) {
        return Err(Error::new(ErrorKind::ArgumentError(
            "Found argument that does not contain '='",
        )));
    }
    let env_vals: Vec<(String, String)> = env_vals
        .map(|s| {
            let mut items = s.splitn(2, "=");
            // We can unwrap because this argument is guaranteed to have one '='
            (
                String::from(items.next().unwrap()),
                String::from(items.next().unwrap()),
            )
        })
        .collect();

    for env_val in env_vals {
        // TODO: create a new variable function
        let (environment, value) = env_val;
        Variable::new(name, &environment, Some(&value), Some("user")).create(b.conn())?;
    }
    // TODO update options for variable
    Ok(())
}
