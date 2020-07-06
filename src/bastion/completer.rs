use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use serde_yaml::Value;
use std::collections::HashMap;

pub struct LineReader {
    editor: Editor<LineReaderHelper>,
    base_yaml: Value,
    request_yaml: Value,
}
impl LineReader {
    pub fn new() -> LineReader {
        // setup rustyline
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let base_yaml: Value = serde_yaml::from_str(include_str!("clap/base.yml")).unwrap();
        let h = LineReaderHelper {
            root_yaml: base_yaml.clone(),
        };
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(h));
        rl.load_history("history.txt").unwrap_or(());

        LineReader {
            editor: rl,
            base_yaml,
            // TODO: add set difference of (base - request) subcommands to request_yaml
            request_yaml: serde_yaml::from_str(include_str!("clap/request.yml")).unwrap(),
        }
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
                self.editor.save_history("history.txt").unwrap_or(());
                None
            }
            Err(_) => {
                self.editor.save_history("history.txt").unwrap_or(());
                None
            }
        }
    }

    pub fn set_base(&mut self) {
        self.editor.helper_mut().unwrap().root_yaml = self.base_yaml.clone();
    }
    pub fn set_request(&mut self) {
        self.editor.helper_mut().unwrap().root_yaml = self.request_yaml.clone();
    }
}

pub struct LineReaderHelper {
    root_yaml: Value,
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
        let cmds = get_cmd_lut(&self.root_yaml);
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
fn get_cmd_lut(yaml: &Value) -> HashMap<String, Vec<String>> {
    let mut map = build_lut_r(yaml, "");
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
fn build_lut_r(root: &Value, prefix: &str) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let subcommands = root.get("subcommands");
    if subcommands.is_none() {
        return map;
    }
    let subcommands = subcommands.unwrap();
    if let Value::Sequence(cmds) = subcommands {
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
fn get_map(cmd: &Value) -> (&str, &Value) {
    let name = get_name(cmd).unwrap();
    (name, cmd.get(name).unwrap())
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
