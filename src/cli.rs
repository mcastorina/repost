use clap_v3::{App, Arg, ArgMatches};

pub fn parse_args(args: &[String]) -> Result<Command, String> {
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
                .subcommand(request_subcommand(args))
                .subcommand(variable_subcommand(args)),
        )
        .get_matches_from(args);
    Command::from_matches(matches)
}

#[derive(Debug, PartialEq)]
pub enum Command {
    CreateRequest(CreateRequest),
    CreateVariable(CreateVariable),
    None,
}
#[derive(Debug, PartialEq)]
pub struct CreateRequest {
    pub name: String,
    pub url: String,
    pub method: Option<String>,
    pub body: Option<String>,
    pub headers: Vec<String>,
}
#[derive(Debug, PartialEq)]
pub struct CreateVariable {
    pub name: String,
    pub env_vals: Vec<(String, String)>,
}

impl Command {
    pub fn from_matches(matches: ArgMatches) -> Result<Command, String> {
        if let Some(c) = matches.value_of("config") {
            println!("Value for config: {}", c);
        }

        if matches.is_present("verbose") {
            println!("Verbose mode is on");
        }

        // Check sub-commands
        match matches.subcommand() {
            ("create", Some(create_matches)) => match create_matches.subcommand() {
                ("request", Some(cr_matches)) => return Command::cr_from_matches(cr_matches),
                ("variable", Some(cv_matches)) => return Command::cv_from_matches(cv_matches),
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
        Ok(Command::None)
    }
    fn cr_from_matches(matches: &ArgMatches) -> Result<Command, String> {
        // We can unwrap because name and url are required
        let name = String::from(matches.value_of("name").unwrap());
        let url = String::from(matches.value_of("url").unwrap());
        let method: Option<String>;
        let body = matches.value_of("data").map(|b| String::from(b));
        let headers: Vec<String> = matches
            .values_of("headers")
            .unwrap()
            .map(|h| String::from(h))
            .collect();
        if let Some(m) = matches.value_of("method") {
            method = Some(String::from(m));
        } else if let Some(m) = name_to_method(&name) {
            method = Some(m)
        } else {
            method = None
        }

        return Ok(Command::CreateRequest(CreateRequest {
            name: name,
            url: url,
            method: method,
            body: body,
            headers: headers,
        }));
    }
    fn cv_from_matches(matches: &ArgMatches) -> Result<Command, String> {
        let name = matches.value_of("name").unwrap();
        let env_vals = matches
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
        return Ok(Command::CreateVariable(CreateVariable {
            name: String::from(name),
            env_vals: env_vals,
        }));
    }
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
fn request_subcommand(_args: &[String]) -> App {
    let contains_colon = |val: String| {
        // val is the argument value passed in by the user
        if val.contains(":") {
            Ok(())
        } else {
            Err(String::from("missing ':' in argument"))
        }
    };
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
                .possible_values(&["GET", "POST", "HEAD", "PUT", "PATCH", "DELETE"]),
        )
        .arg(
            Arg::with_name("headers")
                .help("HTTP request headers")
                .short('H')
                .long("header")
                .validator(contains_colon)
                .multiple(true),
        )
        .arg("-d, --data=[DATA] 'HTTP request body'")
}
fn variable_subcommand(_args: &[String]) -> App {
    let contains_equal = |val: String| {
        // val is the argument value passed in by the user
        if val.contains("=") {
            Ok(())
        } else {
            Err(String::from("missing '=' in argument"))
        }
    };
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
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request() {
        let possible_commands = vec![
            "repost create request name url",
            "repost create req name url",
            "repost create r name url",
            "repost c request name url",
            "repost c req name url",
            "repost c r name url",
        ];
        for cmd in possible_commands {
            let args: Vec<String> = cmd.split_whitespace().map(|s| String::from(s)).collect();
            assert_eq!(
                parse_args(&args),
                Command::CreateRequest {
                    name: String::from("name"),
                    url: String::from("url"),
                    method: None,
                    body: None
                }
            );
        }
    }

    #[test]
    fn test_create_request_method() {
        let possible_commands = vec![
            "repost create request name url -m GET",
            "repost create request name url --method GET",
        ];
        for cmd in possible_commands {
            let args: Vec<String> = cmd.split_whitespace().map(|s| String::from(s)).collect();
            assert_eq!(
                parse_args(&args),
                Command::CreateRequest {
                    name: String::from("name"),
                    url: String::from("url"),
                    method: Some(String::from("GET")),
                    body: None
                }
            );
        }

        let cmd = "repost create request get_name url";
        let args: Vec<String> = cmd.split_whitespace().map(|s| String::from(s)).collect();
        assert_eq!(
            parse_args(&args),
            Command::CreateRequest {
                name: String::from("get_name"),
                url: String::from("url"),
                method: Some(String::from("GET")),
                body: None
            }
        );
    }

    #[test]
    fn test_create_request_body() {
        let possible_commands = vec![
            "repost create request name url -d yay",
            "repost create request name url --data yay",
        ];
        for cmd in possible_commands {
            let args: Vec<String> = cmd.split_whitespace().map(|s| String::from(s)).collect();
            assert_eq!(
                parse_args(&args),
                Command::CreateRequest {
                    name: String::from("name"),
                    url: String::from("url"),
                    method: None,
                    body: Some(String::from("yay"))
                }
            );
        }
    }

    #[test]
    fn test_create_variable() {
        let possible_commands = vec![
            "repost create variable name env=val",
            "repost create var name env=val",
            "repost create v name env=val",
            "repost c variable name env=val",
            "repost c var name env=val",
            "repost c v name env=val",
        ];
        for cmd in possible_commands {
            let args: Vec<String> = cmd.split_whitespace().map(|s| String::from(s)).collect();
            assert_eq!(
                parse_args(&args),
                Command::CreateVariable {
                    name: String::from("name"),
                    env_vals: vec![(String::from("env"), String::from("val"))],
                }
            );
        }
    }

    #[test]
    fn test_name_to_method() {
        assert_eq!(name_to_method("get_req"), Some(String::from("GET")));
        assert_eq!(name_to_method("GET_REQ"), Some(String::from("GET")));
        assert_eq!(name_to_method("create_req"), Some(String::from("POST")));
        assert_eq!(name_to_method("CREATE_REQ"), Some(String::from("POST")));
        assert_eq!(name_to_method("delete_req"), Some(String::from("DELETE")));
        assert_eq!(name_to_method("DELETE_REQ"), Some(String::from("DELETE")));
        assert_eq!(name_to_method("replace_req"), Some(String::from("PUT")));
        assert_eq!(name_to_method("REPLACE_REQ"), Some(String::from("PUT")));
        assert_eq!(name_to_method("update_req"), Some(String::from("PATCH")));
        assert_eq!(name_to_method("UPDATE_REQ"), Some(String::from("PATCH")));
        assert_eq!(name_to_method("req"), None);
        assert_eq!(name_to_method("REQ"), None);
    }
}
