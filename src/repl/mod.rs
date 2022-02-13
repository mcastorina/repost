mod command;
mod line_reader;

use clap::{App, IntoApp, Parser};
use command::Command;
use line_reader::LineReader;

pub struct Repl {
    app: App<'static>,
    line_reader: LineReader,
}

impl Repl {
    pub fn new() -> Self {
        let mut app = Command::into_app();
        app._build_all();
        Self {
            line_reader: LineReader::new(app.clone()),
            app,
        }
    }

    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.line_reader.read_line(input, "> ")
    }

    pub fn execute(&mut self, input: &str) -> Result<(), clap::Error> {
        let args = shlex::split(input).unwrap_or_default();
        let cmd = Command::try_parse_from(args)?;
        dbg!(cmd);
        Ok(())
    }
}
