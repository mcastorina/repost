mod command;
mod line_reader;

use crate::db::models::Environment;
use crate::db::Db;
use clap::{App, IntoApp, Parser};
use command::{Cmd, Command, PrintCmd};
use line_reader::LineReader;

pub struct Repl {
    db: Db,
    app: App<'static>,
    line_reader: LineReader,
}

impl Repl {
    pub async fn new() -> Self {
        let mut app = Command::into_app();
        app._build_all();
        let db = Db::new("repost", "/tmp/repost.db").await.unwrap();
        let mut reader = LineReader::new();
        reader.set_completer(&app, &db);
        Self {
            // TODO: line_reader needs to keep a cache of DB contents for the completer
            //       because the complete function cannot be async as it is part of a rustyline
            //       Trait (db requests require async). alternatively, perhaps tokio::spawn can be
            //       used to wait for an async function to return in a synchronous context. see
            //       https://docs.rs/tokio/latest/tokio/runtime/index.html
            line_reader: reader,
            app,
            db,
        }
    }

    pub async fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.line_reader.read_line(input, "> ")
    }

    pub async fn execute(&mut self, input: &str) -> Result<(), clap::Error> {
        let args = shlex::split(input).unwrap_or_default();
        let cmd = Command::try_parse_from(args)?;
        match cmd.command {
            Cmd::Print(PrintCmd::Requests(args)) => {}
            Cmd::Print(PrintCmd::Variables(args)) => {}
            Cmd::Print(PrintCmd::Environments(args)) => {
                let got: Vec<Environment> = sqlx::query_as("SELECT * FROM environments")
                    .fetch_all(self.db.pool())
                    .await
                    .expect("could not get");
                dbg!(got);
            }
            Cmd::Print(PrintCmd::Workspaces(args)) => {}
        }
        Ok(())
    }
}
