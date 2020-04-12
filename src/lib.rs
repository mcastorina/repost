pub mod config {
    use clap_v3::{App, Arg};

    #[derive(Debug)]
    pub enum Command {
        CreateRequest {
            name: String,
            url: String,
            method: Option<String>,
            body: Option<String>,
        },
        CreateVariable {
            name: String,
            env_vals: Vec<(String, String)>,
        },
        None,
    }

    pub fn parse_args(args: &[String]) -> Command {
        let contains_equal = |val: String| {
            // val is the argument value passed in by the user
            if val.contains("=") {
                Ok(())
            } else {
                Err(String::from("missing '=' in argument"))
            }
        };
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
            .get_matches_from(args);

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
                    // We can unwrap because name and url are required
                    let name = cr_matches.value_of("name").unwrap();
                    let url = cr_matches.value_of("url").unwrap();
                    let method: Option<String>;
                    let body: Option<String>;
                    if let Some(m) = cr_matches.value_of("method") {
                        method = Some(String::from(m));
                    } else if let Some(m) = name_to_method(&name) {
                        method = Some(m)
                    } else {
                        method = None
                    }
                    if let Some(d) = cr_matches.value_of("data") {
                        body = Some(String::from(d));
                    } else {
                        body = None;
                    }

                    return Command::CreateRequest {
                        name: String::from(name),
                        url: String::from(url),
                        method: method,
                        body: body,
                    };
                }
                ("variable", Some(cv_matches)) => {
                    let name = cv_matches.value_of("name").unwrap();
                    let env_vals = cv_matches
                        .values_of("environment=value")
                        .unwrap()
                        .map(|s| {
                            let mut items = s.splitn(2, "=");
                            // We can unwrap because this argument is guaranteed to have one '='
                            (
                                String::from(items.next().unwrap()),
                                String::from(items.next().unwrap()),
                            )
                        })
                        .collect();
                    return Command::CreateVariable {
                        name: String::from(name),
                        env_vals: env_vals,
                    };
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
            }
            // TODO: print help when no subcommand was used
            ("", None) => println!("No subcommand was used"),
            _ => unreachable!(),
        }
        Command::None
    }
    fn name_to_method(name: &str) -> Option<String> {
        let name = name.to_lowercase();
        if name.starts_with("get") {
            Some(String::from("GET"))
        } else if name.starts_with("create") {
            Some(String::from("POST"))
        } else if name.starts_with("delete") {
            Some(String::from("DELETE"))
        } else if name.starts_with("replace") {
            Some(String::from("PUT"))
        } else if name.starts_with("update") {
            Some(String::from("PATCH"))
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_name_to_method() {
            assert_eq!(name_to_method("get_something"), Some(String::from("GET")));
        }
    }
}
