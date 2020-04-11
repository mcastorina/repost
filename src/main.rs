use clap_v3::{App, Arg};

fn main() {
    let matches = App::new("repost")
        .version("0.1.0") // TODO: automatic version
        .author("miccah")
        .about("repost is a tool to easily define and send HTTP requests")
        .arg("-c, --config=[FILE] 'Config file (default: $HOME/.repostrc)'")
        .arg("-v, --verbose 'Verbose output'")
        .subcommand(
            App::new("create")
                .about("Create an HTTP request or variable")
                .aliases(&["c"])
                .subcommand(
                    App::new("request")
                        .about("Create an HTTP request")
                        .aliases(&["req", "r"])
                        .arg("<name> 'Name of the request'")
                        .arg("<url> 'HTTP request URL'")
                        .arg(
                            Arg::with_name("method")
                                .help("HTTP request method")
                                .short('m')
                                .long("method")
                                .possible_values(&[
                                    "GET", "POST", "HEAD", "PUT", "PATCH", "DELETE",
                                ]),
                        )
                        .arg(
                            Arg::with_name("headers")
                                .help("HTTP request headers")
                                .short('H')
                                .long("header")
                                .multiple(true),
                        )
                        .arg("-d, --data=[DATA] 'HTTP request body'"),
                )
                .subcommand(
                    App::new("variable")
                        .about("Create a variable")
                        .aliases(&["var", "v"])
                        .arg("<name> 'Name of the variable'")
                        .arg(
                            Arg::with_name("environment=value")
                                .help("Value for environment")
                                .required(true)
                                .validator(contains_equal)
                                .multiple(true),
                        ),
                ),
        )
        .subcommand(
            App::new("list")
                .about("List HTTP requests or variables")
                // TODO: make this arg sub-commands
                .arg("[requests|variables] 'Type of resources to list (default: all)'"),
        )
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    if matches.is_present("verbose") {
        println!("Verbose mode is on");
    }

    // Check sub-commands
    match matches.subcommand() {
        ("create", Some(create_matches)) => match create_matches.subcommand() {
            ("request", Some(cr_matches)) => {
                println!(
                    "create request {} {}",
                    cr_matches.value_of("name").unwrap(),
                    cr_matches.value_of("url").unwrap()
                );
            }
            ("variable", Some(cv_matches)) => {
                println!(
                    "create variable {} {:?}",
                    cv_matches.value_of("name").unwrap(),
                    cv_matches.values_of("environment=value").unwrap().collect::<Vec<_>>(),
                );
            }
            // TODO: print help when no subcommand was used
            ("", None) => println!("No subcommand was used"),
            _ => unreachable!(),
        },
        ("list", Some(list_matches)) => {
            let mut resources = "all";
            if let Some(r) = list_matches.value_of("requests|variables") {
                resources = r;
            }
            println!("list {}", resources)
        },
        // TODO: print help when no subcommand was used
        ("", None) => println!("No subcommand was used"),
        _ => unreachable!(),
    }
}

fn contains_equal(val: String) -> Result<(), String> {
    // val is the argument value passed in by the user
    if val.contains("=") {
        Ok(())
    } else {
        Err(String::from("missing '=' in argument"))
    }
}
