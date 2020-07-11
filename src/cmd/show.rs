use crate::error::Result;
use crate::db::PrintableTable;
use clap_v3::ArgMatches;

// fn print_table<T: PrintableTable>(t: T) -> Result<(), CmdError> {
pub fn print_table<T: PrintableTable>(matches: &ArgMatches, table: T) -> Result<()> {
    println!("{:?}", T::get_header());
    Ok(())
}
