mod command;
mod config;
mod line_reader;

pub use config::ReplConfig;

use crate::db::Db;
use crate::error::{Error, Result};
use command::Command;
use line_reader::LineReader;

use clap::{IntoApp, Parser};

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
        // build db
        let path = conf.data_dir.join("repost.db");
        let db = Db::new(path.to_str().ok_or(Error::ConfigDataToStr)?).await?;
        // build app for editor completions
        let mut app = Command::into_app();
        app._build_all();
        // build editor
        let mut editor = LineReader::new();
        editor.set_completer(app, &db);

        Ok(Self { conf, db, editor })
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
        cmd.execute(self).await
    }
}
