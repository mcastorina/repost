use crate::bastion::Bastion;
use crate::db::{DbObject, InputOption, Method, OutputOption, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use colored::*;
use regex::Regex;
use reqwest::blocking;
use reqwest::header::HeaderMap;
use serde_json::Value;

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

    // do option substitution
    // TODO: return result with missing options
    req.replace_input_options(&input_opts)?;

    let reqw = create_reqwest(&mut req)?;
    let quiet = matches.is_present("quiet");

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
    let text = String::from_utf8(text).unwrap();

    // TODO: invoke $PAGER if length > $LINES
    let v: serde_json::Result<serde_json::Value> = serde_json::from_str(&text);
    if v.is_ok() {
        let v = v.unwrap();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        print!("{}", text);
        if !(&text).ends_with('\n') {
            println!("{}", "%".bold().reversed());
        }
    }
    if output_opts.len() > 0 {
        println!();
    }

    // extract options into variables
    for opt in output_opts {
        let var = match opt.extraction_type() {
            "body" => body_to_var(&opt, &text, b.current_environment()),
            "header" => header_to_var(&opt, resp.headers(), b.current_environment()),
            x => {
                println!("Encountered unexpected source: {}", x);
                continue;
            }
        };
        if let Err(x) = var {
            println!("[!] {}", x);
            continue;
        }
        let mut var = var.unwrap();
        // TODO: set source
        var.set_source(Some(req.name()));
        if !quiet {
            println!(
                "{}",
                format!("{} => {}", var.name(), var.value().unwrap_or("")).bright_black()
            );
        }
        var.upsert(b.conn())?;
        b.set_options(InputOption::get_by(b.conn(), |x| {
            x.option_name() == var.name()
        })?)?;
    }
    b.set_completions()?;
    Ok(())
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
fn body_to_var(opt: &OutputOption, body: &str, env: Option<&str>) -> Result<Variable> {
    let value = get_json_value(body, opt.extraction_source())?;
    Ok(Variable::new(
        opt.option_name(),
        env.unwrap_or(""), // TODO: allow None for environment
        value.as_str(),
        None,
    ))
}
fn header_to_var(opt: &OutputOption, headers: &HeaderMap, env: Option<&str>) -> Result<Variable> {
    let value = headers
        .get(opt.extraction_source())
        .map(|x| x.to_str().unwrap());
    if value.is_none() {
        return Err(Error::new(ErrorKind::ParseError));
    }
    Ok(Variable::new(
        opt.option_name(),
        env.unwrap_or(""), // TODO: allow None for environment
        value,
        None,
    ))
}
fn get_json_value(data: &str, query: &str) -> Result<Value> {
    let mut v: Value = serde_json::from_str(data)?;
    let mut result: &mut Value = &mut v;

    let re = Regex::new(r"\[(\d+)\]")?;
    for token in query.split(".") {
        let name = token.splitn(2, "[").next().unwrap();
        let mr = result.get_mut(name);
        if mr.is_none() {
            return Err(Error::new(ErrorKind::ParseError));
        }
        result = mr.unwrap();
        for cap in re.captures_iter(token) {
            let num: usize = cap[1].parse()?;
            let mr = result.get_mut(num);
            if mr.is_none() {
                return Err(Error::new(ErrorKind::ParseError));
            }
            result = mr.unwrap();
        }
    }
    Ok(result.take())
}
