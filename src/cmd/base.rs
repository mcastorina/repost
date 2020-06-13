use crate::cmd::{Cmd, CmdError};
use crate::db::{Method, PrintableTable, Request, Variable};
use crate::Repl;
use clap_v3::{App, AppSettings, Arg, ArgMatches};
use prettytable::{format, Table};

pub struct BaseCommand {}
impl Cmd for BaseCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        let contains_equal = |val: String| {
            // val is the argument value passed in by the user
            if val.contains("=") {
                Ok(())
            } else {
                Err(CmdError::ArgsError(format!(
                    "missing '=' in argument: {}",
                    val
                )))
            }
        };
        let contains_colon = |val: String| {
            // val is the argument value passed in by the user
            if val.contains(":") {
                Ok(())
            } else {
                Err(CmdError::ArgsError(format!(
                    "missing ':' in argument: {}",
                    val
                )))
            }
        };
        let is_alphanumeric = |val: String| {
            if val.chars().all(char::is_alphanumeric) {
                Ok(())
            } else {
                Err(CmdError::ArgsError(String::from(
                    "only alphanumeric characters are allowed",
                )))
            }
        };
        let create_variable = App::new("variable")
            .about("Create a variable")
            .aliases(&["var", "v"])
            .arg("<name> 'Name of the variable'")
            .arg(
                Arg::with_name("environment=value")
                    .help("Value for environment")
                    .required(true)
                    .validator(contains_equal)
                    .multiple(true),
            );
        let create_request = App::new("request")
            .about("Create an HTTP request")
            .aliases(&["req", "r"])
            .arg("<name> 'Name of the request'")
            .arg("<url> 'HTTP request URL'")
            .arg(
                Arg::with_name("method")
                    .help("HTTP request method")
                    .short('m')
                    .long("method")
                    .possible_values(&["GET", "POST", "HEAD", "PUT", "PATCH", "DELETE"]),
            )
            .arg(
                Arg::with_name("headers")
                    .help("HTTP request headers")
                    .short('H')
                    .long("header")
                    .validator(contains_colon)
                    .multiple(true),
            )
            .arg("-d, --data=[DATA] 'HTTP request body'");
        let set_environment = App::new("environment")
            .about("Set the environment as used for variable substitution")
            .aliases(&["env", "e"])
            .arg("<environment> 'Environment to use'");
        let set_request = App::new("request")
            .about("Set the request to view and modify specific options")
            .aliases(&["req", "r"])
            .arg("<request> 'Request to use'");
        let set_workspace = App::new("workspace")
            .about("Set the workspace where all data is stored")
            .aliases(&["ws", "w"])
            .arg(
                Arg::with_name("workspace")
                    .help("Workspace to use")
                    .required(true)
                    .validator(is_alphanumeric),
            );
        let matches = App::new("repost")
            .setting(AppSettings::NoBinaryName)
            .subcommand(
                App::new("create")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .about("Create an HTTP request or variable")
                    .aliases(&["new", "c"])
                    .subcommand(create_request)
                    .subcommand(create_variable),
            )
            .subcommand(
                // TODO: use subcommand for show_requests show_variables show_environments
                App::new("show")
                    .about("Print resources")
                    .aliases(&["get", "print", "g", "p"])
                    .arg(
                        Arg::with_name("resource")
                            .help("Resource to show")
                            .required(true)
                            .case_insensitive(true)
                            .possible_values(&[
                                "requests",
                                "variables",
                                "environments",
                                "options",
                                "request",
                                "variable",
                                "environment",
                                "option",
                                "reqs",
                                "vars",
                                "envs",
                                "opts",
                                "req",
                                "var",
                                "env",
                                "opt",
                                "r",
                                "v",
                                "e",
                                "o",
                            ]),
                    ),
            )
            .subcommand(
                App::new("set")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .about(
                        "Set workspace, environment, or request for environment specific commands",
                    )
                    .aliases(&["use", "load", "u"])
                    .subcommand(set_workspace)
                    .subcommand(set_environment)
                    .subcommand(set_request),
            )
            .try_get_matches_from(args)?;

        match matches.subcommand() {
            ("create", Some(matches)) => match matches.subcommand() {
                ("request", Some(matches)) => BaseCommand::create_request(repl, matches),
                ("variable", Some(matches)) => BaseCommand::create_variable(repl, matches),
                _ => unreachable!(),
            },
            ("show", Some(matches)) => BaseCommand::show(repl, matches),
            ("set", Some(matches)) => match matches.subcommand() {
                ("workspace", Some(matches)) => {
                    repl.update_workspace(matches.value_of("workspace").unwrap())
                }
                ("environment", Some(matches)) => {
                    repl.update_environment(matches.value_of("environment").unwrap())
                }
                ("request", Some(matches)) => {
                    repl.update_request(matches.value_of("request").unwrap())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl BaseCommand {
    fn show(repl: &mut Repl, args: &ArgMatches) -> Result<(), CmdError> {
        let resource = args.value_of("resource").unwrap();
        match resource.to_lowercase().as_ref() {
            "r" | "req" | "reqs" | "request" | "requests" => {
                BaseCommand::print_table(repl.db.get_requests()?)
            }
            "v" | "var" | "vars" | "variable" | "variables" => {
                BaseCommand::print_table(repl.db.get_variables()?)
            }
            "e" | "env" | "envs" | "environment" | "environments" => {
                BaseCommand::print_table(repl.db.get_environments()?)
            }
            "o" | "opt" | "opts" | "option" | "options" => {
                BaseCommand::print_table(repl.db.get_options()?)
            }
            _ => unreachable!(),
        }
    }

    fn create_request(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        // We can unwrap because name and url are required
        let name = matches.value_of("name").unwrap();
        let url = matches.value_of("url").unwrap();
        let method: Option<Method>;
        // TODO
        // let body = matches.value_of("data").map(|b| String::from(b));
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

        // TODO: body
        // TODO: move these functions into repl
        repl.db.create_request(request)?;
        repl.update_options_for_request(name)?;
        Ok(())
    }
    fn create_variable(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        let name = matches.value_of("name").unwrap();
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

    fn print_table<T: PrintableTable>(t: T) -> Result<(), CmdError> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.get_format().indent(2);

        table.set_titles(t.column_names());
        for row in t.rows() {
            table.add_row(row);
        }
        println!();
        table.printstd();
        println!();
        Ok(())
    }
}
