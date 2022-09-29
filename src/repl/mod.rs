mod config;
mod line_reader;
mod parser;

pub use config::ReplConfig;

use crate::cmd::Cmd;
use crate::db::{Db, DisplayTable};
use crate::error::{Error, Result};
use line_reader::LineReader;
use parser::Command;

/// Repl object for handling readline editing, a database,
/// and executing commands.
pub struct Repl {
    conf: ReplConfig,
    db: Db,
    editor: LineReader,
}

impl Repl {
    /// Create a new Repl struct with a Db and LineReader struct.
    pub async fn new(conf: ReplConfig) -> Result<Self> {
        // start with an in-memory database
        let db = Db::new_playground().await?;
        // build editor
        let mut editor = LineReader::new();
        editor.set_completer(&db);

        Ok(Self { conf, db, editor })
    }

    /// Read stdin into input using the LineReader.
    /// Returns None on EOF or error.
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.editor.read_line(input, self.prompt().as_ref())
    }

    /// Execute a command line.
    pub async fn execute(&mut self, input: &str) -> Result<()> {
        let cmd = Cmd::new(&self.db);
        match parser::parse_command(input).map_err(|_| Error::ParseError("foo"))? {
            Command::CreateRequest(args) => cmd.create_request(args.try_into()?).await?,
            Command::CreateVariable(args) => cmd.create_variable(args.try_into()?).await?,
            Command::PrintRequests(_) => cmd.print_requests().await?,
            Command::PrintVariables(_) => cmd.print_variables().await?,
            Command::PrintEnvironments(_) => cmd.print_environments().await?,
            Command::PrintWorkspaces(_) => self.workspaces()?.print_with_header(&["workspaces"]),
        }
        Ok(())
    }

    fn prompt(&self) -> String {
        format!("[{}] > ", self.db.name())
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
