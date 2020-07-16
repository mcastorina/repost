use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub struct LineReader {
    root: PathBuf,
    editor: Editor<LineReaderHelper>,
    base_yaml: Value,
    request_yaml: Value,
}
impl LineReader {
    pub fn new<P: AsRef<Path>>(root: P) -> LineReader {
        // setup rustyline
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let base_yaml: Value = serde_yaml::from_str(include_str!("clap/base.yml")).unwrap();
        let request_yaml: Value = serde_yaml::from_str(include_str!("clap/request.yml")).unwrap();
        let h = LineReaderHelper::new(&base_yaml);
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(h));

        let mut lr = LineReader {
            root: root.as_ref().to_path_buf(),
            editor: rl,
            base_yaml,
            // TODO: add set difference of (base - request) subcommands to request_yaml
            request_yaml,
        };
        lr.editor.load_history(&lr.history_filepath()).unwrap_or(());
        lr
    }

    pub fn read_line(&mut self, input: &mut String, prompt: String) -> Option<()> {
        let readline = self.editor.readline(&prompt);
        match readline {
            Ok(line) => {
                self.editor.add_history_entry(line.as_str());
                *input = line;
                Some(())
            }
            Err(ReadlineError::Interrupted) => Some(()),
            Err(ReadlineError::Eof) => {
                self.editor.save_history(&self.history_filepath()).unwrap_or(());
                None
            }
            Err(_) => {
                self.editor.save_history(&self.history_filepath()).unwrap_or(());
                None
            }
        }
    }
    fn history_filepath(&self) -> PathBuf {
        self.root.join("repost_history")
    }

    pub fn set_base(&mut self) {
        self.editor.helper_mut().unwrap().root_yaml = self.base_yaml.clone();
    }
    pub fn set_request(&mut self) {
        self.editor.helper_mut().unwrap().root_yaml = self.request_yaml.clone();
    }

    pub fn environment_completions(&mut self, env: Vec<String>) {
        self.editor.helper_mut().unwrap().environments = env;
    }
    pub fn request_completions(&mut self, reqs: Vec<String>) {
        self.editor.helper_mut().unwrap().requests = reqs;
    }
    pub fn variable_completions(&mut self, vars: Vec<String>) {
        self.editor.helper_mut().unwrap().variables = vars;
    }
    pub fn input_option_completions(&mut self, opts: Vec<String>) {
        self.editor.helper_mut().unwrap().input_options = opts;
    }
    pub fn workspace_completions(&mut self, ws: Vec<String>) {
        self.editor.helper_mut().unwrap().workspaces = ws;
    }
}

pub struct LineReaderHelper {
    root_yaml: Value,
    environments: Vec<String>,
    requests: Vec<String>,
    variables: Vec<String>,
    input_options: Vec<String>,
    workspaces: Vec<String>,
}
impl LineReaderHelper {
    fn new(base_yaml: &Value) -> LineReaderHelper {
        LineReaderHelper {
            root_yaml: base_yaml.clone(),
            environments: vec![],
            requests: vec![],
            variables: vec![],
            input_options: vec![],
            workspaces: vec![],
        }
    }
}

impl Helper for LineReaderHelper {}
impl Hinter for LineReaderHelper {}
impl Highlighter for LineReaderHelper {}
impl Validator for LineReaderHelper {}
impl Completer for LineReaderHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = format!("{}_", line);
        // TODO: store CommandStructure instead of YAML
        let mut cmd = CommandStructure::from(&self.root_yaml);
        // TODO: automatically detect where to use these completions from arg
        if let Some(cmd) = cmd.get_child_mut(vec!["set", "environment"]) {
            cmd.completions = self.environments.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["set", "request"]) {
            cmd.completions = self.requests.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["set", "variable"]) {
            cmd.completions = self.variables.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["set", "workspace"]) {
            cmd.completions = self.workspaces.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["set", "option"]) {
            cmd.completions = self.input_options.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["delete", "requests"]) {
            cmd.completions = self.requests.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["delete", "variables"]) {
            cmd.completions = self.variables.clone();
        }
        if let Some(cmd) = cmd.get_child_mut(vec!["run"]) {
            cmd.completions = self.requests.clone();
        }
        let mut cmd = &cmd;
        // split line
        let mut tokens = line.split_whitespace();
        let mut last_token = String::from(tokens.next_back().unwrap());
        last_token.pop();

        for tok in tokens {
            let next_cmd = cmd.get_child(tok);
            if next_cmd.is_none() {
                return Ok((pos, vec![]));
            }
            cmd = next_cmd.unwrap();
        }

        let candidates: Vec<String> = cmd
            .completions
            .to_vec()
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

struct CommandStructure {
    name: String,                         // command name
    aliases: Vec<String>,                 // possible aliases for name
    completions: Vec<String>,             // subcommand names
    children: Vec<Box<CommandStructure>>, // list of commands (name should match a completion)
}

impl CommandStructure {
    fn new(name: &str) -> CommandStructure {
        CommandStructure {
            name: String::from(name),
            aliases: vec![],
            completions: vec![],
            children: vec![],
        }
    }
    fn get_child<'a>(&'a self, name_or_alias: &str) -> Option<&'a CommandStructure> {
        for cs in self.children.iter() {
            if cs.name == name_or_alias || cs.aliases.iter().any(|e| e == name_or_alias) {
                return Some(cs);
            }
        }
        None
    }
    fn get_child_mut<'a>(
        &'a mut self,
        names_or_aliases: Vec<&str>,
    ) -> Option<&'a mut CommandStructure> {
        let mut cs = self;
        for name_or_alias in names_or_aliases {
            let child = |cs: &'a mut CommandStructure| -> Option<&'a mut CommandStructure> {
                for child in cs.children.iter_mut() {
                    if child.name == name_or_alias
                        || child.aliases.iter().any(|e| e == name_or_alias)
                    {
                        return Some(child);
                    }
                }
                None
            }(cs);
            if child.is_none() {
                return None;
            }
            cs = child.unwrap();
        }
        Some(cs)
    }
}

impl From<&serde_yaml::Value> for CommandStructure {
    fn from(value: &serde_yaml::Value) -> CommandStructure {
        let (name, value) = get_map(value);
        let mut cs = CommandStructure::new(name);
        cs.aliases = get_aliases(&value)
            .into_iter()
            .map(|x| String::from(x))
            .collect();
        cs.completions = get_sub_names(&value)
            .into_iter()
            .map(|x| String::from(x))
            .collect();

        let subcommands = value.get("subcommands");
        if subcommands.is_none() {
            return cs;
        }
        let subcommands = subcommands.unwrap();
        if let Value::Sequence(cmds) = subcommands {
            for cmd in cmds {
                cs.children.push(Box::new(CommandStructure::from(cmd)));
            }
        }
        cs
    }
}
fn get_map(cmd: &Value) -> (&str, &Value) {
    let name = get_name(cmd).unwrap();
    (name, cmd.get(name).unwrap_or(cmd))
}
fn get_aliases(cmd: &Value) -> Vec<&str> {
    let mut names = vec![];
    if let Value::Mapping(m) = cmd {
        for kv in m.iter() {
            let (k, v) = kv;
            match k.as_str().unwrap() {
                "aliases" | "visible_aliases" => {
                    if let Value::Sequence(aliases) = v {
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
fn get_name(cmd: &Value) -> Option<&str> {
    if let Some(v) = cmd.get("name") {
        if v.is_string() {
            return Some(v.as_str().unwrap());
        }
    }
    if let Value::Mapping(m) = cmd {
        for kv in m.iter() {
            let (k, _) = kv;
            // should only be one mapping
            return k.as_str();
        }
    }
    None
}
fn get_sub_names(cmd: &Value) -> Vec<&str> {
    let mut names = vec![];
    let subcommands = cmd.get("subcommands");
    if subcommands.is_none() {
        return names;
    }
    let subcommands = subcommands.unwrap();
    if let Value::Sequence(cmds) = subcommands {
        for cmd in cmds {
            let name = get_name(cmd);
            if name.is_some() {
                names.push(name.unwrap());
            }
        }
    }
    names
}
