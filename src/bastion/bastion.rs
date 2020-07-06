use super::completer::LineReader;
use crate::db::Db;
use crate::error::Result;
use colored::*;

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
        // set the prompt and completer
        Ok(())
    }
}

enum ReplState {
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
