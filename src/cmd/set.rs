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

    for env_val in env_vals {
        let (env, val) = env_val;
        let mut var = Variable::get_unique(b.conn(), name, &env).unwrap_or(Variable::new(
            name,
            &env,
            None,
            Some("user"),
        ));
        var.set_value(Some(&val));
        var.upsert(b.conn())?;
        if let Some(env) = b.environment() {
            if env == var.environment() {
                var.set_options(b.conn())?;
            }
        }
    }
    Ok(())
}
