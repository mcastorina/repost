use crate::bastion::Bastion;
use crate::db::PrintableTable;
use crate::db::{DbObject, Environment, Request, RequestResponse, Variable};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use colored::*;
use comfy_table::{ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub fn requests(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    println!();
    print_table(Request::get_all(b.conn())?);
    println!();
    Ok(())
}
pub fn variables(b: &Bastion, matches: &ArgMatches) -> Result<()> {
    println!();
    let name = matches.value_of("name");
    match (b.environment(), name) {
        (Some(env), Some(name)) => print_table(Variable::get_by(b.conn(), |var| {
            var.environment() == env && var.name() == name
        })?),
        (Some(env), None) => {
            print_table(Variable::get_by(b.conn(), |var| var.environment() == env)?)
        }
        (None, Some(name)) => print_table(Variable::get_by(b.conn(), |v| v.name() == name)?),
        (None, None) => print_table(Variable::get_all(b.conn())?),
    };
    println!();
    Ok(())
}
pub fn options(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    let req = b.request()?;

    println!();
    print_table(req.input_options());
    println!();
    Ok(())
}
pub fn environments(b: &Bastion, _matches: &ArgMatches) -> Result<()> {
    println!();
    print_table(Environment::get_all(b.conn())?);
    println!();
    Ok(())
}
pub fn response(b: &Bastion, matches: &ArgMatches) -> Result<()> {
    let id = matches.value_of("id");
    if id.is_none() {
        println!();
        print_table(RequestResponse::get_all(b.conn())?);
        println!();
        return Ok(());
    }
    let id = id.unwrap();
    let rr = RequestResponse::get_by_id(b.conn(), id.parse()?)?;

    let tx = matches.is_present("transmitted");
    let rx = matches.is_present("received");
    let (tx, rx) = match (tx, rx) {
        (false, false) => (true, true),
        x => x,
    };

    if tx {
        println!("\n{}", "  Request".bold());
        println!("  =========");
        println!(
            "{}",
            format!("> {} {}", rr.method(), rr.url()).bright_black()
        );
        for header in rr.request_headers() {
            println!("{}", format!("> {}", header).bright_black(),);
        }
        println!();

        if let Some(body) = rr.request_body() {
            println!("{}", "  Request Body".bold());
            println!("  ==============");
            super::run::display_body(std::str::from_utf8(body).unwrap(), true);
            println!();
        }
    }

    if rx {
        println!("\n{}", "  Response".bold());
        println!("  ==========");
        println!(
            "{}",
            format!("< {}", rr.status().unwrap_or("-")).bright_black()
        );
        for header in rr.response_headers() {
            println!("{}", format!("< {}", header).bright_black(),);
        }
        println!();

        if let Some(body) = rr.response_body() {
            println!("{}", "  Response Body".bold());
            println!("  ===============");
            super::run::display_body(std::str::from_utf8(body).unwrap(), true);
            println!();
        }

        let extractions = rr.extractions();
        if extractions.len() > 0 {
            println!("{}", "  Extractions".bold());
            println!("  =============");
            for e in extractions {
                let (name, value) = e;
                println!("{}", format!("{} <= {}", name, value).bright_black());
            }
            println!();
        }
    }

    Ok(())
}

pub fn print_table<T: PrintableTable>(t: T) {
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

    for line in table.to_string().split('\n') {
        println!("  {}", line);
    }
}
