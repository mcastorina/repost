use std::io;
use std::io::prelude::*;
pub mod db;
use db::{Db, Request, Variable};

#[macro_use]
extern crate prettytable;
use prettytable::{format, Cell, Row, Table};

use reqwest::blocking;

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

    pub fn execute(&mut self, command: &str) -> Result<(), String> {
        // TODO: investigate using shlex
        let args: Vec<&str> = command.split_whitespace().collect();
        if args.len() == 0 {
            return Ok(());
        }
        if self.environment.is_some() {
            let ret = self.execute_environment(&args);
            if let Err(x) = ret {
                // TODO: error types
                if !x.starts_with("Invalid command: ") {
                    return Err(x);
                }
            } else {
                return ret;
            }
            self.execute_base(&args)?;
        }
        if self.environment.is_none() && self.request.is_none() {
            self.execute_base(&args)?;
        }
        Ok(())
    }

    fn update_prompt(&mut self) {
        let mut prompt = String::from("[repost]");
        if let Some(x) = &self.environment {
            prompt = format!("{}[{}]", prompt, x);
        }
        self.prompt = prompt;
    }

    fn execute_environment(&mut self, args: &Vec<&str>) -> Result<(), String> {
        match args[0] {
            "run" | "r" => self.execute_run(args),
            _ => Err(format!("Invalid command: {}", args[0])),
        }
    }
    fn execute_base(&mut self, args: &Vec<&str>) -> Result<(), String> {
        match args[0] {
            "show" | "get" => self.execute_show(args),
            "create" => self.execute_create(args),
            "use" | "set" => self.execute_use(args),
            x => Err(format!("Invalid command: {}.", x)),
        }
    }

    fn execute_run(&self, args: &Vec<&str>) -> Result<(), String> {
        // TODO run multiple in a row
        if args.len() != 2 {
            println!("Run a named HTTP request\n\nUsage: run <request>\n");
            return Ok(());
        }
        let req: Vec<db::Request> = self.db.get_requests()?.into_iter().filter(|x| x.name == args[1]).collect();
        if req.len() == 0 {
            return Err(format!("Request not found: {}", args[1]));
        }
        let req = &req[0];
        let client = blocking::Client::new();
        let mut builder;
        match req.method.as_ref() {
            "GET" => {
                builder = client.get(&req.url);
            },
            "POST" => {
                builder = client.post(&req.url);
            },
            "PUT" => {
                builder = client.put(&req.url);
            },
            "DELETE" => {
                builder = client.delete(&req.url);
            },
            "PATCH" => {
                builder = client.patch(&req.url);
            },
            x => {
                return Err(format!("Invalid method: {}", x))
            }
        }
        let resp = builder.send();
        println!("{:?}", resp);
        Ok(())
    }

    fn execute_show(&self, args: &Vec<&str>) -> Result<(), String> {
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

    fn execute_create(&self, args: &Vec<&str>) -> Result<(), String> {
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
    fn execute_use(&mut self, args: &Vec<&str>) -> Result<(), String> {
        if args.len() != 2 {
            println!("Use an environment\n\nUsage: use <environment>\n");
            return Ok(());
        }
        if !self.db.get_environments()?.iter().any(|x| x.environment == args[1]) {
            return Err(format!("Environment not found: {}", args[1]));
        }
        self.environment = Some(String::from(args[1]));
        self.update_prompt();
        Ok(())
    }
    fn create_request(&self, args: &Vec<&str>) -> Result<(), String> {
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
    fn create_variable(&self, args: &Vec<&str>) -> Result<(), String> {
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
