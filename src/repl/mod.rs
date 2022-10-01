mod config;
mod line_reader;
mod parser;

pub use config::ReplConfig;

use crate::cmd::Cmd;
use crate::db::models::Environment;
use crate::db::{Db, DisplayTable};
use crate::error::{Error, Result};
use line_reader::LineReader;
use parser::Command;

/// Repl object for handling readline editing, a database,
/// and executing commands.
pub struct Repl {
    conf: ReplConfig,
    editor: LineReader,
    state: ReplState,
}

#[derive(Clone)]
pub struct ReplState {
    db: Db,
    env: Option<Environment>,
}

impl Repl {
    /// Create a new Repl struct with a Db and LineReader struct.
    pub async fn new(conf: ReplConfig) -> Result<Self> {
        // start with an in-memory database
        let state = ReplState {
            db: Db::new_playground().await?,
            env: None,
        };

        Ok(Self {
            conf,
            editor: LineReader::new(),
            state,
        })
    }

    /// Read stdin into input using the LineReader.
    /// Returns None on EOF or error.
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        // TODO: use a read-only reference to self.state
        self.editor.set_completer(self.state.clone());
        self.editor.read_line(input, self.state.prompt().as_ref())
    }

    /// Execute a command line.
    pub async fn execute(&mut self, input: &str) -> Result<()> {
        let cmd = Cmd::new(&self.state.db);
        match parser::parse_command(input)? {
            Command::CreateRequest(args) => cmd.create_request(args.try_into()?).await?,
            Command::CreateVariable(args) => cmd.create_variable(args.try_into()?).await?,
            Command::PrintRequests(_) => cmd.print_requests().await?,
            Command::PrintVariables(_) => cmd.print_variables().await?,
            Command::PrintEnvironments(_) => cmd.print_environments().await?,
            Command::PrintWorkspaces(_) => self.workspaces()?.print_with_header(&["workspaces"]),
            Command::SetEnvironment(args) => {
                self.state
                    .set_environment(args.environment.map(Environment::from))
                    .await?
            }
        }
        Ok(())
    }

    fn workspaces(&self) -> Result<Vec<String>> {
        Ok(self
            .conf
            .dbs()?
            .into_iter()
            .map(|db| Db::name_of(&db.to_string_lossy()).to_owned())
            .collect())
    }
}

impl ReplState {
    fn prompt(&self) -> String {
        let db = self.db.name();
        match &self.env {
            Some(env) => format!("[{}][{}] > ", db, env.name),
            None => format!("[{}] > ", db),
        }
    }

    async fn set_environment(&mut self, env: Option<Environment>) -> Result<()> {
        if env.is_none() {
            self.env = None;
            return Ok(());
        }
        let cand = env.as_ref().unwrap();
        let _: bool = sqlx::query_scalar("SELECT 1 FROM variables WHERE env = ? LIMIT 1")
            .bind(cand.as_ref())
            .fetch_one(self.db.pool())
            .await?;
        self.env = env;
        Ok(())
    }
}
