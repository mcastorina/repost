mod error;

use error::{IResult, ParseError, ParseState};
use nom::Err::Failure;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{escaped, escaped_transform, tag, take, take_till, take_till1, take_until},
    character::complete::{
        alpha1, alphanumeric1, digit0, digit1, line_ending, none_of, not_line_ending, one_of,
        space0, space1,
    },
    character::is_space,
    combinator::{all_consuming, cut, eof, map, map_res, not, opt, peek, recognize, value, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
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

fn eol(input: &str) -> IResult<&str> {
    alt((terminated(tag("--"), eow), eof))(input)
}

fn eow(input: &str) -> IResult<&str> {
    alt((space1, eof))(input)
}

fn print(input: &str) -> IResult<&str> {
    alt((tag("print"), tag("get"), tag("show"), tag("p")))(input)
}

fn create(input: &str) -> IResult<&str> {
    alt((tag("create"), tag("new"), tag("add"), tag("c")))(input)
}

fn requests(input: &str) -> IResult<&str> {
    alt((tag("requests"), tag("reqs"), request))(input)
}

fn request(input: &str) -> IResult<&str> {
    alt((tag("request"), tag("req"), tag("r")))(input)
}

fn variables(input: &str) -> IResult<&str> {
    alt((tag("variables"), tag("vars"), variable))(input)
}

fn variable(input: &str) -> IResult<&str> {
    alt((tag("variable"), tag("var"), tag("v")))(input)
}

fn environments(input: &str) -> IResult<&str> {
    alt((tag("environments"), tag("envs"), environment))(input)
}

fn environment(input: &str) -> IResult<&str> {
    alt((tag("environment"), tag("env"), tag("e")))(input)
}

fn workspaces(input: &str) -> IResult<&str> {
    alt((tag("workspaces"), workspace))(input)
}

fn workspace(input: &str) -> IResult<&str> {
    alt((tag("workspace"), tag("ws"), tag("w")))(input)
}

fn cmd_kind(input: &str) -> IResult<CmdKind> {
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

fn any_string(input: &str) -> IResult<&str> {
    // return an error if the input is empty
    take(1_usize)(input)?;

    let esc_single = escaped(none_of("\\'"), '\\', tag("'"));
    let esc_double = escaped(none_of("\\\""), '\\', tag("\""));
    let esc_space = escaped(none_of("\\ \t'\""), '\\', one_of(" \t'\""));
    terminated(
        alt((
            delimited(tag("'"), alt((esc_single, tag(""))), tag("'")),
            delimited(tag("\""), alt((esc_double, tag(""))), tag("\"")),
            esc_space,
        )),
        eow,
    )(input)
}

fn string(input: &str) -> IResult<&str> {
    verify(any_string, |s: &str| !s.starts_with('-'))(input)
}

fn method_flag(input: &str) -> IResult<&str> {
    preceded(tuple((alt((tag("--method"), tag("-m"))), space1)), string)(input)
}

fn header_flag(input: &str) -> IResult<&str> {
    preceded(tuple((alt((tag("--header"), tag("-H"))), space1)), string)(input)
}

fn create_request(input: &str) -> IResult<CreateRequest> {
    // consume "create request " prefix
    let (rest, _) = tuple((create, space1, request, space1))(input)?;
    let (rest, (mut opts, args)) = get_opts(rest, "method,m: header,H:")?;

    if args.len() != 2 {
        todo!("handle extra / not enough args")
    }
    let (name, url) = (args[0], args[1]);
    let method = opts.remove("method").unwrap_or_default();
    if method.len() > 1 {
        todo!("handle repeated flags")
    }
    let method = method.get(0);

    let headers = opts.remove("header").unwrap_or_default();
    let headers = headers.into_iter().map(|v| v.to_owned()).collect();
    Ok((
        rest,
        CreateRequest {
            name: name.to_string(),
            url: url.to_string(),
            method: method.map(|m| m.to_string()),
            headers,
            body: None,
        },
    ))
}

use std::collections::HashMap;

// Parse the rest of the input as options and arguments.
fn get_opts<'a>(
    input: &'a str,
    legend: &'static str,
) -> IResult<'a, (HashMap<&'a str, Vec<&'a str>>, Vec<&'a str>)> {
    // TODO: use slice of pairs
    let expects_value: HashMap<_, _> = legend
        .split_ascii_whitespace()
        .flat_map(|entry| {
            let mut name = entry.to_string();
            if entry.chars().last() == Some(':') {
                name.pop();
                let key = entry[..entry.len() - 1]
                    .split(',')
                    .next()
                    .expect("an identifier");
                entry[..entry.len() - 1]
                    .split(',')
                    .map(|alias| (alias, (key, true)))
                    .collect::<Vec<_>>()
            } else {
                let key = entry.split(',').next().expect("an identifier");
                entry
                    .split(',')
                    .map(|alias| (alias, (key, false)))
                    .collect::<Vec<_>>()
            }
        })
        .collect();

    let mut rest = input;
    let mut break_seen = false;
    let mut args = Vec::new();
    let mut opts = HashMap::new();
    loop {
        if break_seen {
            if let Ok((r, mut v)) = many0(any_string)(rest) {
                rest = r;
                args.append(&mut v);
                break;
            } else {
                unreachable!();
            }
        }
        if let Ok((r, s)) = alt((long_opt, short_opt))(rest) {
            rest = r;
            if !expects_value.contains_key(s) {
                todo!("flag is not part of legend: {}", s);
            }
            let (key, expects_value) = expects_value.get(s).unwrap();
            if *expects_value {
                let (r, v) = cut(string)(rest).map_err(|_| {
                    Failure(ParseError {
                        // TODO: remove clone
                        state: Some(ParseState {
                            last_option: Some(key),
                            options: opts.clone(),
                            args: args.clone(),
                            next: any_string(rest).unwrap_or_default().1,
                            rest: rest,
                        }),
                        message: None,
                    })
                })?;
                rest = r;
                let vec = opts.entry(*key).or_insert(Vec::new());
                vec.push(v);
            } else {
                opts.entry(*key).or_insert(Vec::new());
            }
            continue;
        }

        if let Ok((r, s)) = multi_short_opt(rest) {
            rest = r;
            todo!("unimplemented");
        }

        if let Ok((r, _)) = break_opt(rest) {
            rest = r;
            break_seen = true;
            continue;
        }

        if let Ok((r, arg)) = string(rest) {
            rest = r;
            args.push(arg);
            continue;
        }

        break;
    }
    Ok((rest, (opts, args)))
}

fn long_opt(input: &str) -> IResult<&str> {
    preceded(tag("--"), string)(input)
}

fn short_opt(input: &str) -> IResult<&str> {
    preceded(tag("-"), verify(string, |s: &str| s.len() == 1))(input)
}

fn multi_short_opt(input: &str) -> IResult<&str> {
    preceded(tag("-"), string)(input)
}

fn break_opt(input: &str) -> IResult<()> {
    value((), tuple((tag("--"), eow)))(input)
}

#[cfg(test)]
mod test {
    use super::error::ParseState;
    use super::*;
    use maplit::hashmap;
    use nom::Err::Failure;

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
    fn test_create_request_fail() {
        let result = create_request("create req foo bar -m");
        assert!(result.is_err());

        match result.unwrap_err() {
            Failure(err) => {
                assert_eq!(
                    err.state,
                    Some(ParseState {
                        last_option: Some("method"),
                        options: HashMap::new(),
                        args: vec!["foo", "bar"],
                        next: "",
                        rest: "",
                    })
                )
            }
            _ => panic!("expected failure error"),
        }

        let result = create_request("create req foo bar --method --foo bar");
        assert!(result.is_err());

        match result.unwrap_err() {
            Failure(err) => {
                assert_eq!(
                    err.state,
                    Some(ParseState {
                        last_option: Some("method"),
                        options: HashMap::new(),
                        args: vec!["foo", "bar"],
                        next: "--foo",
                        rest: "--foo bar",
                    })
                )
            }
            _ => panic!("expected failure error"),
        }

        let result = create_request("create req foo bar --header 'Cont");
        assert!(result.is_err());

        match result.unwrap_err() {
            Failure(err) => {
                assert_eq!(
                    err.state,
                    Some(ParseState {
                        last_option: Some("header"),
                        options: HashMap::new(),
                        args: vec!["foo", "bar"],
                        next: "",
                        rest: "'Cont",
                    })
                )
            }
            _ => panic!("expected failure error"),
        }
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

        assert!(string("'foo").is_err());
        assert!(string("fo'o").is_err());
        assert!(string("foo'").is_err());
        assert_eq!(string("foo\\'"), Ok(("", r#"foo\'"#)));
    }

    #[test]
    fn test_get_opts() {
        assert_eq!(
            get_opts("--foo --bar baz", "foo bar"),
            Ok((
                "",
                (hashmap! {"foo" => vec![], "bar" => vec![]}, vec!["baz"])
            ))
        );

        assert_eq!(
            get_opts("--foo --bar baz", "foo bar:"),
            Ok((
                "",
                (hashmap! {"foo" => vec![], "bar" => vec!["baz"]}, vec![])
            ))
        );

        assert_eq!(
            get_opts("--foo -- --bar baz", "foo bar:"),
            Ok(("", (hashmap! {"foo" => vec![]}, vec!["--bar", "baz"])))
        );

        assert_eq!(
            get_opts("-H foo one -H bar -H baz -- two", "header,H:"),
            Ok((
                "",
                (
                    hashmap! {"header" => vec!["foo", "bar", "baz"]},
                    vec!["one", "two"],
                )
            ))
        );
    }

    #[test]
    fn test_any_string() {
        assert!(any_string("").is_err());
        assert_eq!(any_string("1"), Ok(("", "1")));
        assert_eq!(any_string("foo"), Ok(("", "foo")));
        assert_eq!(any_string("foo bar"), Ok(("bar", "foo")));
        assert_eq!(any_string("'foo bar'"), Ok(("", "foo bar")));
        assert_eq!(any_string("foo\\ bar"), Ok(("", "foo\\ bar")));
        // TODO: test unclosed quotes
    }
}
