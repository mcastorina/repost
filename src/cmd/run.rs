use crate::bastion::Bastion;
use crate::db::{DbObject, InputOption, Method, OutputOption, Request, RequestResponse, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use colored::*;
use regex::Regex;
use reqwest::blocking;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn execute(b: &mut Bastion, matches: &ArgMatches, req: Option<&str>) -> Result<()> {
    let req = req.or(b.current_request());
    if req.is_none() {
        return Err(Error::new(ErrorKind::NotFound));
    }
    // get the request object
    let mut req = Request::get_by_name(b.conn(), req.unwrap())?;
    match req.len() {
        0 => return Err(Error::new(ErrorKind::NotFound)),
        1 => (),
        _ => unreachable!(),
    };
    let mut req = req.remove(0);

    // get options for this request
    let input_opts = InputOption::get_by_name(b.conn(), req.name())?;
    let output_opts = OutputOption::get_by_name(b.conn(), req.name())?;

    // if this request has extractions, check if there is an environment
    if output_opts.len() > 0 && b.current_environment().is_none() {
        return Err(Error::new(ErrorKind::ArgumentError(
            "The request contains extractions and must be ran from an environment.",
        )));
    }

    // modify the request object given run arguments
    if let Some(data) = matches.values_of("data") {
        for data in data {
            if req.body().is_none() {
                req.add_query_param(data);
            } else {
                // TODO: make this a common function
                let data = if data.starts_with('@') {
                    let mut filename = data.chars();
                    filename.next(); // discard @
                    Some(fs::read(filename.collect::<String>())?)
                } else {
                    Some(data.as_bytes().to_vec())
                };
                req.set_body(data);
            }
        }
    }

    // create all request objects given input options
    let requests = create_requests(&req, &input_opts)?;

    // delete extractions
    for opt in output_opts.iter() {
        for var in Variable::get_by(b.conn(), |v| {
            v.name() == opt.option_name() && v.source().unwrap_or("") == req.name()
        })? {
            var.delete(b.conn())?;
        }
    }

    RequestResponse::delete_all(b.conn())?;
    let quiet = matches.is_present("quiet");
    let many_requests = requests.len() > 1;
    for mut req in requests {
        let reqw = create_reqwest(&mut req)?;

        if !quiet {
            println!(
                "{}",
                format!("> {} {}", reqw.method(), reqw.url()).bright_black()
            );
            for header in reqw.headers() {
                let (name, value) = header;
                println!(
                    "{}",
                    format!("> {}: {}", name, value.to_str().unwrap()).bright_black()
                );
            }
            println!();
        }

        let mut rr = RequestResponse::new(&reqw);
        let mut resp = blocking::Client::new().execute(reqw)?;

        // output response code and headers
        if !quiet {
            println!("{}", format!("< {}", resp.status()).bright_black());
            for header in resp.headers() {
                let (name, value) = header;
                println!(
                    "{}",
                    format!("< {}: {}", name, value.to_str().unwrap()).bright_black()
                );
            }
            println!();
        }

        // output body with missing-newline indicator
        let mut text: Vec<u8> = vec![];
        resp.copy_to(&mut text)?;
        rr.set_response(&resp, &text);
        let text = String::from_utf8(text).unwrap();

        display_body(&text, many_requests || matches.is_present("no-pager"));
        if output_opts.len() > 0 {
            println!();
        }

        // extract options into variables
        for opt in output_opts.iter() {
            let vars = match opt.extraction_type() {
                "body" => body_to_vars(&opt, &text, b.current_environment().unwrap()),
                "header" => header_to_vars(&opt, resp.headers(), b.current_environment().unwrap()),
                x => {
                    println!("Encountered unexpected source: {}.", x);
                    continue;
                }
            };
            if let Err(x) = vars {
                println!("[!] {}", x);
                continue;
            }
            for var in vars.unwrap().iter_mut() {
                var.set_source(Some(req.name()));
                rr.add_extraction(var.name(), var.value().unwrap_or(""));
                if !quiet {
                    println!(
                        "{}",
                        format!("{} <= {}", var.name(), var.value().unwrap_or("")).bright_black()
                    );
                }
                var.create(b.conn())?;
                b.set_options(InputOption::get_by(b.conn(), |x| {
                    x.option_name() == var.name()
                })?)?;
            }
        }
        rr.create(b.conn())?;
    }

    if many_requests {
        println!("\n  Summary");
        super::show::print_table(RequestResponse::get_all(b.conn())?)?;
        println!();
    }

    b.set_completions()?;
    Ok(())
}

pub fn create_requests(req: &Request, input_opts: &Vec<InputOption>) -> Result<Vec<Request>> {
    let missing_opts: Vec<_> = input_opts
        .iter()
        .filter(|opt| opt.values().len() == 0)
        .map(|opt| String::from(opt.option_name()))
        .collect();
    if missing_opts.len() > 0 {
        // All input options are required
        return Err(Error::new(ErrorKind::MissingOptions(missing_opts)));
    }

    if input_opts.len() == 0 {
        let mut req = req.clone();
        req.replace_input_options(&input_opts)?;
        return Ok(vec![req]);
    }

    let mut requests = Vec::new();
    let opts: Vec<_> = input_opts.iter().map(|opt| opt.values()).collect();

    for i in 0..opts.iter().map(|x| x.len()).max().unwrap_or(0) {
        let opt_values: Vec<&str> = opts.iter().map(|v| v[i % v.len()]).collect();
        let mut opts = input_opts.clone();
        let mut req = req.clone();
        for (opt, opt_value) in opts.iter_mut().zip(opt_values) {
            opt.set_value(Some(opt_value));
        }
        req.replace_input_options(&opts)?;
        requests.push(req);
    }
    Ok(requests)
}

fn create_reqwest(req: &mut Request) -> Result<blocking::Request> {
    // TODO: should this be a method of Request?
    let client = blocking::Client::new();
    let mut builder = match req.method() {
        Method::GET => client.get(req.url()),
        Method::POST => client.post(req.url()),
        Method::PUT => client.put(req.url()),
        Method::PATCH => client.patch(req.url()),
        Method::DELETE => client.delete(req.url()),
        Method::HEAD => client.head(req.url()),
    };
    // add headers
    if let Some(x) = req.headers() {
        for hv in x.split('\n') {
            let mut items = hv.splitn(2, ":");
            let (header, value) = (items.next(), items.next());
            if header.and(value).is_none() {
                continue;
            }
            builder = builder.header(header.unwrap(), value.unwrap());
        }
    }
    // add body
    if let Some(x) = req.consume_body() {
        builder = builder.body(x);
    }

    Ok(builder.build()?)
}
fn body_to_vars(opt: &OutputOption, body: &str, env: &str) -> Result<Vec<Variable>> {
    let values = get_json_values(&serde_json::from_str(body)?, opt.extraction_source())?;
    Ok(values
        .into_iter()
        .map(|value| Variable::new(opt.option_name(), env, value.as_str(), None))
        .collect())
}
fn header_to_vars(opt: &OutputOption, headers: &HeaderMap, env: &str) -> Result<Vec<Variable>> {
    let value = headers
        .get(opt.extraction_source())
        .map(|x| x.to_str().unwrap());
    if value.is_none() {
        return Err(Error::new(ErrorKind::ParseError));
    }
    // TODO: handle multiple headers
    Ok(vec![Variable::new(opt.option_name(), env, value, None)])
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

fn display_body(text: &str, no_pager: bool) {
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
