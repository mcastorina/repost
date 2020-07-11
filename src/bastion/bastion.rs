use super::completer::LineReader;
use crate::db::{self, Db, DbObject, InputOption, OutputOption, Request, Variable};
use crate::error::Result;
use colored::*;
use std::fs;

pub struct Bastion {
    state: ReplState,
    db: Db,
    line_reader: LineReader,
}

impl Bastion {
    pub fn new() -> Result<Bastion> {
        let mut bastion = Bastion {
            state: ReplState::Base(String::from("repost")),
            db: Db::new("repost.db")?,
            line_reader: LineReader::new(),
        };
        Ok(bastion)
    }

    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        // read the line
        self.line_reader.read_line(input, self.state.get_prompt())
    }

    pub fn execute(&mut self, command: &str) -> Result<()> {
        super::executer::execute(self, command)
    }

    pub fn state(&self) -> &ReplState {
        &self.state
    }

    pub fn get_requests(&self) -> Result<Vec<Request>> {
        Request::get_all(self.db.conn())
    }
    pub fn get_variables(&self) -> Result<Vec<Variable>> {
        Variable::get_all(self.db.conn())
    }
    pub fn get_environments(&self) -> Result<Vec<String>> {
        db::environment::get_all(self.db.conn())
    }
    pub fn get_input_options(&self) -> Result<Vec<InputOption>> {
        InputOption::get_all(self.db.conn())
    }
    pub fn get_output_options(&self) -> Result<Vec<OutputOption>> {
        OutputOption::get_all(self.db.conn())
    }
    pub fn get_workspaces(&self) -> Result<Vec<String>> {
        // TODO: option for config directory; set default to $XDG_CONFIG_DIR/repost
        let mut result = vec![];
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

pub enum ReplState {
    Base(String),
    Environment(String, String),
    Request(String, String),
    EnvironmentRequest(String, String, String),
}

impl ReplState {
    fn get_prompt(&self) -> String {
        match &self {
            ReplState::Base(ws) => format!("[{}] > ", ws.yellow()),
            ReplState::Environment(ws, env) => {
                format!("[{}][{}] > ", ws.yellow(), env.bold().cyan())
            }
            ReplState::Request(ws, req) => format!("[{}][{}] > ", ws.yellow(), req.bold().green()),
            ReplState::EnvironmentRequest(ws, env, req) => format!(
                "[{}][{}][{}] > ",
                ws.yellow(),
                env.bold().cyan(),
                req.bold().green()
            ),
        }
    }
}
