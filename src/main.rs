use repost::error::Error;
use repost::Repl;

fn main() -> Result<(), Error> {
    let mut input = String::new();
    let mut repl = Repl::new()?;

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
