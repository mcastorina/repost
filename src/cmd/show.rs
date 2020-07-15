use crate::db::PrintableTable;
use crate::error::Result;
use clap_v3::ArgMatches;
use comfy_table::{ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub fn print_table<T: PrintableTable>(matches: &ArgMatches, t: T) -> Result<()> {
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
