mod config;
mod line_reader;
mod parser;

pub use config::ReplConfig;

use crate::db::Db;
use crate::error::{Error, Result};
use line_reader::LineReader;

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
        // build editor
        let mut editor = LineReader::new();
        editor.set_completer(&db);

        Ok(Self { conf, db, editor })
    }

    /// Read stdin into input using the LineReader.
    /// Returns None on EOF or error.
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.editor.read_line(input, "> ")
    }

    /// Execute a command line.
    pub async fn execute(&mut self, input: &str) -> Result<()> {
        todo!()
    }
}
