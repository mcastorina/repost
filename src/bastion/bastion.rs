use super::completer::LineReader;
use crate::db::{Db, DbObject, Environment, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use colored::*;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct Bastion {
    state: ReplState,
    db: Db,
    line_reader: LineReader,
}

impl Bastion {
    pub fn new(root: PathBuf) -> Result<Bastion> {
        let mut bastion = Bastion {
            state: ReplState::Base(String::from("repost")),
            db: Db::new(&root, "repost.db")?,
            line_reader: LineReader::new(&root),
        };
        Variable::set_all_options(bastion.conn(), bastion.environment())?;
        Ok(bastion)
    }
    pub fn conn(&self) -> &Connection {
        self.db.conn()
    }
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        // read the line
        self.line_reader.read_line(input, self.state.get_prompt())
    }

    pub fn execute(&mut self, command: &str) -> Result<()> {
        super::executer::execute(self, command)
        // self.set_completions()
    }

    pub fn state(&self) -> &ReplState {
        &self.state
    }
    pub fn environment(&self) -> Option<&str> {
        match &self.state {
            ReplState::Environment(_, env) | ReplState::EnvironmentRequest(_, env, _) => {
                Some(env.as_ref())
            }
            _ => None,
        }
    }
    pub fn request(&self) -> Result<Request> {
        match &self.state {
            ReplState::Request(_, req) | ReplState::EnvironmentRequest(_, _, req) => {
                Ok(Request::get_unique(self.conn(), req)?)
            }
            _ => Err(Error::new(ErrorKind::RequestStateExpected("Command"))),
        }
    }

    pub fn get_workspaces(&self) -> Result<Vec<String>> {
        self.db.get_dbs()
    }

    pub fn set_workspace(&mut self, workspace: &str) -> Result<()> {
        let ws = String::from(workspace);

        self.db.set_db(format!("{}.db", workspace).as_ref())?;
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
                    self.line_reader.set_base();
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
                    (true, false) => {
                        self.line_reader.set_base();
                        ReplState::Environment(ws, env)
                    }
                    (false, true) => ReplState::Request(ws, req),
                    (false, false) => {
                        self.line_reader.set_base();
                        ReplState::Base(ws)
                    }
                }
            }
        };
        Variable::set_all_options(self.conn(), self.environment())?;
        Ok(())
    }
    pub fn set_environment(&mut self, env: Option<&str>) -> Result<()> {
        if env.is_some() && !Environment::exists(self.db.conn(), env.unwrap())? {
            return Err(Error::new(ErrorKind::NotFound));
        }
        self.state.set_environment(env)?;
        Variable::set_all_options(self.conn(), self.environment())?;
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
    pub fn set_state(&mut self) -> Result<()> {
        if let Some(state) = match &self.state {
            ReplState::Environment(ws, env) => {
                let ws = String::from(ws);
                if Environment::exists(self.db.conn(), env)? {
                    None
                } else {
                    Some(ReplState::Base(ws))
                }
            }
            ReplState::Request(ws, req) => {
                let ws = String::from(ws);
                if Request::exists(self.db.conn(), req)? {
                    None
                } else {
                    self.line_reader.set_base();
                    Some(ReplState::Base(ws))
                }
            }
            ReplState::EnvironmentRequest(ws, env, req) => {
                let ws = String::from(ws);
                let env = String::from(env);
                let req = String::from(req);
                match (
                    Environment::exists(self.db.conn(), env.as_ref())?,
                    Request::exists(self.db.conn(), req.as_ref())?,
                ) {
                    (true, true) => None,
                    (true, false) => {
                        self.line_reader.set_base();
                        Some(ReplState::Environment(ws, env))
                    }
                    (false, true) => Some(ReplState::Request(ws, req)),
                    (false, false) => {
                        self.line_reader.set_base();
                        Some(ReplState::Base(ws))
                    }
                }
            }
            _ => None,
        } {
            self.state = state;
        }
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
