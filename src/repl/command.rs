use super::Repl;
use crate::db::{self, DisplayTable};
use crate::db::models::Request;
use crate::error::Result;

use std::convert::TryInto;
use std::fs;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
#[clap(global_setting(AppSettings::DisableVersionFlag))]
pub struct Command {
    #[clap(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[clap(subcommand)]
    Print(PrintCmd),
    #[clap(subcommand)]
    Create(CreateCmd),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["get", "show", "p"])]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub enum PrintCmd {
    Requests(PrintRequestsCmd),
    Variables(PrintVariablesCmd),
    Environments(PrintEnvironmentsCmd),
    Workspaces(PrintWorkspacesCmd),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["new", "add", "c"])]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub enum CreateCmd {
    Request(CreateRequestsCmd),
    Variable(CreateVariablesCmd),
}

#[derive(Debug, Parser)]
#[clap(about = "Print requests")]
#[clap(visible_aliases = &["request", "reqs", "req", "r"])]
pub struct PrintRequestsCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print variables")]
#[clap(visible_aliases = &["variable", "vars", "var", "v"])]
pub struct PrintVariablesCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print environments")]
#[clap(visible_aliases = &["environment", "envs", "env", "e"])]
pub struct PrintEnvironmentsCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print workspaces")]
#[clap(visible_aliases = &["workspace", "ws", "w"])]
pub struct PrintWorkspacesCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Create request")]
#[clap(visible_aliases = &["req", "r"])]
pub struct CreateRequestsCmd {
    #[clap(help = "Name of the request")]
    name: String,

    #[clap(help = "HTTP request URL")]
    url: String,

    #[clap(help = "HTTP request method (default inferred from name)")]
    #[clap(long = "method")]
    #[clap(short = 'm')]
    method: Option<String>,

    #[clap(help = "HTTP request headers")]
    #[clap(long = "header")]
    #[clap(short = 'H')]
    // TODO: validator
    headers: Vec<String>,
}

#[derive(Debug, Parser)]
#[clap(about = "Create variables")]
#[clap(visible_aliases = &["var", "v"])]
pub struct CreateVariablesCmd {}

impl Command {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        self.command.execute(repl).await
    }
}

impl Cmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Print(print) => print.execute(repl).await,
            Self::Create(create) => create.execute(repl).await,
        }
    }
}

impl PrintCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Requests(_) => {
                let got = db::query_as_request!(
                    sqlx::query_as("SELECT * FROM requests")
                        .fetch_all(repl.db.pool())
                        .await?
                );
                got.print();
            }
            Self::Variables(_) => {
                let got = db::query_as_variable!(
                    sqlx::query_as("SELECT * FROM variables")
                        .fetch_all(repl.db.pool())
                        .await?
                );
                got.print();
            }
            Self::Environments(_) => {
                let got = db::query_as_environment!(
                    sqlx::query_as("SELECT * FROM environments")
                        .fetch_all(repl.db.pool())
                        .await?
                );
                got.print();
            }
            Self::Workspaces(_) => {
                let stems: Vec<_> = fs::read_dir(&repl.conf.data_dir)?
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let path = entry.path();
                        match path.extension() {
                            Some(x) if x == "db" => Some(path),
                            _ => None,
                        }
                    })
                    .filter_map(|path| path.file_stem().map(|ws| ws.to_owned()))
                    .collect();
                let refs = stems.iter().filter_map(|stem| stem.to_str()).collect();
                // leverage DisplayTable to print out a nice format
                WorkspaceTable(refs).print();
            }
        }
        Ok(())
    }
}

struct WorkspaceTable<'w>(Vec<&'w str>);

impl<'w> DisplayTable for WorkspaceTable<'w> {
    const HEADER: &'static [&'static str] = &["workspace"];
    fn build(&self, table: &mut comfy_table::Table) {
        for ws in &self.0 {
            table.add_row(&[ws]);
        }
    }
}

impl CreateCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Request(args) => {
                dbg!(&args);
                Request::new(args.name, "GET", args.url);
            }
            Self::Variable(args) => {
                dbg!(args);
            }
        }
        Ok(())
    }
}
