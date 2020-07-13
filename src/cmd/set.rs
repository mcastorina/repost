use crate::bastion::Bastion;
use crate::db::{DbObject, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;

pub fn variable(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let name = matches.value_of("name").unwrap();
    let env_vals = matches.values_of("environment=value").unwrap();

    // TODO: add validator to yaml once available
    if !env_vals.clone().all(|s| s.contains('=')) {
        return Err(Error::new(ErrorKind::ArgumentError(
            "Found argument that does not contain '='",
        )));
    }

    // TODO: Vec<(String, Option<String>)>
    let env_vals: Vec<(String, String)> = env_vals
        .map(|s| {
            let mut items = s.splitn(2, "=");
            // We can unwrap because this argument is guaranteed to have one '='
            (
                String::from(items.next().unwrap()),
                String::from(items.next().unwrap()),
            )
        })
        .collect();

    let mut var = Variable::get_by_name_map(b.conn(), name, |e| String::from(e.environment()))?;
    for env_val in env_vals {
        // TODO: create a new variable function
        let (environment, value) = env_val;
        let mut var = var.remove(&environment).unwrap_or(Variable::new(
            name,
            &environment,
            Some(&value),
            Some("user"),
        ));
        var.set_value(Some(&value));
        var.upsert(b.conn())?;
    }
    // TODO: update options for variable
    Ok(())
}
