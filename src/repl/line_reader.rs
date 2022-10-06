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
use std::collections::HashSet;
use std::iter;
use tokio::runtime::Handle;

pub struct LineReader {
    reader: Editor<CommandCompleter>,
    line: Option<String>,
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
            line: None,
        }
    }

    pub fn set_completer(&mut self, state: ReplState) {
        self.reader.set_helper(Some(CommandCompleter { state }));
    }

    pub fn read_line(&mut self, input: &mut String, prompt: &str) -> Option<()> {
        input.clear();
        let readline = match self.line.take() {
            Some(line) => self.reader.readline_with_initial(&prompt, (&line, "")),
            None => self.reader.readline(&prompt),
        };
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

    pub fn set_line(&mut self, line: String) {
        self.line = Some(line);
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
            Builder::CreateVariableBuilder(b) => self.create_variable(prefix, b, completion).await,
            Builder::DeleteRequestsBuilder(b) => self.delete_requests(b).await,
            Builder::DeleteVariablesBuilder(b) => self.delete_variables(b).await,
            _ => Err(Error::ParseError("not implemented")),
        }
    }

    async fn create_variable(
        &self,
        prefix: &str,
        builder: parser::CreateVariableBuilder,
        completion: Completion,
    ) -> Result<Vec<String>> {
        Ok(match completion {
            Completion::Arg(ArgKey::Name) => {
                // TODO: Use request input variables
                sqlx::query_scalar("SELECT DISTINCT name FROM variables")
                    .fetch_all(self.state.db.pool())
                    .await?
            }
            Completion::Arg(_) => {
                match prefix.split_once('=') {
                    None => {
                        // Environments that exist but that do not have a builder.name variable
                        // TODO: this completion should display as 'foo' and replace as 'foo='
                        // instead of 'foo '
                        // If ArgKey::Name is not the completion, builder.name has a value and it
                        // is safe to unwrap here.
                        let name = builder.name.unwrap();
                        sqlx::query_scalar(
                            "SELECT DISTINCT env FROM variables WHERE name != ?1 AND
                            (env NOT IN (SELECT DISTINCT env FROM variables WHERE name = ?1))",
                        )
                        .bind(name)
                        .fetch_all(self.state.db.pool())
                        .await?
                    }
                    Some((_env, _)) => {
                        // TODO:
                        Vec::new()
                    }
                }
            }
            _ => unreachable!(),
        })
    }

    async fn delete_requests(&self, builder: parser::DeleteRequestsBuilder) -> Result<Vec<String>> {
        let existing_names: HashSet<_> = builder.names.into_iter().collect();
        Ok(sqlx::query_scalar("SELECT name FROM requests")
            .fetch_all(self.state.db.pool())
            .await?
            .into_iter()
            .filter(|s| !existing_names.contains(s))
            .collect())
    }

    async fn delete_variables(
        &self,
        builder: parser::DeleteVariablesBuilder,
    ) -> Result<Vec<String>> {
        // TODO: complete IDs
        let existing_names: HashSet<_> = builder.name_or_ids.into_iter().collect();
        Ok(sqlx::query_scalar("SELECT DISTINCT name FROM variables")
            .fetch_all(self.state.db.pool())
            .await?
            .into_iter()
            .filter(|s| !existing_names.contains(s))
            .collect())
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
