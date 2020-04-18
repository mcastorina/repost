use repost::{self, config};
use std::env;

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let config = config::parse_args(&args);
    println!("{:?}", config);
    repost::run(config)?;
    Ok(())
}
