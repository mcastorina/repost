use repost::config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = config::parse_args(&args);
    println!("{:?}", config);
}
