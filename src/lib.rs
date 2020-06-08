use std::io;
use std::io::prelude::*;

pub struct Repl {
    prompt: String,
    environment: Option<String>,
    request: Option<String>,
}

impl Repl {
    pub fn new() -> Repl {
        Repl {
            prompt: String::from("[repost]"),
            environment: None,
            request: None,
        }
    }

    pub fn get_input(&self, mut input: &mut String) -> Option<()> {
        let stdin = io::stdin();

        print!("{} > ", self.prompt);
        io::stdout().flush().unwrap();
        input.clear();

        // read line and exit on EOF
        if stdin.read_line(&mut input).unwrap() == 0 {
            println!("goodbye");
            return None
        }
        // remove trailing newline
        input.pop();
        Some(())
    }

    pub fn execute(&self, command: &str) -> Result<(), String> {
        // TODO: investigate using shlex
        let args: Vec<&str> = command.split_whitespace().collect();
        if args.len() == 0 {
            return Ok(());
        }
        if self.environment == None && self.request == None {
            self.execute_base(args)
        } else {
            Ok(())
        }
    }

    fn execute_base(&self, args: Vec<&str>) -> Result<(), String> {
        match args[0] {
            "show" => self.execute_show(args),
            "create" => self.execute_create(args),
            "use" => self.execute_use(args),
            x => Err(format!("Invalid command: {}.", x)),
        }
    }

    fn execute_show(&self, args: Vec<&str>) -> Result<(), String> {
        if args.len() != 2 {
            println!("Show various saved data\n\nUsage: show <requests|variables|environments>\n");
        }
        Ok(())
    }
    fn execute_create(&self, args: Vec<&str>) -> Result<(), String> {
        Err(String::from("not implemented: create"))
    }
    fn execute_use(&self, args: Vec<&str>) -> Result<(), String> {
        Err(String::from("not implemented: use"))
    }
}
