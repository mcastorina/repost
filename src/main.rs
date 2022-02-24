use repost::Result;
use repost::{Repl, ReplConfig};

use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let mut repl = Repl::new(ReplConfig {
        data_dir: PathBuf::from("/tmp"),
    })
    .await?;
    let mut input = String::new();

    loop {
        if repl.get_input(&mut input).is_none() {
            break;
        }
        if input == "" {
            continue;
        }

        if let Err(x) = repl.execute(&input).await {
            eprintln!("[!] {}", x);
        }
    }

    Ok(())
}
