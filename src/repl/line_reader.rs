use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{error::ReadlineError, CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use clap::App;
use shlex;

pub struct LineReader {
    reader: Editor<CommandCompleter>,
}

impl LineReader {
    pub fn new(app: App<'static>) -> Self {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();

        let mut rl: Editor<CommandCompleter> = Editor::with_config(config);
        rl.set_helper(Some(CommandCompleter { app }));
        Self { reader: rl }
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
        let mut app = &self.app;
        for tok in tokens {
            let child = app.find_subcommand(&tok);
            if child.is_none() {
                return Ok((pos, vec![]));
            }
            app = child.unwrap();
        }
        // TODO: flags and db queries
        let candidates: Vec<&str> = app
            .get_subcommands()
            .map(|x| x.get_name())
            .filter(|x| x.starts_with(&last_token))
            .collect();
        Ok((
            pos - last_token.len(),
            candidates
                .into_iter()
                .map(|cmd| Pair {
                    display: String::from(cmd),
                    replacement: format!("{} ", cmd),
                })
                .collect(),
        ))
    }
}
