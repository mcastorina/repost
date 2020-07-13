use super::completer::LineReader;
use crate::db::{self, Db, DbObject, Environment, InputOption, OutputOption, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use colored::*;
use rusqlite::Connection;
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
        bastion.line_reader.environment_completions(
            bastion
                .get_environments()?
                .iter()
                .map(|x| String::from(x.name()))
                .collect(),
        );
        bastion.line_reader.request_completions(
            bastion
                .get_requests()?
                .iter()
                .map(|x| String::from(x.name()))
                .collect(),
        );
        bastion
            .line_reader
            .workspace_completions(bastion.get_workspaces()?);
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
    pub fn get_environments(&self) -> Result<Vec<Environment>> {
        Environment::get_all(self.db.conn())
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

    pub fn set_workspace(&mut self, workspace: &str) -> Result<()> {
        let ws = String::from(workspace);

        self.db = Db::new(format!("{}.db", workspace).as_ref())?;
        self.state = match &self.state {
            ReplState::Base(_) => ReplState::Base(ws),
            ReplState::Environment(_, env) => {
                let env = String::from(env);
                if Environment::exists(self.db.conn(), env.as_ref())? {
                    ReplState::Environment(ws, env)
                } else {
                    ReplState::Base(ws)
                }
            }
            ReplState::Request(_, req) => {
                let req = String::from(req);
                if Request::exists(self.db.conn(), req.as_ref())? {
                    ReplState::Request(ws, req)
                } else {
                    ReplState::Base(ws)
                }
            }
            ReplState::EnvironmentRequest(_, env, req) => {
                let env = String::from(env);
                let req = String::from(req);
                match (
                    Environment::exists(self.db.conn(), env.as_ref())?,
                    Request::exists(self.db.conn(), req.as_ref())?,
                ) {
                    (true, true) => ReplState::EnvironmentRequest(ws, env, req),
                    (true, false) => ReplState::Environment(ws, env),
                    (false, true) => ReplState::Request(ws, req),
                    (false, false) => ReplState::Base(ws),
                }
            }
        };
        // TODO: update options
        Ok(())
    }
    pub fn set_environment(&mut self, env: Option<&str>) -> Result<()> {
        if env.is_some() && !Environment::exists(self.db.conn(), env.unwrap())? {
            return Err(Error::new(ErrorKind::NotFound));
        }
        self.state.set_environment(env)?;
        Ok(())
    }
    pub fn set_request(&mut self, req: Option<&str>) -> Result<()> {
        if req.is_some() && !Request::exists(self.db.conn(), req.unwrap())? {
            return Err(Error::new(ErrorKind::NotFound));
        }
        self.state.set_request(req)?;
        match req {
            Some(_) => self.line_reader.set_request(),
            None => self.line_reader.set_base(),
        };
        Ok(())
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
    fn set_environment(&mut self, env: Option<&str>) -> Result<()> {
        if env.is_none() {
            match self {
                ReplState::Environment(ws, _) => {
                    *self = ReplState::Base(ws.clone());
                }
                ReplState::EnvironmentRequest(ws, _, req) => {
                    *self = ReplState::Request(ws.clone(), req.clone());
                }
                _ => (),
            };
            return Ok(());
        }
        let env = String::from(env.unwrap());
        match self {
            ReplState::Base(ws) | ReplState::Environment(ws, _) => {
                *self = ReplState::Environment(ws.clone(), env);
            }
            ReplState::Request(ws, req) | ReplState::EnvironmentRequest(ws, _, req) => {
                *self = ReplState::EnvironmentRequest(ws.clone(), env, req.clone());
            }
        }
        Ok(())
    }
    fn set_request(&mut self, req: Option<&str>) -> Result<()> {
        if req.is_none() {
            match self {
                ReplState::Request(ws, _) => {
                    *self = ReplState::Base(ws.clone());
                }
                ReplState::EnvironmentRequest(ws, env, _) => {
                    *self = ReplState::Environment(ws.clone(), env.clone());
                }
                _ => (),
            };
            return Ok(());
        }
        let req = String::from(req.unwrap());
        match self {
            ReplState::Base(ws) | ReplState::Request(ws, _) => {
                *self = ReplState::Request(ws.clone(), req);
            }
            ReplState::Environment(ws, env) | ReplState::EnvironmentRequest(ws, env, _) => {
                *self = ReplState::EnvironmentRequest(ws.clone(), env.clone(), req);
            }
        }
        Ok(())
    }
}
