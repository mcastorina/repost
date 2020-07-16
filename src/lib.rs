mod bastion;
mod cmd;
mod db;
pub mod error;

use bastion::Bastion;
use error::Result;

use std::env;
use std::fs;
use std::io::{self, prelude::*};
use std::path::Path;

pub struct Repl {
    bastion: Bastion,
}

impl Repl {
    pub fn new() -> Result<Repl> {
        let base_dir = env::var("XDG_CONFIG_DIR");
        let home_dir = env::var("HOME");
        let root = match (base_dir, home_dir) {
            (Err(_), Err(_)) => {
                // ask the user where to create the files
                let mut s = String::new();
                if get_input("Could not find a viable location for repost's files. Please provide a directory to use", &mut s).is_none() {
                    println!("Quitting..");
                    std::process::exit(1);
                }
                Path::new(&s).to_path_buf()
            }
            (Ok(conf), _) => Path::new(&conf).join("repost"),
            (_, Ok(home)) => Path::new(&home).join(".repost"),
        };
        if !root.exists() {
            let mut s = String::new();
            if get_input(
                &format!("Directory {:?} does not exist. Create it? [y/N]", root),
                &mut s,
            )
            .is_none()
                || s.len() == 0
            {
                println!("Quitting..");
                std::process::exit(1);
            }
            let s = s.chars().next().unwrap();
            if s == 'y' || s == 'Y' {
                if let Err(x) = fs::create_dir_all(&root) {
                    println!("Error creating directory: {}", x);
                    println!("Quitting..");
                    std::process::exit(1);
                }
            } else {
                std::process::exit(1);
            }
        }

        let repl = Repl {
            bastion: Bastion::new(root)?,
        };
        Ok(repl)
    }

    pub fn get_input(&mut self, input: &mut String) -> Option<()> {
        self.bastion.get_input(input)
    }

    pub fn execute(&mut self, command: &str) -> Result<()> {
        self.bastion.execute(command)
    }
}

fn get_input(prompt: &str, mut input: &mut String) -> Option<()> {
    let stdin = io::stdin();

    print!("{}: ", prompt);
    io::stdout().flush().unwrap();
    input.clear();

    // read line and exit on EOF
    if stdin.read_line(&mut input).unwrap() == 0 {
        return None;
    }
    // remove trailing newline
    input.pop();
    Some(())
}
