use super::Repl;
use crate::db::models::Request;
use crate::db::{self, Db, DisplayTable};
use crate::error::Result;

use std::convert::TryInto;
use std::fs;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
// #[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
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
// #[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub enum PrintCmd {
    Requests(PrintRequestsCmd),
    Variables(PrintVariablesCmd),
    Environments(PrintEnvironmentsCmd),
    Workspaces(PrintWorkspacesCmd),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["new", "add", "c"])]
// #[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
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
    name: Option<String>,

    #[clap(help = "HTTP request URL")]
    url: Option<String>,

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
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        self.command.arg_candidates(db).await
    }
}

impl Cmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Print(print) => print.execute(repl).await,
            Self::Create(create) => create.execute(repl).await,
        }
    }
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Print(print) => print.arg_candidates(db).await,
            Self::Create(create) => create.arg_candidates(db).await,
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
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
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

use std::collections::HashSet;

impl CreateCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        match self {
            Self::Request(args) => {
                dbg!(&args);
                Request::new(
                    args.name.unwrap_or_default(),
                    "GET",
                    args.url.unwrap_or_default(),
                );
            }
            Self::Variable(args) => {
                dbg!(args);
            }
        }
        Ok(())
    }
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        match self {
            Self::Request(request) => request.arg_candidates(db).await,
            Self::Variable(variable) => variable.arg_candidates(db).await,
        }
    }
}
impl CreateRequestsCmd {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        match (&self.name, &self.url) {
            (None, _) => self.name_candidates(db).await,
            (_, None) => self.url_candidates(db).await,
            _ => Ok(vec![]),
        }
    }

    async fn name_candidates(&self, db: &Db) -> Result<Vec<String>> {
        // candidates for NAME
        let prefixes = &["create", "update", "get", "delete"];
        let names: Vec<String> = sqlx::query_scalar("SELECT name FROM requests")
            .fetch_all(db.pool())
            .await?;
        // candidates are of type {prefix}-{unique-names}
        let name_set: HashSet<String> = names
            .iter()
            .filter_map(|full_name| full_name.split_once('-'))
            .map(|(_, name)| name.to_string())
            .collect();

        if name_set.len() == 0 {
            // TODO: use prefixes slice
            return Ok(vec![
                "create-".to_string(),
                "update-".to_string(),
                "get-".to_string(),
                "delete-".to_string(),
            ]);
        }

        // generate all names and do set difference with existing names
        let candidates: HashSet<_> = prefixes
            .iter()
            .map(|prefix| {
                name_set
                    .iter()
                    .map(move |name| format!("{}-{}", prefix, name))
            })
            .flatten()
            .collect();

        // TODO: lots of clones happening here which is probably unnecessary
        Ok(candidates
            .difference(&names.into_iter().collect())
            .into_iter()
            .cloned()
            .collect())
    }
    async fn url_candidates(&self, db: &Db) -> Result<Vec<String>> {
        let name_query = self
            .name
            .as_ref()
            // if name is some (it should be), then query for requests that end in the same name
            .and_then(|name| name.split_once('-').map(|(_, name)| format!("%-{}", name)))
            // otherwise, get all urls
            .unwrap_or_else(|| "%".to_string());

        let candidates: Vec<String> =
            sqlx::query_scalar("SELECT DISTINCT url FROM requests WHERE name LIKE ?")
                .bind(name_query)
                .fetch_all(db.pool())
                .await?;
        Ok(candidates)
    }
}
impl CreateVariablesCmd {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}
