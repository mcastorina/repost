use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{error::ReadlineError, CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use crate::db::Db;
use crate::error::Error;
use crate::repl::parser;

pub struct LineReader {
    reader: Editor<CommandCompleter>,
}

impl LineReader {
    pub fn new() -> Self {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();

        Self {
            reader: Editor::with_config(config),
        }
    }

    pub fn set_completer(&mut self, db: &Db) {
        self.reader
            .set_helper(Some(CommandCompleter { db: db.clone() }));
    }

    pub fn read_line(&mut self, input: &mut String, prompt: &str) -> Option<()> {
        let readline = self.reader.readline(&prompt);
        match readline {
            Ok(line) => {
                self.reader.add_history_entry(line.as_str());
                *input = line;
                Some(())
            }
            Err(ReadlineError::Interrupted) => Some(()),
            Err(ReadlineError::Eof) => {
                // TODO: save history
                // self.reader
                //     .save_history(&self.history_filepath())
                //     .unwrap_or(());
                None
            }
            Err(_) => {
                // TODO: save history
                // self.reader
                //     .save_history(&self.history_filepath())
                //     .unwrap_or(());
                None
            }
        }
    }
}

#[derive(Helper, Validator, Highlighter, Hinter)]
struct CommandCompleter {
    db: Db,
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let (builder, (s, completion)) =
            parser::parse_completion(&line[..pos]).map_err(|_| ReadlineError::Interrupted)?;
        dbg!(builder, (s, completion));
        Ok((0, vec![]))
    }
}
