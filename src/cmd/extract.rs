use crate::bastion::Bastion;
use crate::db::DbObject;
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;

pub fn execute(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let mut req = b.request()?;
    let var = matches.value_of("variable").unwrap();
    let typ = matches.value_of("type").unwrap();
    let key = matches.value_of("key").unwrap();

    req.add_extraction(var, typ, key);
    req.update(b.conn())?;
    Ok(())
}
