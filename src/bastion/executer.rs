use super::bastion::{Bastion, ReplState};
use crate::cmd::{create, delete, extract, info, run, set, show};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::{load_yaml, App};

pub fn execute(b: &mut Bastion, command: &str) -> Result<()> {
    let args: Vec<String> = shlex::split(command).unwrap_or(vec![]);
    let args = args.iter().map(|x| x.as_ref()).collect();
    execute_args(b, args)
}

pub fn execute_args(b: &mut Bastion, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        return Ok(());
    }
    let base_yaml = load_yaml!("clap/base.yml");
    let request_yaml = load_yaml!("clap/request.yml");
    let app = match b.state() {
        ReplState::Base(_) | ReplState::Environment(_, _) => App::from(base_yaml),
        ReplState::Request(_, _) | ReplState::EnvironmentRequest(_, _, _) => {
            App::from(request_yaml)
        }
    };
    let matches = app.try_get_matches_from(args)?;
    match matches.subcommand() {
        ("create", Some(matches)) => match matches.subcommand() {
            ("request", Some(matches)) => create::request(b, matches),
            ("variable", Some(matches)) => create::variable(b, matches),
            _ => unreachable!(),
        },
        ("show", Some(matches)) => match matches.subcommand() {
            ("requests", Some(matches)) => show::requests(b, matches),
            ("variables", Some(matches)) => show::variables(b, matches),
            ("options", Some(matches)) => show::options(b, matches),
            ("environments", Some(_)) => show::environments(b, matches),
            ("workspaces", Some(_)) => {
                show::print_table((String::from("workspace"), b.get_workspaces()?));
                Ok(())
            }
            ("response", Some(matches)) => show::response(b, matches),
            _ => unreachable!(),
        },
        ("set", Some(matches)) => match matches.subcommand() {
            ("workspace", Some(matches)) => {
                // TODO: add validator to yaml once available
                let ws = matches.value_of("workspace").unwrap();
                if !ws.chars().all(char::is_alphanumeric) {
                    return Err(Error::new(ErrorKind::ArgumentError(
                        "Only alphanumeric characters allowed.",
                    )));
                }
                b.set_workspace(ws)
            }
            ("environment", Some(matches)) => b.set_environment(matches.value_of("environment")),
            ("request", Some(matches)) => b.set_request(matches.value_of("request")),
            ("option", Some(matches)) => b.set_option(
                matches.value_of("option").unwrap(),
                matches.values_of("value").unwrap_or_default().collect(),
            ),
            ("variable", Some(matches)) => set::variable(b, matches),
            _ => unreachable!(),
        },
        ("delete", Some(matches)) => match matches.subcommand() {
            ("requests", Some(matches)) => delete::requests(b, matches),
            ("variables", Some(matches)) => delete::variables(b, matches),
            ("options", Some(matches)) => delete::options(b, matches),
            _ => unreachable!(),
        },
        ("run", Some(matches)) => run::execute(b, matches, matches.value_of("request")),
        ("extract", Some(matches)) => extract::execute(b, matches),
        ("info", Some(matches)) => info::execute(b, matches),
        _ => Err(Error::new(ErrorKind::NotFound)),
    }
}
