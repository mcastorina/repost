use repost::Repl;

#[tokio::main]
async fn main() {
    let mut repl = Repl::new().await;
    let mut input = String::new();

    loop {
        if repl.get_input(&mut input).is_none() {
            break;
        }

        if let Err(x) = repl.execute(&input).await {
            eprintln!("[!] {}", x);
        }
    }
}
