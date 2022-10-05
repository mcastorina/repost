mod config;
mod line_reader;
mod parser;

pub use config::ReplConfig;

use crate::cmd::Cmd;
use crate::db::models::Environment;
use crate::db::{self, Db, DisplayTable};
use crate::error::{Error, Result};
use colored::Colorize;
use line_reader::LineReader;
use parser::Command;

/// Repl object for handling readline editing, a database,
/// and executing commands.
pub struct Repl {
    editor: LineReader,
    state: ReplState,
}

#[derive(Clone)]
pub struct ReplState {
    db: Db,
    conf: ReplConfig,
    env: Option<Environment>,
}

impl Repl {
    /// Create a new Repl struct with a Db and LineReader struct.
    pub async fn new(conf: ReplConfig) -> Result<Self> {
        // start with an in-memory database
        let state = ReplState {
            db: Db::new_playground().await?,
            conf,
            env: None,
        };

        Ok(Self {
            editor: LineReader::new(),
            state,
        })
    }

    /// Read stdin into input using the LineReader.
    /// Returns None on EOF or error.
    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        // TODO: use a read-only reference to self.state
        self.editor.set_completer(self.state.clone());
        self.editor.read_line(input, self.state.prompt().as_ref())
    }

    /// Execute a command line.
    pub async fn execute(&mut self, input: &str) -> Result<()> {
        let cmd = Cmd::new(&self.state.db);
        match parser::parse_command(input)? {
            Command::CreateRequest(args) => cmd.create_request(args.try_into()?).await?,
            Command::CreateVariable(args) => cmd.create_variable(args.try_into()?).await?,
            Command::DeleteRequests(args) => cmd.delete_requests(args.into()).await?,
            Command::DeleteVariables(args) => cmd.delete_variables(args.into()).await?,
            Command::PrintRequests(_) => cmd.print_requests().await?,
            Command::PrintVariables(_) => cmd.print_variables().await?,
            Command::PrintEnvironments(_) => cmd.print_environments().await?,
            Command::PrintWorkspaces(_) => {
                RowHighlighter::new(self.state.workspaces()?, |w| self.state.db.name() == w)
                    .print_with_header(&["workspaces"])
            }
            Command::SetEnvironment(args) => {
                self.state
                    .set_environment(args.environment.map(Environment::from))
                    .await?
            }
            Command::SetWorkspace(args) => self.state.set_workspace(args.workspace).await?,
            Command::Help(builder) => {
                builder.usage();
                self.editor.set_line(String::from(
                    input.trim_end_matches("--help").trim_end_matches("-h"),
                ));
            }
        }
        Ok(())
    }
}

impl ReplState {
    fn prompt(&self) -> String {
        let db = self.db.name().yellow();
        match &self.env {
            Some(env) => format!("[{}][{}] > ", db, env.name.cyan().bold()),
            None => format!("[{}] > ", db),
        }
    }

    async fn set_workspace(&mut self, workspace: Option<String>) -> Result<()> {
        let workspace = workspace.unwrap_or_else(|| String::from("playground"));
        if self.db.name() == workspace {
            return Ok(());
        }
        self.db = match workspace.as_ref() {
            "playground" => Db::new_playground().await?,
            workspace => {
                let path = self.conf.data_dir.join(workspace).with_extension("db");
                Db::new(path.to_string_lossy()).await?
            }
        };
        Ok(())
    }

    async fn set_environment(&mut self, env: Option<Environment>) -> Result<()> {
        if env.is_none() {
            self.env = None;
            return Ok(());
        }
        let cand = env.as_ref().unwrap();
        let _: bool = sqlx::query_scalar("SELECT 1 FROM variables WHERE env = ? LIMIT 1")
            .bind(cand.as_ref())
            .fetch_one(self.db.pool())
            .await?;
        self.env = env;
        Ok(())
    }

    fn workspaces(&self) -> Result<Vec<String>> {
        Ok(self
            .conf
            .dbs()?
            .into_iter()
            .map(|db| Db::name_of(&db.to_string_lossy()).to_owned())
            .collect())
    }
}

struct RowHighlighter<T: DisplayTable, F: Fn(&T) -> bool> {
    rows: Vec<T>,
    color: comfy_table::Color,
    column_index: usize,
    predicate: F,
}

impl<T: DisplayTable, F: Fn(&T) -> bool> DisplayTable for RowHighlighter<T, F> {
    const HEADER: &'static [&'static str] = T::HEADER;
    fn build(&self, table: &mut comfy_table::Table) {
        for (i, item) in self.rows.iter().enumerate() {
            item.build(table);
            if !(self.predicate)(item) {
                continue;
            }
            if let Some(row) = table.get_row_mut(i) {
                *row = row
                    .cell_iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        if i == self.column_index {
                            cell.clone().fg(self.color)
                        } else {
                            cell.clone()
                        }
                    })
                    .fold(comfy_table::Row::new(), |mut row, cell| {
                        row.add_cell(cell);
                        row
                    });
            }
        }
    }
}

impl<T: DisplayTable, F: Fn(&T) -> bool> RowHighlighter<T, F> {
    fn new(rows: Vec<T>, predicate: F) -> Self {
        let column_index = T::HEADER.iter().position(|&s| s == "name").unwrap_or(0);
        Self {
            rows,
            column_index,
            color: comfy_table::Color::Green,
            predicate,
        }
    }
}
