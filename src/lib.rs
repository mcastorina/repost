use std::io;
use std::io::prelude::*;
pub mod db;
use db::{Db, Request, Variable};

#[macro_use]
extern crate prettytable;
use prettytable::{format, Cell, Row, Table};

pub struct Repl {
    prompt: String,
    workspace: String,
    db: Db,
    environment: Option<String>,
    request: Option<String>,
}

impl Repl {
    pub fn new() -> Result<Repl, String> {
        Ok(Repl {
            prompt: String::from("[repost]"),
            workspace: String::from("repost"),
            db: Db::new("repost.db")?,
            environment: None,
            request: None,
        })
    }

    pub fn get_input(&self, mut input: &mut String) -> Option<()> {
        let stdin = io::stdin();

        print!("{} > ", self.prompt);
        io::stdout().flush().unwrap();
        input.clear();

        // read line and exit on EOF
        if stdin.read_line(&mut input).unwrap() == 0 {
            println!("goodbye");
            return None;
        }
        // remove trailing newline
        input.pop();
        Some(())
    }

    pub fn execute(&self, command: &str) -> Result<(), String> {
        // TODO: investigate using shlex
        let args: Vec<&str> = command.split_whitespace().collect();
        if args.len() == 0 {
            return Ok(());
        }
        if self.environment == None && self.request == None {
            self.execute_base(args)
        } else {
            Ok(())
        }
    }

    fn execute_base(&self, args: Vec<&str>) -> Result<(), String> {
        match args[0] {
            "show" | "get" => self.execute_show(args),
            "create" => self.execute_create(args),
            "use" | "set" => self.execute_use(args),
            x => Err(format!("Invalid command: {}.", x)),
        }
    }

    fn execute_show(&self, args: Vec<&str>) -> Result<(), String> {
        if args.len() != 2 {
            println!("Show various saved data\n\nUsage: show <requests|variables|environments>\n");
            return Ok(());
        }
        match args[1].to_lowercase().as_ref() {
            "r" | "req" | "reqs" | "request" | "requests" => {
                self.print_table(self.db.get_requests()?)
            }
            "v" | "var" | "vars" | "variable" | "variables" => {
                self.print_table(self.db.get_variables()?)
            }
            "e" | "env" | "envs" | "environment" | "environments" => {
                self.print_table(self.db.get_environments()?)
            }
            _ => Err(format!("Invalid argument: {}", args[1])),
        }
    }

    fn print_table<T: db::PrintableTable>(&self, t: T) -> Result<(), String> {
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

    fn execute_create(&self, args: Vec<&str>) -> Result<(), String> {
        if args.len() < 2 {
            println!("Create various data\n\nUsage: create <request|variable> args...\n");
            return Ok(());
        }
        match args[1] {
            "request" | "req" => self.create_request(args),
            "variable" | "var" => self.create_variable(args),
            // TODO: print usage
            _ => Err(format!("Invalid argument to create: {}", args[2])),
        }
    }
    fn execute_use(&self, args: Vec<&str>) -> Result<(), String> {
        Err(String::from("not implemented: use"))
    }
    fn create_request(&self, args: Vec<&str>) -> Result<(), String> {
        // TODO: support method, header and body
        //       use clap
        if args.len() < 4 {
            return Err(String::from(
                "Usage: create request name url [-m method] [-H header] [-d body]",
            ));
        }
        // TODO: infer method from name
        self.db.create_request(Request::new(args[2], None, args[3]))
    }
    fn create_variable(&self, args: Vec<&str>) -> Result<(), String> {
        // TODO: use clap; verify arguments contain an =
        if args.len() < 4 {
            return Err(String::from("Usage: create variable name env=value"));
        }
        let name = String::from(args[2]);
        for arg in &args[3..] {
            // TODO: create a new variable function
            let mut items = arg.splitn(2, "=");
            let environment = String::from(items.next().unwrap());
            let value = Some(String::from(items.next().unwrap()));
            self.db.create_variable(Variable {
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

    fn get_table_from_alias(alias: &str) -> Option<String> {
        match alias {
            "r" | "req" | "reqs" | "request" | "requests" => Some(String::from("requests")),
            "v" | "var" | "vars" | "variable" | "variables" => Some(String::from("variables")),
            "e" | "env" | "envs" | "environment" | "environments" => {
                Some(String::from("environments"))
            }
            _ => None,
        }
    }
}
