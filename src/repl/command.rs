use clap::{AppSettings, Parser, Subcommand};
use crate::error::Result;
use crate::db::{self, Db, DisplayTable};
use std::convert::TryInto;

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

impl Command {
    pub async fn execute(self, db: &Db) -> Result<()> {
        self.command.execute(db).await
    }
}

impl Cmd {
    pub async fn execute(self, db: &Db) -> Result<()> {
        match self {
            Self::Print(print) => print.execute(db).await
        }
    }
}

impl PrintCmd {
    pub async fn execute(self, db: &Db) -> Result<()> {
        match self {
            Self::Requests(_) => {
                let got = db::query_as_request!(
                    sqlx::query_as("SELECT * FROM requests")
                    .fetch_all(db.pool())
                    .await?
                );
                got.print();
            },
            Self::Variables(_) => {
                let got = db::query_as_variable!(
                    sqlx::query_as("SELECT * FROM variables")
                    .fetch_all(db.pool())
                    .await?
                );
                got.print();
            },
            Self::Environments(_) => {
                let got = db::query_as_environment!(
                    sqlx::query_as("SELECT * FROM environments")
                    .fetch_all(db.pool())
                    .await?
                );
                got.print();
            },
            Self::Workspaces(_) => {
                todo!("blocked on Repl configuration data")
            },
        }
        Ok(())
    }
}
