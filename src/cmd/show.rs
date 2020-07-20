use crate::bastion::Bastion;
use crate::db::PrintableTable;
use crate::db::{DbObject, Environment, InputOption, Request, Variable};
use crate::error::Result;
use clap_v3::ArgMatches;
use comfy_table::{ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub fn requests(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    print_table(Request::get_all(b.conn())?)
}
pub fn variables(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    match b.current_environment() {
        Some(env) => print_table(Variable::get_by(b.conn(), |var| var.environment() == env)?),
        None => print_table(Variable::get_all(b.conn())?),
    }
}
pub fn options(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    match b.current_request() {
        Some(req) => print_table(InputOption::get_by_name(b.conn(), req)?),
        None => print_table(InputOption::get_all(b.conn())?),
    }
}
pub fn environments(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    print_table(Environment::get_all(b.conn())?)
}

pub fn print_table<T: PrintableTable>(t: T) -> Result<()> {
    let mut width = 76;
    if let Some((Width(w), _)) = terminal_size() {
        width = w - 4;
    }
    let mut table = Table::new();
    table
        .load_preset(TABLE_FORMAT)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_table_width(width);

    table.set_header(t.get_header());
    for row in t.get_rows() {
        table.add_row(row);
    }

    println!();
    for line in table.to_string().split('\n') {
        println!("  {}", line);
    }
    println!();

    Ok(())
}
