pub mod db;
pub mod cmd;

#[macro_use]
extern crate prettytable;

use std::io::{self,prelude::*};
use db::Db;
use cmd::{Cmd,CmdError};

pub struct Repl {
    prompt: String,
    _workspace: String,
    db: Db,
    environment: Option<String>,
    request: Option<String>,
}

impl Repl {
    pub fn new() -> Result<Repl, CmdError> {
        Ok(Repl {
            prompt: String::from("[repost]"),
            _workspace: String::from("repost"),
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

    fn cmds() -> Vec<Box<dyn Cmd>> {
        vec![
            Box::new(cmd::EnvironmentalCommand{}),
            Box::new(cmd::BaseCommand{}),
        ]
    }

    pub fn execute(&mut self, command: &str) -> Result<(), CmdError> {
        // TODO: investigate using shlex
        let args: Vec<&str> = command.split_whitespace().collect();
        if args.len() == 0 {
            return Ok(());
        }
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
        let mut prompt = String::from("[repost]");
        if let Some(x) = &self.environment {
            prompt = format!("{}[{}]", prompt, x);
        }
        self.prompt = prompt;
    }
}
