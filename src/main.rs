use repost::Repl;

fn main() -> Result<(), String> {
    let mut input = String::new();
    let repl = Repl::new()?;

    loop {
        if repl.get_input(&mut input) == None {
            break;
        }

        if let Err(x) = repl.execute(&input) {
            eprintln!("[!] {}", x);
        }
    }

    Ok(())
}
