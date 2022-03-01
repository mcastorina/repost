use nom::{
    branch::{alt, permutation},
    bytes::complete::{escaped, escaped_transform, tag, take_till, take_till1, take_until},
    character::complete::{
        alpha1, alphanumeric1, digit0, digit1, line_ending, none_of, not_line_ending, one_of,
        space0, space1,
    },
    character::is_space,
    combinator::{all_consuming, eof, map, map_res, not, opt, peek, recognize, value, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Eq)]
enum Cmd {
    CreateRequest(CreateRequest),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct CreateRequest {
    name: String,
    url: String,
    method: Option<String>,
    headers: Vec<String>,
    // TODO: blob body
    body: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum CmdKind {
    PrintRequests,
    PrintVariables,
    PrintEnvironments,
    PrintWorkspaces,

    CreateRequest,
    CreateVariable,
}

fn eol(input: &str) -> IResult<&str, &str> {
    alt((terminated(tag("--"), eow), eof))(input)
}

fn eow(input: &str) -> IResult<&str, &str> {
    alt((space1, eof))(input)
}

fn print(input: &str) -> IResult<&str, &str> {
    alt((tag("print"), tag("get"), tag("show"), tag("p")))(input)
}

fn create(input: &str) -> IResult<&str, &str> {
    alt((tag("create"), tag("new"), tag("add"), tag("c")))(input)
}

fn requests(input: &str) -> IResult<&str, &str> {
    alt((tag("requests"), tag("reqs"), request))(input)
}

fn request(input: &str) -> IResult<&str, &str> {
    alt((tag("request"), tag("req"), tag("r")))(input)
}

fn variables(input: &str) -> IResult<&str, &str> {
    alt((tag("variables"), tag("vars"), variable))(input)
}

fn variable(input: &str) -> IResult<&str, &str> {
    alt((tag("variable"), tag("var"), tag("v")))(input)
}

fn environments(input: &str) -> IResult<&str, &str> {
    alt((tag("environments"), tag("envs"), environment))(input)
}

fn environment(input: &str) -> IResult<&str, &str> {
    alt((tag("environment"), tag("env"), tag("e")))(input)
}

fn workspaces(input: &str) -> IResult<&str, &str> {
    alt((tag("workspaces"), workspace))(input)
}

fn workspace(input: &str) -> IResult<&str, &str> {
    alt((tag("workspace"), tag("ws"), tag("w")))(input)
}

fn cmd_kind(input: &str) -> IResult<&str, CmdKind> {
    alt((
        map(tuple((print, space1, requests)), |_| CmdKind::PrintRequests),
        map(tuple((print, space1, variables)), |_| {
            CmdKind::PrintVariables
        }),
        map(tuple((print, space1, environments)), |_| {
            CmdKind::PrintEnvironments
        }),
        map(tuple((print, space1, workspaces)), |_| {
            CmdKind::PrintWorkspaces
        }),
        map(tuple((create, space1, request)), |_| CmdKind::CreateRequest),
        map(tuple((create, space1, variable)), |_| {
            CmdKind::CreateVariable
        }),
    ))(input)
}

fn any_string(input: &str) -> IResult<&str, &str> {
    let esc_single = escaped(none_of("\\\'"), '\\', tag("'"));
    let esc_double = escaped(none_of("\\\""), '\\', tag("\""));
    let esc_space = escaped(none_of("\\ \t"), '\\', one_of(" \t"));
    terminated(
        alt((
            delimited(tag("'"), alt((esc_single, tag(""))), tag("'")),
            delimited(tag("\""), alt((esc_double, tag(""))), tag("\"")),
            esc_space,
        )),
        eow,
    )(input)
}

fn string(input: &str) -> IResult<&str, &str> {
    verify(any_string, |s: &str| !s.starts_with('-'))(input)
}

fn method_flag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((alt((tag("--method"), tag("-m"))), space1)), string)(input)
}

fn header_flag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((alt((tag("--header"), tag("-H"))), space1)), string)(input)
}

fn create_request(input: &str) -> IResult<&str, CreateRequest> {
    // consume "create request " prefix
    let (mut rest, _) = tuple((create, space1, request, space1))(input)?;

    // continuously try to parse flags and arguments until we either
    // run out of data or we can't parse anything
    let mut method = None;
    let mut headers = Vec::new();
    let mut name = None;
    let mut url = None;
    // TODO:
    //  * parse body
    //  * parse `--` argument separator
    loop {
        if let Ok((r, m)) = method_flag(rest) {
            rest = r;
            // TODO: check if method is already Some
            method = Some(m.to_string());
            continue;
        }

        if let Ok((r, h)) = header_flag(rest) {
            rest = r;
            headers.push(h.to_string());
            continue;
        }

        // at this point, whatever is left is not a method or header
        // TODO: see if it's a flag or not
        match (&name, &url) {
            (Some(_), Some(_)) => (),
            (None, _) => {
                let (r, n) = string(rest)?;
                rest = r;
                name = Some(n.to_string());
                continue;
            }
            (Some(_), None) => {
                let (r, u) = string(rest)?;
                rest = r;
                url = Some(u.to_string());
                continue;
            }
        }

        break;
    }
    Ok((
        rest,
        CreateRequest {
            name: name.unwrap(),
            url: url.unwrap(),
            method,
            headers,
            body: None,
        },
    ))
}

use std::collections::HashMap;

// Parse as much of the input as we can, given the legend.
fn get_opts<'a>(
    legend: &'static str,
    input: &'a str,
) -> IResult<&'a str, (HashMap<String, Option<String>>, Vec<String>)> {
    let expects_value: HashMap<_, _> = legend
        .split_ascii_whitespace()
        .map(|entry| {
            let mut name = entry.to_string();
            if name.chars().last() == Some(':') {
                name.pop();
                (name, true)
            } else {
                (name, false)
            }
        })
        .collect();

    let mut rest = input;
    let mut break_seen = false;
    let mut args = Vec::new();
    let mut opts = HashMap::new();
    loop {
        let s = any_string(rest);
        if s.is_err() {
            break;
        }
        let word;
        (rest, word) = s.unwrap();

        if break_seen {
            args.push(word.to_string());
            continue;
        }
        match kind(word) {
            OptKind::Break => break_seen = true,
            OptKind::None => args.push(word.to_string()),
            OptKind::Long(s) | OptKind::Short(s) => {
                if !expects_value.contains_key(&s) {
                    // unexpected option: return an error or break?
                    break;
                }
                // we can unwrap here because we check for contains_key first
                if *expects_value.get(&s).unwrap() {
                    if let Ok((r, value)) = string(rest) {
                        rest = r;
                        opts.insert(s, Some(value.to_string()));
                    } else {
                        // flag expects value but no value given
                        break;
                    }
                } else {
                    opts.insert(s, None);
                }
            }
            OptKind::MultiShort(s) => {
                todo!()
            }
        }
    }
    Ok((rest, (opts, args)))
}

fn long_opt(input: &str) -> IResult<&str, &str> {
    preceded(tag("--"), string)(input)
}

fn short_opt(input: &str) -> IResult<&str, &str> {
    preceded(tag("-"), verify(string, |s: &str| s.len() == 1))(input)
}

fn multi_short_opt(input: &str) -> IResult<&str, &str> {
    preceded(tag("-"), string)(input)
}

fn break_opt(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("--"), eow)))(input)
}

enum OptKind {
    Long(String),
    Short(String),
    MultiShort(String),
    Break,
    None,
}

fn kind(s: &str) -> OptKind {
    let mut iter = s.chars();
    match (iter.next(), iter.next()) {
        (Some('-'), Some('-')) => {
            let s: String = iter.collect();
            if s.len() == 0 {
                OptKind::Break
            } else {
                OptKind::Long(s)
            }
        }
        (Some('-'), Some(c)) => {
            let mut s: String = iter.collect();
            if s.len() == 0 {
                OptKind::Short(c.to_string())
            } else {
                // prefix the already consumed char
                s.insert(0, c);
                OptKind::MultiShort(s)
            }
        }
        (Some(_), _) => OptKind::None,
        _ => unreachable!("kind should always be given a non-empty string"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmd_kind() {
        for print in &["print", "get", "show", "p"] {
            for req in &["requests", "request", "reqs", "req", "r"] {
                assert_eq!(
                    cmd_kind(format!("{print} {req}").as_str()),
                    Ok(("", CmdKind::PrintRequests))
                );
            }

            for var in &["variables", "variable", "vars", "var", "v"] {
                assert_eq!(
                    cmd_kind(format!("{print} {var}").as_str()),
                    Ok(("", CmdKind::PrintVariables))
                );
            }

            for env in &["environments", "environment", "envs", "env", "e"] {
                assert_eq!(
                    cmd_kind(format!("{print} {env}").as_str()),
                    Ok(("", CmdKind::PrintEnvironments))
                );
            }

            for ws in &["workspaces", "workspace", "ws", "w"] {
                assert_eq!(
                    cmd_kind(format!("{print} {ws}").as_str()),
                    Ok(("", CmdKind::PrintWorkspaces))
                );
            }
        }

        for create in &["create", "new", "add", "c"] {
            for req in &["request", "req", "r"] {
                assert_eq!(
                    cmd_kind(format!("{create} {req}").as_str()),
                    Ok(("", CmdKind::CreateRequest))
                );
            }

            for var in &["variable", "var", "v"] {
                assert_eq!(
                    cmd_kind(format!("{create} {var}").as_str()),
                    Ok(("", CmdKind::CreateVariable))
                );
            }
        }
    }

    #[test]
    fn test_print() {
        assert_eq!(print("print"), Ok(("", "print")));
        assert_eq!(print("get"), Ok(("", "get")));
        assert_eq!(print("show"), Ok(("", "show")));
        assert_eq!(print("p"), Ok(("", "p")));
    }

    #[test]
    fn test_flag() {
        assert_eq!(method_flag("-m foo"), Ok(("", "foo")));
    }

    #[test]
    fn test_create_request() {
        let without_method = CreateRequest {
            name: "foo".to_string(),
            url: "bar".to_string(),
            method: None,
            headers: Vec::new(),
            body: None,
        };
        assert_eq!(
            create_request("create req foo bar"),
            Ok(("", without_method.clone()))
        );
        assert_eq!(
            create_request("create req foo bar -m"),
            Ok(("-m", without_method.clone()))
        );

        let with_method = CreateRequest {
            name: "foo".to_string(),
            url: "bar".to_string(),
            method: Some("yay".to_string()),
            headers: Vec::new(),
            body: None,
        };
        assert_eq!(
            create_request("create req foo bar -m yay"),
            Ok(("", with_method.clone()))
        );
        assert_eq!(
            create_request("create req foo -m yay bar"),
            Ok(("", with_method.clone()))
        );
        assert_eq!(
            create_request("create req -m yay foo bar"),
            Ok(("", with_method.clone()))
        );

        let with_headers = CreateRequest {
            name: "foo".to_string(),
            url: "bar".to_string(),
            method: None,
            headers: vec!["h1".to_string(), "h2".to_string()],
            body: None,
        };
        assert_eq!(
            create_request("c r -H h1 foo bar -H h2"),
            Ok(("", with_headers.clone()))
        );
        assert_eq!(
            create_request("c r foo bar -H h1 -H h2"),
            Ok(("", with_headers.clone()))
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(string("foo"), Ok(("", "foo")));
        assert_eq!(string("foo\\ bar"), Ok(("", "foo\\ bar")));
        assert_eq!(string("foo\\  bar"), Ok(("bar", "foo\\ ")));
        assert_eq!(string("'foo bar'"), Ok(("", "foo bar")));
        assert_eq!(string(r#"'fo\'o'"#), Ok(("", r#"fo\'o"#)));
        assert_eq!(string(r#""fo'o""#), Ok(("", r#"fo'o"#)));
        assert_eq!(string(r#"'fo"o'"#), Ok(("", r#"fo"o"#)));
        assert_eq!(string(r#""fo\"o""#), Ok(("", r#"fo\"o"#)));
        assert_eq!(string(r#"''"#), Ok(("", "")));
        assert_eq!(string("foo "), Ok(("", "foo")));
        assert!(string(" foo ").is_err());
    }
}
