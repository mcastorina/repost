use rustyline::completion::{Candidate, Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{
    error::ReadlineError, CompletionType, Config, Context, EditMode, Editor,
    Result as ReadlineResult,
};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use crate::db;
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
        let prefix = match completion {
            Some((prefix, _)) => prefix,
            _ => "",
        };
        let candidates = match completion {
            Some((_, Completion::Command(cmds))) => cmds
                .iter()
                .filter_map(|cmd| {
                    cmd.completions()
                        .iter()
                        .filter(|cand| cand.starts_with(prefix))
                        .next()
                })
                .copied()
                .filter(|cand| cand.starts_with(prefix))
                .map(SmartCompletion::default)
                .map(Pair::from)
                .collect(),
            Some((_, Completion::OptKey)) => builder
                .unwrap()
                .opts()
                .flat_map(|opt| opt.completions())
                .copied()
                .filter(|cand| cand.starts_with(prefix))
                .map(SmartCompletion::default)
                .map(Pair::from)
                .collect(),
            Some((_, completion)) => {
                return Ok(tokio::task::block_in_place(|| {
                    Handle::current().block_on(async {
                        self.smart_complete(line, prefix, builder.unwrap(), completion)
                            .await
                    })
                })
                .unwrap_or_default());
            }
            None => vec![],
        };

        Ok((line.len() - prefix.len(), candidates))
    }
}

impl CommandCompleter {
    #[rustfmt::skip]
    const COMMON_HEADERS: &'static [&'static str] = &[
        "A-IM", "Accept", "Accept-Charset", "Accept-Datetime", "Accept-Encoding",
        "Accept-Language", "Access-Control-Request-Method", "Access-Control-Request-Headers",
        "Authorization", "Cache-Control", "Connection", "Permanent", "Content-Encoding",
        "Content-Length", "Content-MD5", "Content-Type", "Cookie", "Date", "Expect",
        "Forwarded", "From", "Host", "HTTP2-Settings", "If-Match", "If-Modified-Since",
        "If-None-Match", "If-Range", "If-Unmodified-Since", "Max-Forwards", "Origin",
        "Pragma", "Prefer", "Proxy-Authorization", "Range", "Referer", "TE", "Trailer",
        "Transfer-Encoding", "User-Agent", "Upgrade", "Via", "Warning",
    ];
    const COMMON_METHODS: &'static [&'static str] = &[
        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
    ];
    async fn smart_complete(
        &self,
        line: &str,
        prefix: &str,
        builder: Builder,
        completion: Completion,
    ) -> Result<(usize, Vec<Pair>)> {
        let (delim, prefix) = match prefix.chars().next() {
            c @ Some('\'') | c @ Some('"') => (c, &prefix[1..]),
            _ => (None, prefix),
        };
        let candidates: Vec<_> = match builder {
            Builder::SetEnvironmentBuilder(_) => self.set_environment(completion).await,
            Builder::SetWorkspaceBuilder(_) => self.set_workspace(completion).await,
            Builder::CreateRequestBuilder(b) => self.create_request(prefix, b, completion).await,
            Builder::CreateVariableBuilder(b) => self.create_variable(prefix, b, completion).await,
            Builder::DeleteRequestsBuilder(b) => self.delete_requests(b).await,
            Builder::DeleteVariablesBuilder(b) => self.delete_variables(b).await,
            _ => return Err(Error::ParseError("Unsupported completion")),
        }?
        .into_iter()
        .filter(|cand| cand.starts_with(prefix))
        .map(|cand| cand.with_delim(delim))
        .map(Pair::from)
        .collect();

        Ok(match delim {
            Some(_) => (line.len() - prefix.len() - 1, candidates),
            None => (line.len() - prefix.len(), candidates),
        })
    }

    async fn create_request(
        &self,
        prefix: &str,
        builder: parser::CreateRequestBuilder,
        completion: Completion,
    ) -> Result<Vec<SmartCompletion>> {
        match completion {
            Completion::Arg(ArgKey::Name) => {
                // TODO
                Err(Error::ParseError("not implemented"))
            }
            Completion::Arg(ArgKey::URL) => {
                // TODO
                Err(Error::ParseError("not implemented"))
            }
            Completion::Arg(_) => Err(Error::ParseError("no completions")),
            Completion::OptValue(OptKey::Header) => Ok(Self::COMMON_HEADERS
                .iter()
                .map(SmartCompletion::header_key)
                .collect()),
            Completion::OptValue(OptKey::Method) => Ok(Self::COMMON_METHODS
                .iter()
                .map(SmartCompletion::method)
                .collect()),
            _ => Err(Error::ParseError("no completions")),
        }
    }

    async fn create_variable(
        &self,
        prefix: &str,
        builder: parser::CreateVariableBuilder,
        completion: Completion,
    ) -> Result<Vec<SmartCompletion>> {
        match completion {
            Completion::Arg(ArgKey::Name) => {
                // Name will probably come from request variables.
                let requests = db::query_as_request!(
                    sqlx::query_as("SELECT * FROM requests")
                        .fetch_all(self.state.db.pool())
                        .await?
                );
                let variables: Vec<_> = requests.iter().map(|r| r.input_variables()).collect();
                let variables: HashSet<_> = variables.iter().flat_map(|v| v.iter()).collect();
                Ok(variables.iter().map(SmartCompletion::default).collect())
            }
            Completion::Arg(_) => {
                // environment=value completion
                // If ArgKey::Name is not the completion, builder.name has a value and it
                // is safe to unwrap here.
                let name = builder.name.unwrap();
                match prefix.split_once('=') {
                    None => {
                        // Environment part of the environment=value completion.
                        let existing_envs: HashSet<_> = builder
                            .env_vals
                            .iter()
                            .filter_map(|ev| ev.split_once('=').map(|(e, _v)| e))
                            .collect();
                        // Return environments that exist but don't have a variable associated with
                        // name.
                        Ok(sqlx::query_scalar(
                            "SELECT DISTINCT env FROM variables WHERE name != ?1 AND
                            (env NOT IN (SELECT DISTINCT env FROM variables WHERE name = ?1))",
                        )
                        .bind(name)
                        .fetch_all(self.state.db.pool())
                        .await?
                        .into_iter()
                        .chain(self.state.env.iter().map(|e| e.name.clone()))
                        .filter(|cand: &String| !existing_envs.contains(cand.as_str()))
                        .map(SmartCompletion::env_val_key)
                        .collect())
                    }
                    Some((_env, _)) => {
                        // Suggest the same value for variables with the same name.
                        Ok(dbg!(
                            sqlx::query_scalar::<_, String>(
                                "SELECT DISTINCT value FROM variables WHERE name = ?"
                            )
                            .bind(name)
                            .fetch_all(self.state.db.pool())
                            .await?
                        )
                        .into_iter()
                        // TODO: smart complete for EnvValValue (this currently never works)
                        .map(SmartCompletion::default)
                        .collect())
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    async fn delete_requests(
        &self,
        builder: parser::DeleteRequestsBuilder,
    ) -> Result<Vec<SmartCompletion>> {
        let existing_names: HashSet<_> = builder.names.into_iter().collect();
        Ok(sqlx::query_scalar::<_, String>("SELECT name FROM requests")
            .fetch_all(self.state.db.pool())
            .await?
            .into_iter()
            .filter(|s| !existing_names.contains(s))
            .map(SmartCompletion::default)
            .collect())
    }

    async fn delete_variables(
        &self,
        builder: parser::DeleteVariablesBuilder,
    ) -> Result<Vec<SmartCompletion>> {
        // TODO: complete IDs
        let existing_names: HashSet<_> = builder.name_or_ids.into_iter().collect();
        Ok(
            sqlx::query_scalar::<_, String>("SELECT DISTINCT name FROM variables")
                .fetch_all(self.state.db.pool())
                .await?
                .into_iter()
                .filter(|s| !existing_names.contains(s))
                .map(SmartCompletion::default)
                .collect(),
        )
    }

    async fn set_workspace(&self, completion: Completion) -> Result<Vec<SmartCompletion>> {
        let candidates = match completion {
            Completion::Arg(ArgKey::Name) => self.state.workspaces()?,
            _ => return Err(Error::ParseError("Invalid completion")),
        };
        Ok(candidates
            .into_iter()
            .chain(iter::once(String::from("playground")))
            .filter(|ws| ws != self.state.db.name())
            .map(SmartCompletion::default)
            .collect())
    }

    async fn set_environment(&self, completion: Completion) -> Result<Vec<SmartCompletion>> {
        Ok(match completion {
            Completion::Arg(ArgKey::Name) => {
                sqlx::query_scalar::<_, String>("SELECT DISTINCT env FROM variables")
                    .fetch_all(self.state.db.pool())
                    .await?
            }
            _ => return Err(Error::ParseError("Invalid completion")),
        }
        .into_iter()
        .filter(|e| {
            if let Some(env) = &self.state.env {
                e != env.as_ref()
            } else {
                true
            }
        })
        .map(SmartCompletion::default)
        .collect())
    }
}

struct SmartCompletion {
    display: String,
    kind: CompletionKind,
    delim: Option<char>,
}

enum CompletionKind {
    Default,
    HeaderKey,
    Method,
    EnvValKey,
}

impl SmartCompletion {
    fn new(kind: CompletionKind, value: impl AsRef<str>) -> Self {
        Self {
            kind,
            display: value.as_ref().to_string(),
            delim: None,
        }
    }
    fn default(s: impl AsRef<str>) -> Self {
        Self::new(CompletionKind::Default, s)
    }
    fn header_key(s: impl AsRef<str>) -> Self {
        Self::new(CompletionKind::HeaderKey, s)
    }
    fn method(s: impl AsRef<str>) -> Self {
        Self::new(CompletionKind::Method, s)
    }
    fn env_val_key(s: impl AsRef<str>) -> Self {
        Self::new(CompletionKind::EnvValKey, s)
    }
    fn with_delim(mut self, delim: Option<char>) -> Self {
        self.delim = delim;
        self
    }
    fn starts_with(&self, prefix: impl AsRef<str>) -> bool {
        let prefix = prefix.as_ref();
        match self.kind {
            CompletionKind::Default | CompletionKind::EnvValKey => self.display.starts_with(prefix),
            CompletionKind::Method | CompletionKind::HeaderKey => self
                .display
                .to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase()),
        }
    }
}

impl From<SmartCompletion> for Pair {
    fn from(completion: SmartCompletion) -> Self {
        let replacement = match (completion.kind, completion.delim) {
            (CompletionKind::Default, None) => format!("{} ", completion.display),
            (CompletionKind::Default, Some(c)) => format!("{c}{}{c} ", completion.display),
            (CompletionKind::HeaderKey, None) => format!("{}:", completion.display),
            (CompletionKind::HeaderKey, Some(c)) => format!("{c}{}: ", completion.display),
            (CompletionKind::Method, _) => format!("{} ", completion.display.to_ascii_uppercase()),
            (CompletionKind::EnvValKey, None) => format!("{}=", completion.display),
            (CompletionKind::EnvValKey, Some(c)) => format!("{c}{}=", completion.display),
        };
        Self {
            display: completion.display,
            replacement,
        }
    }
}
