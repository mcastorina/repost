use crate::bastion::Bastion;
use crate::db::{DbObject, Request};
use crate::error::{Error, ErrorKind, Result};
use clap_v3::ArgMatches;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub fn execute(b: &mut Bastion, _matches: &ArgMatches) -> Result<()> {
    let req = b.request()?;
    // display request, input options, and output options
    // get options for this request
    let input_opts = req.input_options();
    let output_opts = req.output_options();

    let mut width = 76;
    if let Some((Width(w), _)) = terminal_size() {
        width = w - 4;
    }
    // print request
    let mut table = Table::new();
    table
        .load_preset("                   ")
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_table_width(width);

    let has_body = {
        if req.body().is_some() {
            "true"
        } else {
            "false"
        }
    };

    table.add_row(vec![
        Cell::new("Name:").set_alignment(CellAlignment::Right),
        Cell::new(req.name()),
    ]);
    table.add_row(vec![
        Cell::new("Method:").set_alignment(CellAlignment::Right),
        Cell::new(req.method().to_string()),
    ]);
    table.add_row(vec![
        Cell::new("URL:").set_alignment(CellAlignment::Right),
        Cell::new(req.url()),
    ]);
    table.add_row(vec![
        Cell::new("Headers:").set_alignment(CellAlignment::Right),
        Cell::new(req.headers().join("\n")),
    ]);
    table.add_row(vec![
        Cell::new("Body?:").set_alignment(CellAlignment::Right),
        Cell::new(has_body),
    ]);
    println!();
    for line in table.to_string().split('\n') {
        println!("  {}", line);
    }
    println!();

    // print input options
    if input_opts.len() > 0 {
        let mut table = Table::new();
        table
            .load_preset(super::show::TABLE_FORMAT)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_table_width(width);
        println!("  Input Options");
        table.set_header(vec!["name", "current values"]);
        for opt in input_opts {
            table.add_row(vec![opt.option_name(), &opt.values().join("\n")]);
        }
        for line in table.to_string().split('\n') {
            println!("  {}", line);
        }
        println!();
    }

    // print output options
    if output_opts.len() > 0 {
        let mut table = Table::new();
        table
            .load_preset(super::show::TABLE_FORMAT)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_table_width(width);
        println!("  Output Options");
        table.set_header(vec!["output variable", "type", "source"]);
        for opt in output_opts {
            table.add_row(vec![
                opt.option_name(),
                opt.extraction_type(),
                opt.extraction_source(),
            ]);
        }
        for line in table.to_string().split('\n') {
            println!("  {}", line);
        }
        println!();
    }

    // print planned requests
    let requests = Request::create_requests(req).unwrap_or_default();
    println!("  Planned Requests");
    super::show::print_table(requests);
    println!();

    Ok(())
}
