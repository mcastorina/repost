use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{error::ReadlineError, CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use clap::App;
use shlex;

use crate::db::models::Environment;
use crate::db::Db;

use tokio::runtime::Handle;

use super::RootCmdCompleter;
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
        let mut tokens = shlex::split(&format!("{}_", &line[..pos])).unwrap_or_else(|| {
            // this may happen when there is an unbalanced quote
            format!("{}_", &line[..pos])
                .split(' ')
                .into_iter()
                // replace double with single quote because completions use single quote
                .map(|s| s.replace('"', "'"))
                .collect()
        });
        let mut last_token = tokens.pop().unwrap_or_default();
        // remove the extra character
        last_token.pop().unwrap();

        // find possible subcommands from App
        let mut app = &self.app;
        for tok in &tokens {
            match app.find_subcommand(tok) {
                None => continue,
                Some(child) => app = child,
            }
        }

        let candidates: Vec<String> =
            // try to parse the command
            if let Ok(cmd) = RootCmdCompleter::try_parse_from(&tokens) {
                if last_token.starts_with('-') {
                    // this is probably a flag or option, so we can get that from App
                    // TODO: better completions
                    app.get_opts().filter_map(|arg|
                        arg.get_long().map(|long|
                            format!("--{}", long)
                        ).or(arg.get_short().map(|short|
                            format!("-{}", short)
                        ))
                    ).collect()
                } else {
                    tokio::task::block_in_place(|| {
                        Handle::current().block_on(async { cmd.arg_candidates(&self.db).await })
                    })
                    .unwrap_or_default()
                }
            } else {
                // if the last word is an option
                let last_word = tokens.last();
                if last_word.filter(|s| s.starts_with('-')).is_some() {
                    // try again without the last word
                    if let Ok(cmd) = RootCmdCompleter::try_parse_from(&tokens[..tokens.len()-1]) {
                        tokio::task::block_in_place(|| {
                            Handle::current().block_on(async { cmd.opt_candidates(last_word.unwrap(), &self.db).await })
                        })
                        .unwrap_or_default()
                    } else {
                        todo!()
                    }
                } else {
                // otherwise, use the current app subcommand names
                app.get_subcommands()
                    .map(|x| x.get_name().to_string())
                    .collect()
                }
            };
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
