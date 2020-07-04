use repost::Repl;

fn main() -> Result<(), String> {
    let mut input = String::new();
    let mut repl = Repl::new()?;
    repl.run();

    Ok(())
}
