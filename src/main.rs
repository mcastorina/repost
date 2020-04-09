use clap_v3::App;

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
                .arg("<object> 'type of object to create'"),
        )
        .subcommand(
            App::new("list")
                .about("List HTTP requests or variables")
                .arg("[object] 'type of object to list (default: all)'"),
        )
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    if matches.is_present("verbose") {
        println!("Verbose mode is on");
    }

    // Check sub-commands
match matches.subcommand_name() {
        Some("create") => println!("create!"),
        Some("list") => println!("list!"),
        None => println!("No subcommand was used"),
        _ => println!("Unrecognized subcommand"),
    }
}
