mod command;
mod line_reader;

#[macro_use]
use crate::db;
use crate::db::Db;
use clap::{App, IntoApp, Parser};
use command::{Cmd, Command, PrintCmd};
use line_reader::LineReader;
use std::convert::TryInto;

pub struct Repl {
    db: Db,
    editor: LineReader,
}

impl Repl {
    pub async fn new() -> Self {
        let mut app = Command::into_app();
        app._build_all();
        let db = Db::new("repost", "/tmp/repost.db").await.unwrap();
        let mut editor = LineReader::new();
        editor.set_completer(app, &db);
        Self {
            editor,
            db,
        }
    }

    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.editor.read_line(input, "> ")
    }

    pub async fn execute(&mut self, input: &str) -> Result<(), clap::Error> {
        let args = shlex::split(input).unwrap_or_default();
        let cmd = Command::try_parse_from(args)?;
        match cmd.command {
            Cmd::Print(PrintCmd::Requests(args)) => {
                let got = db::query_as_request!(sqlx::query_as("SELECT * FROM requests")
                    .fetch_all(self.db.pool())
                    .await
                    .expect("could not get"));
                dbg!(got);
            }
            Cmd::Print(PrintCmd::Variables(args)) => {
                let got = db::query_as_variable!(sqlx::query_as("SELECT * FROM variables")
                    .fetch_all(self.db.pool())
                    .await
                    .expect("could not get"));
                dbg!(got);
            }
            Cmd::Print(PrintCmd::Environments(args)) => {
                let got = db::query_as_environment!(sqlx::query_as("SELECT * FROM environments")
                    .fetch_all(self.db.pool())
                    .await
                    .expect("could not get"));
                dbg!(got);
            }
            Cmd::Print(PrintCmd::Workspaces(args)) => {}
        }
        Ok(())
    }
}
