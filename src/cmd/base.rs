use crate::cmd::{Cmd, CmdError};
use crate::db::{Method, PrintableTable, Request, Variable};
use crate::Repl;
use clap_v3::{App, AppSettings, Arg, ArgMatches};
use prettytable::{format, Table};

pub struct BaseCommand {}
impl Cmd for BaseCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        // TODO: can this be a sinlge static clap_v3::App variable?
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
        let show_requests = App::new("requests")
            .about("Print requests")
            .aliases(&["request", "reqs", "req", "r"]);
        let show_variables = App::new("variables")
            .about("Print variables")
            .aliases(&["variable", "vars", "var", "v"]);
        let show_environments = App::new("environments")
            .about("Print environments")
            .aliases(&["environment", "envs", "env", "e"]);
        let show_options = App::new("options")
            .about("Print options")
            .aliases(&["option", "opts", "opt", "o"]);
        let show_workspaces =
            App::new("workspaces")
                .about("Print workspaces")
                .aliases(&["workspace", "ws", "w"]);
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
        let delete_requests = App::new("requests")
            .about("Delete the named HTTP requests")
            .aliases(&["request", "reqs", "req", "r"])
            .arg(
                Arg::with_name("request")
                    .help("Request to delete")
                    .required(true)
                    .multiple(true),
            );
        let delete_variables = App::new("variables")
            .about("Delete the named variables")
            .aliases(&["variable", "vars", "var", "v"])
            .arg(
                Arg::with_name("variable")
                    .help("Variable to delete")
                    .required(true)
                    .multiple(true),
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
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .about("Print resources")
                    .aliases(&["get", "print", "g", "p"])
                    .subcommand(show_requests)
                    .subcommand(show_variables)
                    .subcommand(show_environments)
                    .subcommand(show_options)
                    .subcommand(show_workspaces),
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
            .subcommand(
                App::new("delete")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .about("Delete named requests or variables")
                    .aliases(&["remove", "del", "rm"])
                    .subcommand(delete_requests)
                    .subcommand(delete_variables),
            )
            .try_get_matches_from(args)?;

        match matches.subcommand() {
            ("create", Some(matches)) => match matches.subcommand() {
                ("request", Some(matches)) => BaseCommand::create_request(repl, matches),
                ("variable", Some(matches)) => BaseCommand::create_variable(repl, matches),
                _ => unreachable!(),
            },
            ("show", Some(matches)) => match matches.subcommand() {
                ("requests", _) => BaseCommand::print_table(repl.db.get_requests()?),
                ("variables", _) => BaseCommand::print_table(repl.db.get_variables()?),
                ("environments", _) => BaseCommand::print_table(repl.db.get_environments()?),
                ("options", _) => BaseCommand::print_table(repl.db.get_options()?),
                ("workspaces", _) => BaseCommand::print_table(repl.get_workspaces()?),
                _ => unreachable!(),
            },
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
            ("delete", Some(matches)) => match matches.subcommand() {
                ("requests", Some(matches)) => BaseCommand::delete_requests(repl, matches),
                ("variables", Some(matches)) => BaseCommand::delete_variables(repl, matches),
                _ => unreachable!(),
            },
            _ => unreachable!(),
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
        let body = matches.value_of("data").map(|b| b.as_bytes().to_vec());
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
