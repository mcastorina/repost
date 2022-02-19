mod command;
mod line_reader;

use crate::db;
use crate::db::Db;
use clap::{IntoApp, Parser};
use command::{Cmd, Command, PrintCmd};
use line_reader::LineReader;

use crate::error::Result;

/// Repl object for handling readline editing, a database,
/// and executing commands.
pub struct Repl {
    db: Db,
    editor: LineReader,
}

impl Repl {
    /// Create a new Repl struct with a Db and LineReader struct.
    pub async fn new() -> Result<Self> {
        let mut app = Command::into_app();
        app._build_all();
        let db = Db::new("/tmp/repost.db").await?;
        let mut editor = LineReader::new();
        editor.set_completer(app, &db);
        Ok(Self { editor, db })
    }

    /// Read stdin into input using the LineReader.
    /// Returns None on EOF or error.
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.editor.read_line(input, "> ")
    }

    /// Execute a command line.
    pub async fn execute(&mut self, input: &str) -> Result<()> {
        let args = shlex::split(input).unwrap_or_default();
        // TODO: this may not always be a Command struct (for context-aware commands)
        let cmd = Command::try_parse_from(args)?;
        cmd.execute(&self.db).await
    }
}
