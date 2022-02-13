// #[tokio::main]
// async fn main() {}

use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

use clap::{App, AppSettings, IntoApp, Parser, Subcommand};

#[derive(Helper, Validator, Highlighter, Hinter)]
struct DIYCompleter {
    app: App<'static>,
}

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Create { kind: String },
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    #[clap(visible_aliases = &["show", "s", "p"])]
    Print { kind: String },
}

impl Completer for DIYCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        // add an extra character to preserve empty
        let line = format!("{}_", line);
        // split line
        let mut tokens = line.split_whitespace();
        let mut last_token = String::from(tokens.next_back().unwrap());
        // pop off the extra char
        last_token.pop();

        // walk through tokens
        let mut app = &self.app;
        for tok in tokens {
            let child = app.find_subcommand(tok);
            if child.is_none() {
                return Ok((pos, vec![]));
            }
            app = child.unwrap();
        }
        let candidates: Vec<&str> = app
            .get_subcommands()
            .map(|x| x.get_name())
            // .map(|x| x.get_all_aliases())
            // .flatten()
            .filter(|x| x.starts_with(&last_token))
            .collect();
        Ok((
            line.len() - last_token.len() - 1,
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

fn main() -> Result<()> {
    // let args = Cli::into_app().try_get_matches_from(vec!["create", "bar"]);
    let app = Cli::into_app();
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let mut rl: Editor<DIYCompleter> = Editor::with_config(config);
    rl.set_helper(Some(DIYCompleter { app: app.clone() }));

    loop {
        let app = app.clone();
        let input = rl.readline("> ")?;
        let matches = app.try_get_matches_from(input.split_whitespace());
        if let Err(err) = matches {
            eprintln!("{}", err);
            continue;
        }
    }
}
