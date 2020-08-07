use crate::bastion::Bastion;
use crate::db::{DbObject, Request, Variable};
use crate::error::Result;
use clap_v3::ArgMatches;

pub fn requests(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let requests: Vec<&str> = matches.values_of("request").unwrap().collect();
    for request in requests {
        let v = Request::get_by(b.conn(), |req| req.name() == request)?;
        if v.len() == 0 {
            println!("Request '{}' not found.", request);
            continue;
        }
        for e in v {
            e.delete(b.conn())?;
        }
    }
    // b.set_state()?;
    // b.set_completions()?;
    Ok(())
}

pub fn variables(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let vars: Vec<&str> = matches.values_of("variable").unwrap().collect();
    for var in vars {
        let v = Variable::get_by(b.conn(), |v| v.name() == var)?;
        if v.len() == 0 {
            println!("Variable '{}' not found.", var);
            continue;
        }
        for e in v {
            // only delete variables in the current environment if set
            match b.current_environment() {
                Some(x) if x != e.environment() => (),
                _ => e.delete(b.conn())?,
            };
        }
        // TODO: only update if the option source is "variable"
        // b.set_options(InputOption::get_by(b.conn(), |x| x.option_name() == var)?)?;
    }
    // b.set_completions()?;
    Ok(())
}

pub fn options(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    todo!();
}
