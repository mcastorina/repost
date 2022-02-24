use super::Repl;
use crate::db::{self, Db, DisplayTable};
use crate::error::Result;

use std::convert::TryInto;
use std::fs;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "Print requests")]
#[clap(visible_aliases = &["request", "reqs", "req", "r"])]
pub struct PrintRequestsCmd {}

impl PrintRequestsCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        let got = db::query_as_request!(
            sqlx::query_as("SELECT * FROM requests")
                .fetch_all(repl.db.pool())
                .await?
        );
        got.print();
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print variables")]
#[clap(visible_aliases = &["variable", "vars", "var", "v"])]
pub struct PrintVariablesCmd {}

impl PrintVariablesCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        let got = db::query_as_variable!(
            sqlx::query_as("SELECT * FROM variables")
                .fetch_all(repl.db.pool())
                .await?
        );
        got.print();
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print environments")]
#[clap(visible_aliases = &["environment", "envs", "env", "e"])]
pub struct PrintEnvironmentsCmd {}

impl PrintEnvironmentsCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        let got = db::query_as_environment!(
            sqlx::query_as("SELECT * FROM environments")
                .fetch_all(repl.db.pool())
                .await?
        );
        got.print();
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print workspaces")]
#[clap(visible_aliases = &["workspace", "ws", "w"])]
pub struct PrintWorkspacesCmd {}

impl PrintWorkspacesCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
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

// The following structs are used for tab-completions and should match their corresponding
// non-completer structs, with the distinction that every argument should be made optional.
// These completer structs should only be used in the line_reader module.
//
// TODO: maybe these structs can be built via a derive?

#[derive(Debug, Parser)]
#[clap(about = "Print requests")]
#[clap(visible_aliases = &["request", "reqs", "req", "r"])]
pub struct PrintRequestsCmdCompleter {}

impl PrintRequestsCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print variables")]
#[clap(visible_aliases = &["variable", "vars", "var", "v"])]
pub struct PrintVariablesCmdCompleter {}

impl PrintVariablesCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print environments")]
#[clap(visible_aliases = &["environment", "envs", "env", "e"])]
pub struct PrintEnvironmentsCmdCompleter {}

impl PrintEnvironmentsCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Print workspaces")]
#[clap(visible_aliases = &["workspace", "ws", "w"])]
pub struct PrintWorkspacesCmdCompleter {}

impl PrintWorkspacesCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}
