use crate::cmd::{Cmd, CmdError};
use crate::db::{Method, PrintableTable, Request, RequestOutput, Variable};
use crate::Repl;
use clap_v3::{App, ArgMatches, load_yaml};
use colored::*;
use comfy_table::{ContentArrangement, Table};
use reqwest::blocking;
use std::fs;
use terminal_size::{terminal_size, Width};

pub struct BaseCommand {}
impl Cmd for BaseCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        let matches = App::from(load_yaml!("cli.yml")).try_get_matches_from(args)?;

        match matches.subcommand() {
            ("create", Some(matches)) => match matches.subcommand() {
                ("request", Some(matches)) => BaseCommand::create_request(repl, matches),
                ("variable", Some(matches)) => BaseCommand::create_variable(repl, matches),
                _ => unreachable!(),
            },
            ("show", Some(matches)) => match matches.subcommand() {
                ("requests", _) => BaseCommand::print_table(repl.get_requests()?),
                ("variables", _) => BaseCommand::print_table(repl.get_variables()?),
                ("environments", _) => BaseCommand::print_table(repl.get_environments()?),
                ("options", _) => BaseCommand::print_table(repl.get_input_options()?),
                ("workspaces", _) => BaseCommand::print_table(repl.get_workspaces()?),
                _ => unreachable!(),
            },
            ("set", Some(matches)) => match matches.subcommand() {
                ("workspace", Some(matches)) => {
                    // TODO: use regex validator is YAML when available
                    //       https://github.com/clap-rs/clap/issues/1968
                    let ws = matches.value_of("workspace").unwrap();
                    if !ws.chars().all(char::is_alphanumeric) {
                        return Err(CmdError::ArgsError(String::from(
                            "only alphanumeric characters allowed",
                        )))
                    }
                    repl.update_workspace(matches.value_of("workspace").unwrap())
                }
                ("environment", Some(matches)) => {
                    repl.update_environment(matches.value_of("environment"))
                }
                ("request", Some(matches)) => repl.update_request(matches.value_of("request")),
                ("option", Some(matches)) => repl.set_option(
                    matches.value_of("option").unwrap(),
                    matches.value_of("value"),
                ),
                ("variable", Some(matches)) => BaseCommand::set_variable(repl, matches),
                _ => unreachable!(),
            },
            ("delete", Some(matches)) => match matches.subcommand() {
                ("requests", Some(matches)) => BaseCommand::delete_requests(repl, matches),
                ("variables", Some(matches)) => BaseCommand::delete_variables(repl, matches),
                _ => unreachable!(),
            },
            ("run", Some(matches)) => BaseCommand::execute_run(repl, matches),
            _ => Err(CmdError::NotFound),
        }
    }
}
impl BaseCommand {
    fn create_request(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        // We can unwrap because name and url are required
        let name = matches.value_of("name").unwrap();
        let url = matches.value_of("url").unwrap();
        let method: Option<Method>;
        // TODO
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
        // TODO: use regex validator is YAML when available
        //       https://github.com/clap-rs/clap/issues/1968
        for h in matches.values_of("headers").unwrap_or_default() {
            if !h.contains(':') {
                return Err(CmdError::ArgsError(format!(
                    "missing ':' in argument: {}",
                    h,
                )))
            }
        }
        let headers: Vec<(&str, &str)> = matches
            .values_of("headers")
            .unwrap_or_default()
            .map(|h| {
                let mut items = h.splitn(2, ":");
                // We can unwrap because this argument is guaranteed to have one ':'
                (items.next().unwrap().trim(), items.next().unwrap().trim())
            })
            .collect();
        method = {
            match matches.value_of("method") {
                Some(x) => Some(Method::new(x)),
                None => None,
            }
        };

        let mut request = Request::new(name, method, url);
        for header in headers {
            request.add_header(header.0, header.1);
        }

        request.set_body(body);
        // TODO: move these functions into repl
        repl.db.create_request(request)?;
        repl.update_options_for_request(name)?;
        Ok(())
    }
    fn create_variable(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let name = matches.value_of("name").unwrap();
        // verify all arguments contain an equal
        // TODO: use regex validator is YAML when available
        //       https://github.com/clap-rs/clap/issues/1968
        for ev in matches.values_of("environment=value").unwrap() {
            if !ev.contains('=') {
                return Err(CmdError::ArgsError(format!(
                    "missing '=' in argument: {}",
                    ev,
                )))
            }
        }
        let env_vals: Vec<(String, String)> = matches
            .values_of("environment=value")
            .unwrap()
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
            repl.db.create_variable(Variable {
                rowid: 0,
                name: String::from(name),
                environment,
                value: Some(value),
                source: Some(String::from("user")),
                timestamp: None,
            })?;
        }
        repl.update_options_for_variable(&name)?;
        Ok(())
    }
    fn set_variable(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let name = matches.value_of("name").unwrap();
        // TODO: use regex validator is YAML when available
        //       https://github.com/clap-rs/clap/issues/1968
        for ev in matches.values_of("environment=value").unwrap() {
            if !ev.contains('=') {
                return Err(CmdError::ArgsError(format!(
                    "missing '=' in argument: {}",
                    ev,
                )))
            }
        }
        let env_vals: Vec<(String, String)> = matches
            .values_of("environment=value")
            .unwrap()
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
            repl.db.upsert_variable(Variable {
                rowid: 0,
                name: String::from(name),
                environment,
                value: Some(value),
                source: Some(String::from("user")),
                timestamp: None,
            })?;
        }
        repl.update_options_for_variable(&name)?;
        Ok(())
    }
    fn delete_requests(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let requests: Vec<&str> = matches.values_of("request").unwrap().collect();
        for request in requests {
            // TODO: notify when not found
            repl.db.delete_request_by_name(request)?;
            repl.update_options_for_request(request)?;
        }
        Ok(())
    }
    fn delete_variables(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let vars: Vec<&str> = matches.values_of("variable").unwrap().collect();
        for var in vars {
            // TODO: notify when not found
            repl.db.delete_variable_by_name(var)?;
            repl.update_options_for_variable(var)?;
        }
        Ok(())
    }

    fn print_table<T: PrintableTable>(t: T) -> Result<(), CmdError> {
        // TODO: lazy static
        let mut width = 76;
        if let Some((Width(w), _)) = terminal_size() {
            width = w - 4;
        }
        let mut table = Table::new();
        table
            .load_preset(crate::TABLE_FORMAT)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_table_width(width);

        table.set_header(t.column_names());
        for row in t.rows() {
            table.add_row(row);
        }

        println!();
        for line in table.to_string().split('\n') {
            println!("  {}", line);
        }
        println!();

        Ok(())
    }
    fn execute_run(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let req = matches.value_of("request").unwrap();
        BaseCommand::run(repl, matches, req)
    }
    pub fn run(repl: &mut Repl, matches: &ArgMatches, req: &str) -> Result<(), CmdError> {
        let mut req = repl.db.get_request(req)?;
        // get options for this request
        let input_opts = repl
            .db
            .get_input_options()?
            .into_iter()
            .filter(|x| req.name() == x.request_name())
            .collect();
        let output_opts: Vec<RequestOutput> = repl
            .db
            .get_output_options()?
            .into_iter()
            .filter(|x| req.name() == x.request_name())
            .collect();
        // do option substitution
        // TODO: return result with missing options
        if !req.substitute_options(&input_opts) {
            return Err(CmdError::MissingOptions);
        }

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
        // (note: $LINES and $COLUMNS are not exported by default)
        //       pretty print JSON

        let v: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&text);
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
            let var = match opt.extraction_source() {
                "body" => repl.body_to_var(&opt, &text),
                "header" => repl.header_to_var(&opt, resp.headers()),
                x => {
                    println!("Encountered unexpected source: {}", x);
                    continue;
                }
            };
            if let Err(x) = var {
                println!("[!] {}", x);
                continue;
            }
            let mut var = var.unwrap_or_else(|_| unreachable!());
            var.source = Some(String::from(req.name()));
            if !quiet {
                println!(
                    "{}",
                    format!("{} => {}", var.name(), var.value().unwrap_or("")).bright_black()
                );
            }
            repl.db.upsert_variable(var)?;
            repl.update_options_for_variable(opt.option_name())?;
        }

        Ok(())
    }
}

fn create_reqwest(req: &mut Request) -> Result<blocking::Request, CmdError> {
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
