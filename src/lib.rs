pub mod cmd;
pub mod db;

#[macro_use]
extern crate prettytable;

use cmd::{Cmd, CmdError};
use db::{Db, RequestOption, Variable};
use std::fs;
use std::io::{self, prelude::*};

pub struct Repl {
    prompt: String,
    workspace: String,
    db: Db,
    environment: Option<String>,
    request: Option<String>,
}

impl Repl {
    pub fn new() -> Result<Repl, CmdError> {
        let repl = Repl {
            prompt: String::from("[repost]"),
            workspace: String::from("repost"),
            db: Db::new("repost.db")?,
            environment: None,
            request: None,
        };
        repl.update_all_options()?;
        Ok(repl)
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

    fn cmds() -> Vec<Box<dyn Cmd>> {
        vec![
            Box::new(cmd::EnvironmentalCommand {}),
            Box::new(cmd::BaseCommand {}),
        ]
    }

    pub fn execute(&mut self, command: &str) -> Result<(), CmdError> {
        let args: Vec<String> = shlex::split(command).unwrap_or(vec![]);
        if args.len() == 0 {
            return Ok(());
        }
        let args = args.iter().map(|x| x.as_ref()).collect();
        for cmd in Repl::cmds() {
            let ret = cmd.execute(self, &args);
            match ret {
                Ok(x) => return Ok(x),
                Err(x) => match x {
                    CmdError::NotFound => (),
                    _ => return Err(x),
                },
            }
        }
        Err(CmdError::NotFound)
    }

    fn update_prompt(&mut self) {
        let mut prompt = format!("[{}]", &self.workspace);
        if let Some(x) = &self.environment {
            prompt = format!("{}[{}]", prompt, x);
        }
        self.prompt = prompt;
    }

    pub fn update_environment(&mut self, environment: &str) -> Result<(), CmdError> {
        if !self.db.environment_exists(environment)? {
            return Err(CmdError::ArgsError(format!(
                "Environment not found: {}",
                environment,
            )));
        }
        self.environment = Some(String::from(environment));
        self.update_all_options()?;
        self.update_prompt();
        Ok(())
    }

    pub fn update_workspace(&mut self, workspace: &str) -> Result<(), CmdError> {
        self.workspace = String::from(workspace);
        self.db = Db::new(format!("{}.db", workspace).as_ref())?;
        if let Some(environment) = self.environment.as_ref() {
            if !self.db.environment_exists(environment)? {
                self.environment = None;
                self.request = None;
            }
            // TODO: check request exists in new workspace
        }
        self.update_all_options()?;
        self.update_prompt();
        Ok(())
    }

    pub fn update_request(&mut self, request: &str) -> Result<(), CmdError> {
        Err(CmdError::NotImplemented)
    }

    fn update_all_options(&self) -> Result<(), CmdError> {
        // get all unique request_name in options table
        let request_names = self.db.get_unique_request_names_from_options()?;
        // call self.update_options_for_request(req)
        for name in request_names {
            self.update_options_for_request(name.as_ref())?;
        }
        Ok(())
    }
    fn update_options(&self, opts: Vec<RequestOption>) -> Result<(), CmdError> {
        if self.environment.is_none() {
            // if the current environment is none, clear the value
            for mut opt in opts {
                opt.update_value(None);
                self.db.update_option(opt)?;
            }
        } else {
            // else set option.value according to the environment
            for mut opt in opts {
                let mut var: Vec<Variable> = self
                    .db
                    .get_variables()?
                    .into_iter()
                    .filter(|var| {
                        var.environment() == self.environment().unwrap()
                            && var.name() == opt.option_name()
                    })
                    .collect();
                if var.len() == 0 {
                    opt.update_value(None);
                } else if var.len() == 1 {
                    let var = &mut var[0];
                    opt.update_value(var.consume_value());
                } else {
                    unreachable!();
                }
                self.db.update_option(opt)?;
            }
        }
        Ok(())
    }
    fn update_options_for_request(&self, request: &str) -> Result<(), CmdError> {
        // get all options for request
        let opts: Vec<RequestOption> = self
            .db
            .get_options()?
            .into_iter()
            .filter(|opt| opt.request_name() == request)
            .collect();
        self.update_options(opts)
    }
    fn update_options_for_variable(&self, variable: &str) -> Result<(), CmdError> {
        // get all opts where option_name == variable_name
        let opts: Vec<RequestOption> = self
            .db
            .get_options()?
            .into_iter()
            .filter(|opt| opt.option_name() == variable)
            .collect();
        self.update_options(opts)
    }

    fn environment(&self) -> Option<&str> {
        match &self.environment {
            Some(x) => Some(x.as_ref()),
            None => None,
        }
    }

    fn get_workspaces(&self) -> Result<Vec<String>, CmdError> {
        // TODO: use a struct if this is needed in other operations
        //       for now, it is only being used to print the workspaces
        //       so we prefix the vector with the header "workspace"
        let mut result = vec![String::from("workspace")];
        let paths = fs::read_dir("./")?;
        for path in paths {
            let path = path?.path();
            // filter out .db extensions
            match path.extension() {
                Some(x) => {
                    if x != "db" {
                        continue;
                    }
                }
                _ => continue,
            }
            let ws = path.file_stem().unwrap();
            if let Some(x) = ws.to_str() {
                result.push(String::from(x));
            }
        }
        Ok(result)
    }
}
