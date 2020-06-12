use crate::cmd::{Cmd,CmdError};
use crate::db::{Method, Request};
use crate::Repl;
use reqwest::blocking;

pub struct EnvironmentalCommand{}
impl Cmd for EnvironmentalCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        match args[0].to_lowercase().as_ref() {
            "run" | "r" => EnvironmentalCommand::execute_run(repl, args),
            _ => Err(CmdError::NotFound),
        }
    }
}
impl EnvironmentalCommand {
    fn execute_run(repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        // TODO run multiple in a row
        if args.len() != 2 {
            println!("Run a named HTTP request\n\nUsage: run <request>\n");
            return Ok(());
        }
        let req: Vec<Request> = repl
            .db
            .get_requests()?
            .into_iter()
            .filter(|x| x.name() == args[1])
            .collect();
        if req.len() == 0 {
            return Err(CmdError::ArgsError(format!(
                "Request not found: {}",
                args[1]
            )));
        }
        let req = &req[0];
        let client = blocking::Client::new();
        let builder = match req.method() {
            Method::GET => client.get(req.url()),
            Method::POST => client.post(req.url()),
            Method::PUT => client.put(req.url()),
            Method::PATCH => client.patch(req.url()),
            Method::DELETE => client.delete(req.url()),
            Method::HEAD => client.head(req.url()),
        };
        let resp = builder.send();
        println!("{:?}", resp);
        Ok(())
    }
}
