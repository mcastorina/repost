use crate::cmd::{Cmd, CmdError};
use crate::db::{Method, Request, RequestOutput};
use crate::Repl;
use clap_v3::{App, AppSettings, Arg, ArgMatches};
use colored::*;
use reqwest::blocking;

pub struct ContextualCommand {}
impl Cmd for ContextualCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        let matches = clap_args().try_get_matches_from(args)?;
        match matches.subcommand() {
            ("run", Some(matches)) => ContextualCommand::execute_run(repl, matches),
            ("extract", Some(matches)) => ContextualCommand::extract(repl, matches),
            _ => Err(CmdError::NotFound),
        }
    }
}
impl ContextualCommand {
    fn execute_run(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let req = matches.value_of("request").unwrap();
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
            let mut var = match opt.extraction_source() {
                "body" => repl.body_to_var(&opt, &text)?,
                "header" => repl.hader_to_var(&opt, resp.headers())?,
                x => {
                    println!("Encountered unexpected source: {}", x);
                    continue;
                }
            };
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

    fn extract(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        if repl.request().is_none() {
            return Err(CmdError::ArgsError(String::from("Extract is only available in a request specific context. Try setting a request first.")));
        }
        let request = repl.request().unwrap();
        let extraction_source = matches.value_of("type").unwrap();
        let key = matches.value_of("key").unwrap();
        let var = matches.value_of("variable").unwrap();

        let opt = RequestOutput::new(request, var, extraction_source, key);
        repl.db.create_output_option(opt)?;
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
fn clap_args() -> clap_v3::App<'static> {
    // TODO: can this be a sinlge static clap_v3::App variable?
    App::new("repost")
        .setting(AppSettings::NoBinaryName)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .subcommand(
            App::new("run")
                .about("Run a named HTTP request")
                .aliases(&["r"])
                .arg(
                    Arg::with_name("request")
                        .help("Request to run")
                        .required(true)
                        .multiple(false), // TODO run multiple in a row
                )
                .arg(
                    Arg::with_name("quiet")
                        .help("Verbose output")
                        .short('q')
                        .long("quiet")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("extract")
                .about("Extract data from the output of a request")
                .aliases(&["ex"])
                .arg(
                    Arg::with_name("type")
                        .help("Body or header extraction")
                        .required(true)
                        .possible_values(&["body", "header"]),
                )
                .arg(
                    Arg::with_name("key")
                        .help("Key to extract - header name or JSON body path")
                        .required(true),
                )
                .arg(
                    Arg::with_name("variable")
                        .help("Variable to store the extracted data")
                        .short('t')
                        .long("to-var")
                        .takes_value(true)
                        .required(true),
                ),
        )
}
