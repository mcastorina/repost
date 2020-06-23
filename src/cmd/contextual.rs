use crate::cmd::{BaseCommand, Cmd, CmdError};
use crate::db::RequestOutput;
use crate::Repl;
use clap_v3::{App, AppSettings, Arg, ArgMatches};

pub struct ContextualCommand {}
impl Cmd for ContextualCommand {
    fn execute(&self, repl: &mut Repl, args: &Vec<&str>) -> Result<(), CmdError> {
        let matches = clap_args().try_get_matches_from(args)?;
        match matches.subcommand() {
            ("run", Some(matches)) => ContextualCommand::execute_run(repl, matches),
            ("extract", Some(matches)) => ContextualCommand::extract(repl, matches),
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
}

fn clap_args() -> clap_v3::App<'static> {
    // TODO: can this be a sinlge static clap_v3::App variable?
    App::new("repost")
        .setting(AppSettings::NoBinaryName)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpSubcommand)
        .subcommand(
            App::new("run")
                .about("Run a named HTTP request")
                .aliases(&["r"])
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
                .aliases(&["ex"])
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
}
