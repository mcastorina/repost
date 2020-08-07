use crate::bastion::Bastion;
use crate::db::{DbObject, Request};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub fn execute(b: &mut Bastion, _matches: &ArgMatches) -> Result<()> {
    todo!()
}
