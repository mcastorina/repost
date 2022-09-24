use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{error::ReadlineError, CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use crate::db::Db;
use crate::error::Error;
use crate::repl::parser::{self, Completion};

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
        let line = &line[..pos];
        let (builder, completion) = match parser::parse_completion(line) {
            Ok(ok) => ok,
            Err(_) => return Ok((pos, vec![])),
        };
        // let (prefix, candidates) = match (builder, completion) {
        //     (_, None) => return Ok((pos, vec![])),
        //     (None, Some((prefix, completion)) => (prefix, completion.complete(prefix)),
        //     (Some(builder), Some((prefix, completion))) => (prefix, builder.smart_complete(prefix, completion)),
        // };
        let (prefix, candidates) = match completion {
            Some((prefix, Completion::Command(cmds))) => (
                prefix,
                cmds.iter().map(|cmd| cmd.completions()[0]).collect(),
            ),
            Some((prefix, Completion::OptKey)) => {
                let builder = builder.unwrap();
                (
                    prefix,
                    builder
                        .opts()
                        .iter()
                        .flat_map(|opt| opt.completions())
                        .copied()
                        .collect(),
                )
            }
            Some((prefix, completion)) => {
                let builder = builder.unwrap();
                // builder.smart_complete(completion, self.db)
                dbg!(builder, prefix, completion);
                (prefix, vec![])
            }
            None => ("", vec![]),
        };

        let candidates = candidates
            .into_iter()
            .filter(|cand| cand.starts_with(prefix))
            .map(|cand| Pair {
                display: cand.to_string(),
                replacement: format!("{} ", cand),
            })
            .collect();
        Ok((line.len() - prefix.len(), candidates))
    }
}
