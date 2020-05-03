pub mod cli;
mod models;

use cli::Command;
use models::{Request, Variable};

pub fn run(conf: Command) -> Result<(), &'static str> {
    match conf {
        Command::CreateRequest {
            name,
            url,
            method,
            body,
            headers,
        } => {
            let method = method.unwrap_or(String::from("GET"));
            let request = Request::new(name, url, method, headers, body)?;
            request.save()
        }
        Command::CreateVariable { name, env_vals } => {
            let variable = Variable::new(name, env_vals)?;
            variable.save()
        }
        Command::None => Err("no command provided"),
    }
}
