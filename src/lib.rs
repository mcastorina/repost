use rusqlite::{Connection, NO_PARAMS};
use std::io;
use std::io::prelude::*;

pub struct Repl {
    prompt: String,
    workspace: String,
    db: Db,
    environment: Option<String>,
    request: Option<String>,
}

impl Repl {
    pub fn new() -> Result<Repl, String> {
        Ok(Repl {
            prompt: String::from("[repost]"),
            workspace: String::from("repost"),
            db: Db::new("repost.db")?,
            environment: None,
            request: None,
        })
    }

    pub fn get_input(&self, mut input: &mut String) -> Option<()> {
        let stdin = io::stdin();

        print!("{} > ", self.prompt);
        io::stdout().flush().unwrap();
        input.clear();

        // read line and exit on EOF
        if stdin.read_line(&mut input).unwrap() == 0 {
            println!("goodbye");
            return None;
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
        match Repl::get_table_from_alias(args[1]) {
            Some(table) => self.db.get_table(&table),
            _ => Err(format!("Invalid argument: {}", args[1])),
        }
    }
    fn execute_create(&self, args: Vec<&str>) -> Result<(), String> {
        Err(String::from("not implemented: create"))
    }
    fn execute_use(&self, args: Vec<&str>) -> Result<(), String> {
        Err(String::from("not implemented: use"))
    }

    fn get_table_from_alias(alias: &str) -> Option<String> {
        match alias {
            "r" | "req" | "reqs" | "request" | "requests" => Some(String::from("requests")),
            "v" | "var" | "vars" | "variable" | "variables" => Some(String::from("variables")),
            "e" | "env" | "envs" | "environment" | "environments" => {
                Some(String::from("environments"))
            }
            _ => None,
        }
    }
}

#[macro_use] extern crate prettytable;
use prettytable::{format, Table};

struct Db {
    path: String,
    conn: Connection,
}

struct Request {
    name: String,
    method: String,
    url: String,
    headers: Option<String>,
    body: Option<String>,
}

impl Db {
    fn new(path: &str) -> Result<Db, String> {
        let conn = Connection::open(path);
        if let Err(x) = conn {
            return Err(format!("Error connecting to {}: {}", path, x));
        }
        let db = Db {
            path: String::from(path),
            conn: conn.unwrap(),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), String> {
        if let Err(x) = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                  name            TEXT PRIMARY KEY,
                  method          TEXT NOT NULL,
                  url             TEXT NOT NULL,
                  headers         TEXT,
                  body            TEXT
              )",
            NO_PARAMS,
        ) {
            return Err(format!("Error creating requests table: {}", x));
        }

        if let Err(x) = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS variables (
                  rowid           INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  environment     TEXT NOT NULL,
                  value           TEXT,
                  source          TEXT,
                  timestamp       TEXT
              )",
            NO_PARAMS,
        ) {
            return Err(format!("Error creating variables table: {}", x));
        }

        Ok(())
    }

    // TODO: change to return table contents as enum
    fn get_table(&self, name: &str) -> Result<(), String> {
        match name {
            "requests" => self.get_requests(),
            "variables" => self.get_variables(),
            "environments" => self.get_environments(),
            x => Err(format!("Table {} not recognized", x)),
        }
    }
    fn get_requests(&self) -> Result<(), String> {
        let stmt = self.conn
            .prepare("SELECT name, method, url, headers, body FROM requests;");
        if let Err(x) = stmt {
            return Err(x.to_string());
        }
        let mut stmt = stmt.unwrap();

        let requests = stmt
            .query_map(NO_PARAMS, |row| {
                Ok(Request{
                    name: row.get(0).unwrap(),
                    method: row.get(1).unwrap(),
                    url: row.get(2).unwrap(),
                    headers: row.get(3).unwrap(),
                    body: row.get(4).unwrap(),
                })
            })
            .unwrap();

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.get_format().indent(2);

        table.set_titles(row!["name", "method", "url", "headers", "body?"]);
        for req in requests {
            let req = req.unwrap();
            let headers = req.headers.unwrap_or(String::from(""));
            let body: String;
            if req.body.is_some() {
                body = String::from("true");
            } else {
                body = String::from("false");
            }
            table.add_row(row![req.name, req.method, req.url, headers, body]);
        }
        println!();
        table.printstd();
        println!();
        Ok(())
    }
    fn get_variables(&self) -> Result<(), String> {
        Err(String::from("not implemented"))
    }
    fn get_environments(&self) -> Result<(), String> {
        let stmt = self.conn
            .prepare("SELECT DISTINCT environment FROM variables;");
        if let Err(x) = stmt {
            return Err(x.to_string());
        }
        let mut stmt = stmt.unwrap();

        let envs = stmt
            .query_map(NO_PARAMS, |row| {
                let name: String = row.get(0).unwrap();
                Ok(name)
            })
            .unwrap();

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(row!["environments"]);
        for env in envs {
            table.add_row(row![env.unwrap()]);
        }
        table.printstd();
        Ok(())
    }
}
