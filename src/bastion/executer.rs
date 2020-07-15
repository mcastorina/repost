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
            ("requests", Some(matches)) => show::print_table(matches, b.get_requests()?),
            ("variables", Some(matches)) => show::print_table(matches, b.get_variables()?),
            ("environments", Some(matches)) => show::print_table(matches, b.get_environments()?),
            ("options", Some(matches)) => {
                show::print_table(matches, b.get_input_options()?)?;
                show::print_table(matches, b.get_output_options()?)?;
                Ok(())
            }
            ("workspaces", Some(matches)) => {
                show::print_table(matches, (String::from("workspace"), b.get_workspaces()?))
            }
            _ => unreachable!(),
        },
        ("set", Some(matches)) => match matches.subcommand() {
            ("workspace", Some(matches)) => b.set_workspace(matches.value_of("workspace").unwrap()),
            ("environment", Some(matches)) => b.set_environment(matches.value_of("environment")),
            ("request", Some(matches)) => b.set_request(matches.value_of("request")),
            ("option", Some(matches)) => b.set_option(
                matches.value_of("option").unwrap(),
                matches.value_of("value"),
            ),
            ("variable", Some(matches)) => set::variable(b, matches),
            _ => unreachable!(),
        },
        ("delete", Some(matches)) => match matches.subcommand() {
            ("requests", Some(matches)) => delete::requests(b, matches),
            ("variables", Some(matches)) => delete::variables(b, matches),
            _ => unreachable!(),
        },
        // TODO: run in request state
        ("run", Some(matches)) => run::execute(b, matches, matches.value_of("request")),
        ("extract", Some(matches)) => extract::execute(b, matches),
        ("info", Some(matches)) => info::execute(b, matches),
        _ => Err(Error::new(ErrorKind::NotFound)),
    }
}
