use repost::{self, cli};
use std::env;

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let config = cli::parse_args(&args)?;
    println!("{:?}", config);
    repost::run(config)?;
    Ok(())
}
