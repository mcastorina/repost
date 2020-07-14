use crate::bastion::Bastion;
use crate::db::{DbObject, Method, Request, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;

pub fn requests(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let requests: Vec<&str> = matches.values_of("request").unwrap().collect();
    for request in requests {
        let v = Request::get_by_name(b.conn(), request)?;
        if v.len() == 0 {
            println!("Request '{}' not found.", request);
            continue;
        }
        for e in v {
            e.delete(b.conn())?;
        }
        // TODO: update options for request
    }
    b.set_completions()?;
    Ok(())
}

pub fn variables(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let vars: Vec<&str> = matches.values_of("variable").unwrap().collect();
    for var in vars {
        let v = Variable::get_by_name(b.conn(), var)?;
        if v.len() == 0 {
            println!("Variable '{}' not found.", var);
            continue;
        }
        for e in v {
            e.delete(b.conn())?;
        }
        // TODO: update options for variable
    }
    b.set_completions()?;
    Ok(())
}
