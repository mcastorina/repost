use repost::{self, cli};
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = cli::parse_args(&args)?;
    println!("{:?}", config);
    repost::run(config)?;
    Ok(())
}
