pub mod cmd;
pub mod db;

use cmd::{Cmd, CmdError};
use colored::*;
use db::{Db, Environment, Request, RequestInput, RequestOutput, Variable};
use regex::Regex;
use reqwest::header::HeaderMap;
use std::fs;
use std::io::{self, prelude::*};

use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use std::collections::HashMap;

use std::borrow::Cow::{self, Borrowed, Owned};

const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub struct Repl {
    prompt: String,
    workspace: String,
    db: Db,
    environment: Option<String>,
    request: Option<String>,
    line_reader: Editor<ReplHelper>,
}

impl Repl {
    pub fn new() -> Result<Repl, CmdError> {
        // setup rustyline
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let h = ReplHelper {
            completer: CmdCompleter {},
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "".to_owned(),
            validator: MatchingBracketValidator::new(),
        };
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(h));
        rl.load_history("history.txt").unwrap_or(());

        let mut repl = Repl {
            prompt: String::from("[repost]"),
            workspace: String::from("repost"),
            db: Db::new("repost.db")?,
            environment: None,
            request: None,
            line_reader: rl,
        };
        repl.update_all_options()?;
        repl.update_prompt();
        Ok(repl)
    }

    pub fn get_input(&mut self, mut input: &mut String) -> Option<()> {
        // set the prompt and completer
        let prompt = format!("{} > ", self.prompt);
        self.line_reader.helper_mut().unwrap().colored_prompt = prompt.clone();
        // set completer based on current state of repl
        self.line_reader.helper_mut().unwrap().completer = CmdCompleter {};

        // read the line
        let readline = self.line_reader.readline(&prompt);
        match readline {
            Ok(line) => {
                self.line_reader.add_history_entry(line.as_str());
                *input = line;
                Some(())
            }
            Err(ReadlineError::Interrupted) => {
                Some(())
            }
            Err(ReadlineError::Eof) => {
                self.line_reader.save_history("history.txt").unwrap_or(());
                None
            }
            Err(_) => {
                self.line_reader.save_history("history.txt").unwrap_or(());
                None
            }
        }
    }

    fn cmds() -> Vec<Box<dyn Cmd>> {
        vec![
            Box::new(cmd::ContextualCommand {}),
            Box::new(cmd::BaseCommand {}),
        ]
    }

    pub fn execute(&mut self, command: &str) -> Result<(), CmdError> {
        let args: Vec<String> = shlex::split(command).unwrap_or(vec![]);
        if args.len() == 0 {
            return Ok(());
        }
        let args = args.iter().map(|x| x.as_ref()).collect();
        for cmd in Repl::cmds() {
            let ret = cmd.execute(self, &args);
            match ret {
                Ok(x) => return Ok(x),
                Err(x) => match x {
                    CmdError::NotFound => (),
                    _ => return Err(x),
                },
            }
        }
        Err(CmdError::NotFound)
    }

    fn update_prompt(&mut self) {
        let mut prompt = format!("[{}]", &self.workspace.yellow());
        if let Some(x) = &self.environment {
            prompt = format!("{}[{}]", prompt, x.bold().cyan());
        }
        if let Some(x) = &self.request {
            prompt = format!("{}[{}]", prompt, x.bold().green());
        }
        self.prompt = prompt;
    }

    pub fn update_environment(&mut self, environment: Option<&str>) -> Result<(), CmdError> {
        if let Some(environment) = environment {
            if !self.db.environment_exists(environment)? {
                return Err(CmdError::ArgsError(format!(
                    "Environment not found: {}",
                    environment,
                )));
            }
            self.environment = Some(String::from(environment));
        } else {
            self.environment = None;
        }
        self.update_all_options()?;
        self.update_prompt();
        Ok(())
    }

    pub fn update_workspace(&mut self, workspace: &str) -> Result<(), CmdError> {
        self.workspace = String::from(workspace);
        self.db = Db::new(format!("{}.db", workspace).as_ref())?;
        if let Some(environment) = self.environment.as_ref() {
            if !self.db.environment_exists(environment)? {
                self.environment = None;
                self.request = None;
            }
            // TODO: check request exists in new workspace
        }
        self.update_all_options()?;
        self.update_prompt();
        Ok(())
    }

    pub fn update_request(&mut self, request: Option<&str>) -> Result<(), CmdError> {
        if let Some(request) = request {
            self.db.get_request(request)?;
            self.request = Some(String::from(request));
        } else {
            self.request = None;
        }
        self.update_prompt();
        Ok(())
    }

    fn update_all_options(&self) -> Result<(), CmdError> {
        // get all unique request_name in options table
        let request_names = self.db.get_unique_request_names_from_options()?;
        // call self.update_options_for_request(req)
        for name in request_names {
            self.update_options_for_request(name.as_ref())?;
        }
        Ok(())
    }
    fn update_options(&self, opts: Vec<RequestInput>) -> Result<(), CmdError> {
        if self.environment.is_none() {
            // if the current environment is none, clear the value
            for mut opt in opts {
                opt.update_value(None);
                self.db.update_input_option(opt)?;
            }
        } else {
            // else set option.value according to the environment
            for mut opt in opts {
                let mut var: Vec<Variable> = self
                    .db
                    .get_variables()?
                    .into_iter()
                    .filter(|var| {
                        var.environment() == self.environment().unwrap()
                            && var.name() == opt.option_name()
                    })
                    .collect();
                if var.len() == 0 {
                    opt.update_value(None);
                } else if var.len() == 1 {
                    let var = &mut var[0];
                    opt.update_value(var.consume_value());
                } else {
                    unreachable!();
                }
                self.db.update_input_option(opt)?;
            }
        }
        Ok(())
    }
    fn update_options_for_request(&self, request: &str) -> Result<(), CmdError> {
        // get all options for request
        let opts: Vec<RequestInput> = self
            .db
            .get_input_options()?
            .into_iter()
            .filter(|opt| opt.request_name() == request)
            .collect();
        self.update_options(opts)
    }
    fn update_options_for_variable(&self, variable: &str) -> Result<(), CmdError> {
        // get all opts where option_name == variable_name
        let opts: Vec<RequestInput> = self
            .db
            .get_input_options()?
            .into_iter()
            .filter(|opt| opt.option_name() == variable)
            .collect();
        self.update_options(opts)
    }

    fn environment(&self) -> Option<&str> {
        match &self.environment {
            Some(x) => Some(x.as_ref()),
            None => None,
        }
    }
    fn request(&self) -> Option<&str> {
        match &self.request {
            Some(x) => Some(x.as_ref()),
            None => None,
        }
    }

    fn get_requests(&self) -> Result<Vec<Request>, CmdError> {
        Ok(self.db.get_requests()?)
    }
    fn get_variables(&self) -> Result<Vec<Variable>, CmdError> {
        let mut result = self.db.get_variables()?;
        if let Some(env) = self.environment() {
            result = result
                .into_iter()
                .filter(|x| x.environment == env)
                .collect();
        }
        Ok(result)
    }
    fn get_input_options(&self) -> Result<Vec<RequestInput>, CmdError> {
        let mut result = self.db.get_input_options()?;
        if let Some(req) = self.request() {
            result = result
                .into_iter()
                .filter(|x| x.request_name() == req)
                .collect();
        }
        Ok(result)
    }
    fn get_environments(&self) -> Result<Vec<Environment>, CmdError> {
        Ok(self.db.get_environments()?)
    }
    fn get_workspaces(&self) -> Result<Vec<String>, CmdError> {
        // TODO: use a struct if this is needed in other operations
        //       for now, it is only being used to print the workspaces
        //       so we prefix the vector with the header "workspace"
        let mut result = vec![String::from("workspace")];
        let paths = fs::read_dir("./")?;
        for path in paths {
            let path = path?.path();
            // filter out .db extensions
            match path.extension() {
                Some(x) => {
                    if x != "db" {
                        continue;
                    }
                }
                _ => continue,
            }
            let ws = path.file_stem().unwrap();
            if let Some(x) = ws.to_str() {
                result.push(String::from(x));
            }
        }
        Ok(result)
    }
    fn set_option(&self, opt_name: &str, value_ref: Option<&str>) -> Result<(), CmdError> {
        if self.request.is_none() {
            return Err(CmdError::ArgsError(String::from("Set option is only available in a request specific context. Try setting a request first.")));
        }

        let value = match value_ref {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        // Set option only applies to input options
        let opt = RequestInput::new(self.request().unwrap(), opt_name, value);
        self.db.update_input_option(opt)?;
        println!(
            "{}",
            format!("{} => {}", opt_name, value_ref.unwrap_or("None")).bright_black()
        );
        Ok(())
    }

    fn body_to_var(&self, opt: &RequestOutput, body: &str) -> Result<Variable, CmdError> {
        let value = get_json_value(body, opt.path())?;
        Ok(Variable::new(
            opt.option_name(),
            self.environment().unwrap_or(""), // TODO: allow None environment for variable
            value.as_str(),
            None,
        ))
    }
    fn header_to_var(
        &self,
        opt: &RequestOutput,
        headers: &HeaderMap,
    ) -> Result<Variable, CmdError> {
        let value = match headers.get(opt.path()) {
            Some(x) => Some(x.to_str().unwrap()),
            None => None,
        };
        if value.is_none() {
            return Err(CmdError::ParseError);
        }
        Ok(Variable::new(
            opt.option_name(),
            self.environment().unwrap_or(""), // TODO: allow None environment for variable
            value,
            None,
        ))
    }
}

fn get_json_value(data: &str, query: &str) -> Result<serde_json::Value, CmdError> {
    // TODO: Result
    let mut v: serde_json::Value = serde_json::from_str(data)?;
    let mut result: &mut serde_json::Value = &mut v;

    let re = Regex::new(r"\[(\d+)\]")?;
    for token in query.split(".") {
        let name = token.splitn(2, "[").next().unwrap();
        let mr = result.get_mut(name);
        if mr.is_none() {
            return Err(CmdError::ParseError);
        }
        result = mr.unwrap();
        for cap in re.captures_iter(token) {
            let num: usize = cap[1].parse()?;
            let mr = result.get_mut(num);
            if mr.is_none() {
                return Err(CmdError::ParseError);
            }
            result = mr.unwrap();
        }
    }
    Ok(result.take())
}

impl From<serde_json::Error> for CmdError {
    fn from(_err: serde_json::Error) -> CmdError {
        CmdError::ParseError
    }
}
impl From<regex::Error> for CmdError {
    fn from(_err: regex::Error) -> CmdError {
        CmdError::ParseError
    }
}
impl From<std::num::ParseIntError> for CmdError {
    fn from(_err: std::num::ParseIntError) -> CmdError {
        CmdError::ParseError
    }
}

struct ReplHelper {
    completer: CmdCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}
impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ReplHelper {
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        // self.hinter.hint(line, pos, ctx)
        None
    }
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ReplHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

struct CmdCompleter {}

impl Completer for CmdCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = format!("{}_", line);
        let cmds = get_cmd_lut();
        // split line
        let mut tokens = line.split_whitespace();
        let mut last_token = String::from(tokens.next_back().unwrap());
        last_token.pop();
        let key = tokens
            .map(|x| String::from(x))
            .collect::<Vec<String>>()
            .join(" ");

        let candidates = cmds.get(&key);
        if candidates.is_none() {
            return Ok((pos, vec![]));
        }
        let candidates = candidates.unwrap().to_vec();
        let candidates: Vec<String> = candidates
            .into_iter()
            .filter(|x| x.starts_with(&last_token))
            .collect();
        Ok((
            line.len() - last_token.len() - 1,
            candidates
                .iter()
                .map(|cmd| Pair {
                    display: String::from(cmd),
                    replacement: format!("{} ", cmd),
                })
                .collect(),
        ))
    }
}
// TODO: refactor
fn get_cmd_lut() -> HashMap<String, Vec<String>> {
    let yaml: serde_yaml::Value = serde_yaml::from_str(include_str!("cmd/cli.yml")).unwrap();
    let mut map = build_lut_r(&yaml, "");
    // base commands
    map.insert(
        String::from(""),
        get_sub_names(&yaml)
            .into_iter()
            .map(|x| String::from(x))
            .collect(),
    );
    map
}
fn build_lut_r(root: &serde_yaml::Value, prefix: &str) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let subcommands = root.get("subcommands");
    if subcommands.is_none() {
        return map;
    }
    let subcommands = subcommands.unwrap();
    if let serde_yaml::Value::Sequence(cmds) = subcommands {
        for cmd in cmds {
            let (name, cmd) = get_map(cmd);
            let mut aliases = get_aliases(cmd);
            aliases.push(name);
            let sub_names = get_sub_names(cmd);
            for alias in aliases {
                let p = match prefix {
                    "" => String::from(alias),
                    _ => format!("{} {}", prefix, alias),
                };
                map.insert(
                    String::from(&p),
                    sub_names.iter().map(|&x| String::from(x)).collect(),
                );
                let child = build_lut_r(cmd, &p);
                map.extend(child);
            }
        }
    }
    map
}
fn get_map(cmd: &serde_yaml::Value) -> (&str, &serde_yaml::Value) {
    let name = get_name(cmd).unwrap();
    (name, cmd.get(name).unwrap())
}

fn get_aliases(cmd: &serde_yaml::Value) -> Vec<&str> {
    let mut names = vec![];
    if let serde_yaml::Value::Mapping(m) = cmd {
        for kv in m.iter() {
            let (k, v) = kv;
            match k.as_str().unwrap() {
                "aliases" | "visible_aliases" => {
                    if let serde_yaml::Value::Sequence(aliases) = v {
                        for alias in aliases {
                            names.push(alias.as_str().unwrap());
                        }
                    }
                }
                _ => (),
            }
        }
    }
    names
}
fn get_name(cmd: &serde_yaml::Value) -> Option<&str> {
    if let serde_yaml::Value::Mapping(m) = cmd {
        for kv in m.iter() {
            let (k, _) = kv;
            // should only be one mapping
            return k.as_str();
        }
    }
    None
}
fn get_sub_names(cmd: &serde_yaml::Value) -> Vec<&str> {
    let mut names = vec![];
    let subcommands = cmd.get("subcommands");
    if subcommands.is_none() {
        return names;
    }
    let subcommands = subcommands.unwrap();
    if let serde_yaml::Value::Sequence(cmds) = subcommands {
        for cmd in cmds {
            let name = get_name(cmd);
            if name.is_some() {
                names.push(name.unwrap());
            }
        }
    }
    names
}
