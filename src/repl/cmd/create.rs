use super::Repl;
use crate::db::Db;
use crate::error::Result;

use std::collections::HashSet;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "Create request")]
#[clap(visible_aliases = &["req", "r"])]
pub struct CreateRequestCmd {
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

impl CreateRequestCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Parser)]
#[clap(about = "Create variables")]
#[clap(visible_aliases = &["var", "v"])]
pub struct CreateVariableCmd {}

impl CreateVariableCmd {
    pub async fn execute(self, repl: &mut Repl) -> Result<()> {
        todo!()
    }
}

// The following structs are used for tab-completions and should match their corresponding
// non-completer structs, with the distinction that every argument should be made optional.
// These completer structs should only be used in the line_reader module.
//
// TODO: maybe these structs can be built via a derive?

#[derive(Debug, Parser)]
#[clap(about = "Create request")]
#[clap(visible_aliases = &["req", "r"])]
pub struct CreateRequestCmdCompleter {
    #[clap(help = "Name of the request")]
    name: Option<String>,

    #[clap(help = "HTTP request URL")]
    url: Option<String>,

    #[clap(help = "HTTP request method (default inferred from name)")]
    #[clap(long = "method")]
    #[clap(short = 'm')]
    method: Option<Option<String>>,

    #[clap(help = "HTTP request headers")]
    #[clap(long = "header")]
    #[clap(short = 'H')]
    headers: Option<Vec<String>>,
}

impl CreateRequestCmdCompleter {
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

#[derive(Debug, Parser)]
#[clap(about = "Create variables")]
#[clap(visible_aliases = &["var", "v"])]
pub struct CreateVariableCmdCompleter {}

impl CreateVariableCmdCompleter {
    pub async fn arg_candidates(&self, db: &Db) -> Result<Vec<String>> {
        todo!()
    }
}
