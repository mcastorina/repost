use crate::cmd::{BaseCommand, Cmd, CmdError};
use crate::db::{RequestInput, RequestOutput};
use crate::Repl;
use clap_v3::{App, AppSettings, Arg, ArgMatches};
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

pub struct ContextualCommand {}
impl Cmd for ContextualCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        let matches = clap_args().try_get_matches_from(args)?;
        match matches.subcommand() {
            ("run", Some(matches)) => ContextualCommand::execute_run(repl, matches),
            ("extract", Some(matches)) => ContextualCommand::extract(repl, matches),
            ("info", Some(matches)) => ContextualCommand::info(repl, matches),
            ("delete", Some(matches)) => match matches.subcommand() {
                ("options", Some(matches)) => ContextualCommand::delete_options(repl, matches),
                _ => Err(CmdError::NotFound)
            },
            _ => Err(CmdError::NotFound),
        }
    }
}
impl ContextualCommand {
    fn execute_run(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        if repl.request().is_none() {
            return Err(CmdError::NotFound);
        }
        let req = String::from(repl.request().unwrap());
        BaseCommand::run(repl, matches, req.as_ref())
    }

    fn extract(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        if repl.request().is_none() {
            return Err(CmdError::ArgsError(String::from("Extract is only available in a request specific context. Try setting a request first.")));
        }
        let request = repl.request().unwrap();
        let extraction_source = matches.value_of("type").unwrap();
        let key = matches.value_of("key").unwrap();
        let var = matches.value_of("variable").unwrap();

        let opt = RequestOutput::new(request, var, extraction_source, key);
        repl.db.create_output_option(opt)?;
        Ok(())
    }

    fn info(repl: &mut Repl, _matches: &ArgMatches) -> Result<(), CmdError> {
        if repl.request().is_none() {
            return Err(CmdError::ArgsError(String::from("Info is only available in a request specific context. Try setting a request first.")));
        }
        let req = repl.request().unwrap();
        // display request, input options, and output options
        let req = repl.db.get_request(req)?;
        // get options for this request
        let input_opts: Vec<RequestInput> = repl
            .db
            .get_input_options()?
            .into_iter()
            .filter(|x| req.name() == x.request_name())
            .collect();
        let output_opts: Vec<RequestOutput> = repl
            .db
            .get_output_options()?
            .into_iter()
            .filter(|x| req.name() == x.request_name())
            .collect();

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
            Cell::new(req.headers().as_ref().unwrap_or(&String::from(""))),
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
                .load_preset(crate::TABLE_FORMAT)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_table_width(width);
            println!("  Input Options");
            table.set_header(vec!["name", "current value"]);
            for opt in input_opts {
                table.add_row(vec![opt.option_name(), opt.value().unwrap_or("")]);
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
                .load_preset(crate::TABLE_FORMAT)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_table_width(width);
            println!("  Output Options");
            table.set_header(vec!["output variable", "type", "source"]);
            for opt in output_opts {
                table.add_row(vec![opt.option_name(), opt.extraction_source(), opt.path()]);
            }
            for line in table.to_string().split('\n') {
                println!("  {}", line);
            }
            println!();
        }

        Ok(())
    }

    fn delete_options(repl: &mut Repl, matches: &ArgMatches) -> Result<(), CmdError> {
        if repl.request().is_none() {
            return Err(CmdError::ArgsError(String::from("Delete option is only available in a request specific context. Try setting a request first.")));
        }
        let request = repl.request().unwrap();
        let options: Vec<&str> = matches.values_of("option").unwrap().collect();
        for option in options {
            // TODO: notify when not found
            // TODO: flag for input or output option
            repl.db.delete_input_option_by_name(request, option)?;
            repl.db.delete_output_option_by_name(request, option)?;
        }
        Ok(())
    }
}

fn clap_args() -> clap_v3::App<'static> {
    // TODO: can this be a sinlge static clap_v3::App variable?
    let delete_options = App::new("options")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowExternalSubcommands)
        .about("Delete input or output options")
        .visible_aliases(&["option", "opts", "opt", "o"])
        .arg(
            Arg::with_name("option")
                .help("Option to delete")
                .required(true)
                .multiple(true),
        );
    App::new("repost")
        .setting(AppSettings::NoBinaryName)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowExternalSubcommands)
        .subcommand(
            App::new("run")
                .about("Run a named HTTP request")
                .visible_aliases(&["r"])
                // TODO
                // .arg(
                //     Arg::with_name("request")
                //         .help("Request to run")
                //         .required(false)
                //         .multiple(false), // TODO run multiple in a row
                // )
                .arg(
                    Arg::with_name("quiet")
                        .help("Verbose output")
                        .short('q')
                        .long("quiet")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("extract")
                .about("Extract data from the output of a request")
                .visible_aliases(&["ex"])
                .arg(
                    Arg::with_name("type")
                        .help("Body or header extraction")
                        .required(true)
                        .possible_values(&["body", "header"]),
                )
                .arg(
                    Arg::with_name("key")
                        .help("Key to extract - header name or JSON body path")
                        .required(true),
                )
                .arg(
                    Arg::with_name("variable")
                        .help("Variable to store the extracted data")
                        .short('t')
                        .long("to-var")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("info")
                .about("Print information about the current request")
                .visible_aliases(&["i"]),
        )
        .subcommand(
            App::new("delete")
                .setting(AppSettings::VersionlessSubcommands)
                .setting(AppSettings::DisableHelpSubcommand)
                .setting(AppSettings::NoAutoHelp)
                .about("Delete named requests or variables")
                .visible_aliases(&["remove", "del", "rm"])
                .subcommand(delete_options),
        )
}
