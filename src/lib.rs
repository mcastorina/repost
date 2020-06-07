pub mod cli;
mod models;

use cli::Command;
use models::{Request, Variable};

pub fn run(conf: Command) -> Result<(), String> {
    match conf {
        Command::CreateRequest(cmd) => {
            let method = cmd.method.unwrap_or(String::from("GET"));
            let request = Request::new(cmd.name, cmd.url, method, cmd.headers, cmd.body)?;
            request.save()
        }
        Command::CreateVariable(cmd) => {
            let variable = Variable::new(cmd.name, cmd.env_vals)?;
            variable.save()
        }
        Command::None => Err(String::from("no command provided")),
    }
}
