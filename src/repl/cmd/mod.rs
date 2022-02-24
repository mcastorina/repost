mod create;
mod print;

use create::*;
use print::*;

use super::Repl;
use crate::db::Db;
use crate::error::Result;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
#[clap(global_setting(AppSettings::DisableVersionFlag))]
pub struct RootCmd {
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
    Request(CreateRequestCmd),
    Variable(CreateVariableCmd),
}

impl RootCmd {
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
            Self::Requests(request) => request.execute(repl).await,
            Self::Variables(variable) => variable.execute(repl).await,
            Self::Environments(env) => env.execute(repl).await,
            Self::Workspaces(workspace) => workspace.execute(repl).await,
        }
    }
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}

impl CreateCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Request(request) => request.execute(repl).await,
            Self::Variable(variable) => variable.execute(repl).await,
        }
    }
}

// The following structs are used for tab-completions and should match their corresponding
// non-completer structs, with the distinction that every argument should be made optional.
// These completer structs should only be used in the line_reader module.
//
// TODO: maybe these structs can be built via a derive?

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
#[clap(global_setting(AppSettings::DisableVersionFlag))]
pub struct RootCmdCompleter {
    #[clap(subcommand)]
    pub command: CmdCompleter,
}

#[derive(Debug, Subcommand)]
pub enum CmdCompleter {
    #[clap(subcommand)]
    Print(PrintCmdCompleter),
    #[clap(subcommand)]
    Create(CreateCmdCompleter),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["get", "show", "p"])]
pub enum PrintCmdCompleter {
    Requests(PrintRequestsCmdCompleter),
    Variables(PrintVariablesCmdCompleter),
    Environments(PrintEnvironmentsCmdCompleter),
    Workspaces(PrintWorkspacesCmdCompleter),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["new", "add", "c"])]
pub enum CreateCmdCompleter {
    Request(CreateRequestCmdCompleter),
    Variable(CreateVariableCmdCompleter),
}

impl RootCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        self.command.arg_candidates(db).await
    }
    pub async fn opt_candidates(&self, opt: &str, db: &Db) -> Result<Vec<String>> {
        self.command.opt_candidates(opt, db).await
    }
}

impl CmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Print(print) => print.arg_candidates(db).await,
            Self::Create(create) => create.arg_candidates(db).await,
        }
    }
    pub async fn opt_candidates(&self, opt: &str, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Print(print) => print.opt_candidates(opt, db).await,
            Self::Create(create) => create.opt_candidates(opt, db).await,
        }
    }
}

impl PrintCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
    pub async fn opt_candidates(&self, opt: &str, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}

impl CreateCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Request(request) => request.arg_candidates(db).await,
            Self::Variable(variable) => variable.arg_candidates(db).await,
        }
    }
    pub async fn opt_candidates(&self, opt: &str, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Request(request) => request.opt_candidates(opt, db).await,
            Self::Variable(variable) => variable.opt_candidates(opt, db).await,
        }
    }
}
