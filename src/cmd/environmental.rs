use crate::cmd::{Cmd, CmdError};
use crate::db::{Method, Request};
use crate::Repl;
use reqwest::blocking;

pub struct EnvironmentalCommand {}
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
        let mut req = repl.db.get_request(args[1])?;
        // get options for this request
        let opts = repl.db.get_options()?.into_iter().filter(|x| req.has_option(&x)).collect();
        // do option substitution
        // TODO: return result with missing options
        if !req.substitute_options(opts) {
            return Err(CmdError::MissingOptions);
        }

        // TODO: this can be a method of Request
        let client = blocking::Client::new();
        let mut builder = match req.method() {
            Method::GET => client.get(req.url()),
            Method::POST => client.post(req.url()),
            Method::PUT => client.put(req.url()),
            Method::PATCH => client.patch(req.url()),
            Method::DELETE => client.delete(req.url()),
            Method::HEAD => client.head(req.url()),
        };

        // add headers
        if let Some(x) = req.headers() {
            for hv in x.split('\n') {
                let mut items = hv.splitn(2, ":");
                let (header, value) = (items.next(), items.next());
                if header.and(value).is_none() {
                    continue
                }
                builder = builder.header(header.unwrap(), value.unwrap());
            }
        }

        let resp = builder.send();
        println!("{:?}", resp);
        Ok(())
    }
}
