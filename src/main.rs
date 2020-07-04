use repost::Repl;

fn main() -> Result<(), String> {
    let mut repl = Repl::new()?;
    repl.run();

    Ok(())
}
