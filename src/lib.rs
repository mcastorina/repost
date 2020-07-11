mod bastion;
mod cmd;
mod db;
pub mod error;

use bastion::Bastion;
use error::Result;

pub struct Repl {
    bastion: Bastion,
}

impl Repl {
    pub fn new() -> Result<Repl> {
        let mut repl = Repl {
            bastion: Bastion::new()?,
        };
        Ok(repl)
    }

    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.bastion.get_input(input)
    }

    pub fn execute(&mut self, command: &str) -> Result<()> {
        self.bastion.execute(command)
    }
}
