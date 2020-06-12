use prettytable::{format, Table};
use reqwest::blocking;

use crate::Repl;
use crate::db::{DbError, Method, Request, Variable, PrintableTable};

pub enum CmdError {
    DbError(DbError),
    ArgsError(String),
    NotFound,
    NotImplemented,
}

pub trait Cmd {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError>;
}

pub struct BaseCommand{}
impl Cmd for BaseCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        match args[0].to_lowercase().as_ref() {
            "show" | "get" => BaseCommand::execute_show(repl, args),
            "create" => BaseCommand::execute_create(repl, args),
            "use" | "set" => BaseCommand::execute_use(repl, args),
            _ => Err(CmdError::NotFound),
        }
    }
}

impl BaseCommand {

    fn execute_show(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        if args.len() != 2 {
            println!("Show various saved data\n\nUsage: show <requests|variables|environments>\n");
            return Ok(());
        }
        match args[1].to_lowercase().as_ref() {
            "r" | "req" | "reqs" | "request" | "requests" => {
                BaseCommand::print_table(repl.db.get_requests()?)
            }
            "v" | "var" | "vars" | "variable" | "variables" => {
                BaseCommand::print_table(repl.db.get_variables()?)
            }
            "e" | "env" | "envs" | "environment" | "environments" => {
                BaseCommand::print_table(repl.db.get_environments()?)
            }
            _ => Err(CmdError::ArgsError(format!(
                "Invalid argument: {}",
                args[1]
            ))),
        }
    }

    fn execute_create(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        if args.len() < 2 {
            println!("Create various data\n\nUsage: create <request|variable> args...\n");
            return Ok(());
        }
        match args[1] {
            "request" | "req" => BaseCommand::create_request(repl, args),
            "variable" | "var" => BaseCommand::create_variable(repl, args),
            // TODO: print usage
            _ => Err(CmdError::ArgsError(format!(
                "Invalid argument to create: {}",
                args[2]
            ))),
        }
    }
    fn execute_use(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        if args.len() != 2 {
            println!("Use an environment\n\nUsage: use <environment>\n");
            return Ok(());
        }
        if !repl
            .db
            .get_environments()?
            .iter()
            .any(|x| x.environment == args[1])
        {
            return Err(CmdError::ArgsError(format!(
                "Environment not found: {}",
                args[1]
            )));
        }
        repl.environment = Some(String::from(args[1]));
        repl.update_prompt();
        Ok(())
    }
    fn create_request(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        // TODO: support method, header and body
        //       use clap
        if args.len() < 4 {
            return Err(CmdError::ArgsError(String::from(
                "Usage: create request name url [-m method] [-H header] [-d body]",
            )));
        }
        // TODO: infer method from name
        repl.db
            .create_request(Request::new(args[2], None, args[3]))?;
        Ok(())
    }
    fn create_variable(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        // TODO: use clap; verify arguments contain an =
        if args.len() < 4 {
            return Err(CmdError::ArgsError(String::from(
                "Usage: create variable name env=value",
            )));
        }
        let name = String::from(args[2]);
        for arg in &args[3..] {
            // TODO: create a new variable function
            let mut items = arg.splitn(2, "=");
            let environment = String::from(items.next().unwrap());
            let value = Some(String::from(items.next().unwrap()));
            repl.db.create_variable(Variable {
                rowid: 0,
                name: name.clone(),
                environment,
                value,
                source: Some(String::from("user")),
                timestamp: None,
            })?;
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


pub struct EnvironmentalCommand{}
impl Cmd for EnvironmentalCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        match args[0].to_lowercase().as_ref() {
            "run" | "r" => EnvironmentalCommand::execute_run(repl, args),
            _ => Err(CmdError::NotFound),
        }
    }
}
impl EnvironmentalCommand {
    fn execute_run(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        // TODO run multiple in a row
        if args.len() != 2 {
            println!("Run a named HTTP request\n\nUsage: run <request>\n");
            return Ok(());
        }
        let req: Vec<Request> = repl
            .db
            .get_requests()?
            .into_iter()
            .filter(|x| x.name() == args[1])
            .collect();
        if req.len() == 0 {
            return Err(CmdError::ArgsError(format!(
                "Request not found: {}",
                args[1]
            )));
        }
        let req = &req[0];
        let client = blocking::Client::new();
        let builder = match req.method() {
            Method::GET => client.get(req.url()),
            Method::POST => client.post(req.url()),
            Method::PUT => client.put(req.url()),
            Method::PATCH => client.patch(req.url()),
            Method::DELETE => client.delete(req.url()),
            Method::HEAD => client.head(req.url()),
        };
        let resp = builder.send();
        println!("{:?}", resp);
        Ok(())
    }
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CmdError::DbError(x) => match x {
                DbError::Rusqlite(x) => write!(f, "{}", x),
            },
            CmdError::ArgsError(x) => write!(f, "{}", x),
            CmdError::NotFound => write!(f, "Command not found."),
            CmdError::NotImplemented => write!(f, "Command not implemented."),
        }
    }
}
impl From<CmdError> for String {
    fn from(err: CmdError) -> String {
        match err {
            CmdError::DbError(x) => match x {
                DbError::Rusqlite(x) => format!("{}", x),
            },
            CmdError::ArgsError(x) => x,
            CmdError::NotFound => String::from("Command not found."),
            CmdError::NotImplemented => String::from("Command not implemented."),
        }
    }
}
impl From<DbError> for CmdError {
    fn from(err: DbError) -> CmdError {
        CmdError::DbError(err)
    }
}
