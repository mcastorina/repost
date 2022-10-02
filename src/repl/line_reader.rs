use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{
    error::ReadlineError, CompletionType, Config, Context, EditMode, Editor,
    Result as ReadlineResult,
};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use crate::error::{Error, Result};
use crate::repl::parser::{self, ArgKey, Builder, Completion, OptKey};
use crate::repl::ReplState;
use std::iter;
use tokio::runtime::Handle;

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

    pub fn set_completer(&mut self, state: ReplState) {
        self.reader.set_helper(Some(CommandCompleter { state }));
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
    state: ReplState,
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> ReadlineResult<(usize, Vec<Self::Candidate>)> {
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
                cmds.iter()
                    .filter_map(|cmd| {
                        cmd.completions()
                            .iter()
                            .filter(|cand| cand.starts_with(prefix))
                            .next()
                    })
                    // TODO: only allocate after filtering results
                    .copied()
                    .map(String::from)
                    .collect(),
            ),
            Some((prefix, Completion::OptKey)) => {
                let builder = builder.unwrap();
                (
                    prefix,
                    builder
                        .opts()
                        .iter()
                        .flat_map(|opt| opt.completions())
                        // TODO: only allocate after filtering results
                        .copied()
                        .map(String::from)
                        .collect(),
                )
            }
            Some((prefix, completion)) => (
                prefix,
                tokio::task::block_in_place(|| {
                    Handle::current().block_on(async {
                        self.smart_complete(prefix, builder.unwrap(), completion)
                            .await
                    })
                })
                .unwrap_or_default(),
            ),
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

impl CommandCompleter {
    async fn smart_complete(
        &self,
        prefix: &str,
        builder: Builder,
        completion: Completion,
    ) -> Result<Vec<String>> {
        match builder {
            Builder::SetEnvironmentBuilder(_) => self.set_environment(completion).await,
            Builder::SetWorkspaceBuilder(_) => self.set_workspace(completion).await,
            _ => Err(Error::ParseError("not implemented")),
        }
    }

    async fn set_workspace(&self, completion: Completion) -> Result<Vec<String>> {
        let candidates = match completion {
            Completion::Arg(ArgKey::Name) => self.state.workspaces()?,
            _ => return Err(Error::ParseError("Invalid completion")),
        };
        Ok(candidates
            .into_iter()
            .chain(iter::once(String::from("playground")))
            .filter(|ws| ws != self.state.db.name())
            .collect())
    }

    async fn set_environment(&self, completion: Completion) -> Result<Vec<String>> {
        let candidates = match completion {
            Completion::Arg(ArgKey::Name) => {
                sqlx::query_scalar("SELECT DISTINCT env FROM variables")
                    .fetch_all(self.state.db.pool())
                    .await?
            }
            _ => return Err(Error::ParseError("Invalid completion")),
        };

        Ok(match &self.state.env {
            Some(env) => candidates
                .into_iter()
                .filter(|e| e != env.as_ref())
                .collect(),
            None => candidates,
        })
    }
}
