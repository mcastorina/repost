use crate::bastion::Bastion;
use crate::db::DbObject;
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;

pub fn execute(b: &mut Bastion, matches: &ArgMatches) -> Result<()> {
    let req = b.request()?;
    let extraction_source = matches.value_of("type").unwrap();
    let key = matches.value_of("key").unwrap();
    let var = matches.value_of("variable").unwrap();

    todo!();
}
