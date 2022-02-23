use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{error::ReadlineError, CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use clap::App;
use shlex;

use crate::db::models::Environment;
use crate::db::Db;

use tokio::runtime::Handle;

use super::Command;
use clap::Parser;

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

    pub fn set_completer(&mut self, app: App<'static>, db: &Db) {
        self.reader.set_helper(Some(CommandCompleter {
            app,
            db: db.clone(),
        }));
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
    app: App<'static>,
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
        // split line; add an extra character at the end to not ignore trailing whitespace
        let mut tokens = shlex::split(&format!("{}_", &line[..pos])).unwrap_or_default();
        let mut last_token = tokens.pop().unwrap_or_default();
        // remove the extra character
        last_token.pop().unwrap();

        // recurse through subcommands
        // try to parse the command
        let candidates: Vec<String> = if let Ok(cmd) = Command::try_parse_from(tokens.clone()) {
            // cmd.arg_candidates(repl)
            tokio::task::block_in_place(|| {
                Handle::current().block_on(async { cmd.arg_candidates(&self.db).await })
            })
            .unwrap_or_default()
        } else {
            // otherwise find possible subcommands from App
            let mut app = &self.app;
            for tok in tokens {
                // TODO: pattern match
                let child = app.find_subcommand(&tok);
                if child.is_none() {
                    continue;
                }
                app = child.unwrap();
            }
            app.get_subcommands()
                .map(|x| x.get_name().to_string())
                .collect()
        };
        // TODO: flags and db queries
        // use message passing to get a result?
        {
            let envs = tokio::task::block_in_place(|| {
                Handle::current().block_on(async {
                    let got: Vec<Environment> = sqlx::query_as("SELECT * FROM environments")
                        .fetch_all(self.db.pool())
                        .await
                        .expect("could not get");
                    got
                })
            });
            // dbg!(envs);
        }
        // TODO:
        // * check if the last_token starts with --
        // * check if the last token starts with -
        // * check if the last token starts with empty
        //   * check for positional args
        //   * check for options
        // * delegate to sub-functions for special casing
        //   * name will autocomplete to 'get-' 'create-' etc. (or names from DB)
        //   * url will autocomplete to 'https://' (or urls from DB)
        //   * --method will complete to case-insensitive GET, POST, etc.
        //   * --header will complete to a list of common headers
        // dbg!(app.get_opts().collect::<Vec<_>>());
        Ok((
            pos - last_token.len(),
            candidates
                .into_iter()
                .filter(|x| x.starts_with(&last_token))
                .map(|cmd| Pair {
                    replacement: format!("{} ", cmd),
                    display: cmd,
                })
                .collect(),
        ))
    }
}
