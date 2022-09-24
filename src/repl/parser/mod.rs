mod create_request;
mod create_variable;
mod error;
mod print_requests;
mod print_variables;

use create_request::{CreateRequest, CreateRequestBuilder};
use create_variable::{CreateVariable, CreateVariableBuilder};
use error::{IResult, ParseError, ParseErrorKind};
use nom::Err::{Error, Failure};
use nom::{
    branch::{alt, permutation},
    bytes::complete::{escaped, escaped_transform, tag, take, take_till, take_till1, take_until},
    character::complete::{
        alpha1, alphanumeric1, digit0, digit1, line_ending, none_of, not_line_ending, one_of,
        space0, space1,
    },
    character::is_space,
    combinator::{
        all_consuming, complete, cut, eof, fail, map, map_res, not, opt, peek, recognize, value,
        verify,
    },
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Parser,
};
use print_requests::{PrintRequests, PrintRequestsBuilder};
use print_variables::{PrintVariables, PrintVariablesBuilder};

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    CreateRequest(CreateRequest),
    CreateVariable(CreateVariable),
    PrintRequest(PrintRequests),
    PrintVariable(PrintVariables),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Builder {
    CreateRequestBuilder(CreateRequestBuilder),
    CreateVariableBuilder(CreateVariableBuilder),
    PrintRequestsBuilder(PrintRequestsBuilder),
    PrintVariablesBuilder(PrintVariablesBuilder),
}

impl Builder {
    pub fn opts(&self) -> &'static [OptKey] {
        match self {
            Self::CreateRequestBuilder(_) => CreateRequestBuilder::OPTS,
            Self::CreateVariableBuilder(_) => CreateVariableBuilder::OPTS,
            Self::PrintRequestsBuilder(_) => PrintRequestsBuilder::OPTS,
            Self::PrintVariablesBuilder(_) => PrintVariablesBuilder::OPTS,
        }
    }
}

pub fn parse_command(input: &str) -> Result<Command, ()> {
    let (rest, kind) = parse_command_kind(input, false).map_err(|_| ())?;
    Ok(match kind {
        CommandKind::CreateRequest => {
            Command::CreateRequest(parse_subcommand::<CreateRequestBuilder>(rest)?.try_into()?)
        }
        CommandKind::CreateVariable => {
            Command::CreateVariable(parse_subcommand::<CreateVariableBuilder>(rest)?.try_into()?)
        }
        CommandKind::PrintRequests => {
            Command::PrintRequest(parse_subcommand::<PrintRequestsBuilder>(rest)?.try_into()?)
        }
        CommandKind::PrintVariables => {
            Command::PrintVariable(parse_subcommand::<PrintVariablesBuilder>(rest)?.try_into()?)
        }
    })
}

pub fn parse_completion(input: &str) -> Result<(Option<Builder>, Option<(&str, Completion)>), ()> {
    let (rest, kind) = match parse_command_kind(input, true) {
        Ok(ok) => ok,
        Err(err) => return Ok((None, err)),
    };
    Ok(match kind {
        CommandKind::CreateRequest => {
            let (s, (builder, completion)) = parse_subcommand_completion(rest).map_err(|_| ())?;
            (
                Some(Builder::CreateRequestBuilder(builder)),
                completion.map(|c| (s, c)),
            )
        }
        CommandKind::CreateVariable => {
            let (s, (builder, completion)) = parse_subcommand_completion(rest).map_err(|_| ())?;
            (
                Some(Builder::CreateVariableBuilder(builder)),
                completion.map(|c| (s, c)),
            )
        }
        CommandKind::PrintRequests => {
            let (s, (builder, completion)) = parse_subcommand_completion(rest).map_err(|_| ())?;
            (
                Some(Builder::PrintRequestsBuilder(builder)),
                completion.map(|c| (s, c)),
            )
        }
        CommandKind::PrintVariables => {
            let (s, (builder, completion)) = parse_subcommand_completion(rest).map_err(|_| ())?;
            (
                Some(Builder::PrintVariablesBuilder(builder)),
                completion.map(|c| (s, c)),
            )
        }
    })
}

// Create: { Request, Variable }
// Print: { Requests, Variables, Environments }

#[derive(Debug, PartialEq, Clone)]
enum CommandKind {
    CreateRequest,
    CreateVariable,
    PrintRequests,
    PrintVariables,
}

impl CommandKind {
    const KINDS: &'static [Self] = &[
        Self::CreateRequest,
        Self::CreateVariable,
        Self::PrintRequests,
        Self::PrintVariables,
    ];
    fn keys(&self) -> &'static [CommandKey] {
        use CommandKey::*;
        match self {
            Self::CreateRequest => &[Create, Request],
            Self::CreateVariable => &[Create, Variable],
            Self::PrintRequests => &[Print, Requests],
            Self::PrintVariables => &[Print, Variables],
        }
    }
    fn parse(input: &str) -> Result<(&str, Self), ()> {
        'main: for kind in Self::KINDS.into_iter() {
            let mut input = input;
            for key in kind.keys() {
                input = strip_leading_space(input);
                input = match tuple((|i| key.parse(i), eow))(input) {
                    Ok((input, _)) => input,
                    _ => continue 'main,
                }
            }
            return Ok((input, kind.to_owned()));
        }
        Err(())
    }
}

// TODO: HashMap of CommandKey => &[CommandKey]
const CMDS: &'static [CommandKey] = &[CommandKey::Create, CommandKey::Print];

fn parse_command_kind(
    input: &str,
    completion: bool,
) -> Result<(&str, CommandKind), Option<(&str, Completion)>> {
    let input = strip_leading_space(input);
    // Happy case.
    if let Ok((rest, cmd)) = CommandKind::parse(input) {
        if !(completion && rest.len() == 0) {
            return Ok((strip_leading_space(rest), cmd));
        }
    }

    // Error case. Figure out what the completion should be.
    if input.len() == 0 {
        return Err(Some((input, Completion::Command(CMDS))));
    }

    let (rest, cmd) = match CMDS.iter().filter_map(|cmd| cmd.parse(input).ok()).next() {
        Some((rest, cmd)) => (rest, cmd),
        None => {
            return Err(match terminated(word, space1)(input) {
                Ok(_) => None,
                _ => Some((input, Completion::Command(CMDS))),
            })
        }
    };

    let (rest, _) =
        space1(rest).map_err(|_: nom::Err<ParseError<_>>| (input, Completion::Command(CMDS)))?;

    let sub_cmds: &'static [CommandKey] = match cmd {
        CommandKey::Create => &[CommandKey::Request, CommandKey::Variable],
        CommandKey::Print => &[CommandKey::Requests, CommandKey::Variables],
        _ => unreachable!(),
    };

    Err(match terminated(word, space1)(rest) {
        Ok(_) => None,
        _ => Some((rest, Completion::Command(sub_cmds))),
    })
}

fn all<'a, O, F>(
    mut parser: F,
    kind: ParseErrorKind,
) -> impl FnMut(&'a str) -> nom::IResult<&'a str, O, ParseError<&'a str>>
where
    F: nom::Parser<&'a str, O, ParseError<&'a str>>,
{
    move |input: &str| match parser.parse(input.clone()) {
        Err(_) => match word(input) {
            Ok((_, word)) => Err(Error(ParseError {
                kind: kind.clone(),
                word: word,
            })),
            _ => Err(Error(ParseError {
                kind: kind.clone(),
                word: input,
            })),
        },
        rest => rest,
    }
}

fn word(input: &str) -> IResult<&str> {
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

fn eol(input: &str) -> IResult<&str> {
    alt((terminated(tag("--"), eow), eof))(input)
}

fn eow(input: &str) -> IResult<&str> {
    peek(alt((space1, eof)))(input)
}

fn eoo(input: &str) -> IResult<&str> {
    peek(alt((space1, tag("="), eof)))(input)
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandKey {
    Create,
    Print,
    Request,
    Requests,
    Variable,
    Variables,
}

impl CommandKey {
    pub fn completions<'a>(&'a self) -> &'static [&'static str] {
        match self {
            CommandKey::Create => &["create", "new", "add", "c"],
            CommandKey::Print => &["print", "get", "show", "p"],
            CommandKey::Request => &["request", "req", "r"],
            CommandKey::Requests => &["requests", "request", "reqs", "req", "r"],
            CommandKey::Variable => &["variable", "var", "v"],
            CommandKey::Variables => &["variables", "variable", "vars", "var", "v"],
        }
    }
    fn parse<'a>(&'a self, input: &'a str) -> IResult<Self> {
        map(
            |i| parse_literal(i, self.completions()),
            |_| self.to_owned(),
        )(input)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OptKey {
    Header,
    Method,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgKey {
    Unknown,
    Name,
    URL,
}

impl OptKey {
    pub fn completions<'a>(&'a self) -> &'static [&'static str] {
        match &self {
            OptKey::Header => &["--header", "-H"],
            OptKey::Method => &["--method", "-m"],
        }
    }
    fn parse<'a>(&'a self, input: &'a str) -> IResult<&str> {
        for variant in self.completions() {
            if let Ok((rest, _)) = terminated(tag(*variant), eoo)(input) {
                let (rest, sep) = cut(alt((space1, tag("="))))(rest)?;
                return match sep {
                    "=" => cut(word)(rest),
                    _ => cut(verify(word, |s: &str| !s.starts_with('-')))(rest),
                };
            }
        }
        Err(nom::Err::Error(ParseError::default()))
    }
}

trait CmdLineBuilder: Default {
    const ARGS: &'static [ArgKey];
    const OPTS: &'static [OptKey];
    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()> {
        Err(())
    }
    fn add_opt<S: Into<String>>(&mut self, key: OptKey, arg: S) -> Result<(), ()> {
        Err(())
    }
    fn get_completion(&self, kind: Completion) -> Option<Completion> {
        Some(kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Completion {
    Arg(ArgKey),
    OptKey,
    OptValue(OptKey),
    Command(&'static [CommandKey]),
}

fn strip_leading_space(s: &str) -> &str {
    space0::<_, ParseError<&str>>(s)
        .expect("stripping leading whitespace")
        .0
}

fn parse_literal<'a>(input: &'a str, aliases: &'static [&'static str]) -> IResult<'a, &'a str> {
    for alias in aliases.iter().cloned() {
        let result = terminated(tag(alias), eow)(input);
        if result.is_ok() {
            return result;
        }
    }
    Err(nom::Err::Error(ParseError::default()))
}

fn parse_subcommand<B>(input: &str) -> Result<B, ()>
where
    B: CmdLineBuilder,
{
    let parser = |i| _parse_subcommand(i, false);
    let (_, builder): (_, B) = map(parser, |(b, _)| b)(input).map_err(|_| ())?;
    Ok(builder)
}

fn parse_subcommand_completion<B>(input: &str) -> IResult<(B, Option<Completion>)>
where
    B: CmdLineBuilder,
{
    _parse_subcommand(input, true)
}

fn _parse_subcommand<B>(mut input: &str, completion: bool) -> IResult<(B, Option<Completion>)>
where
    B: CmdLineBuilder,
{
    let mut builder = B::default();
    let mut double_dash = false;
    let mut arg: &str;
    let err = |_| nom::Err::Error(ParseError::default());
    let done_parsing = if completion {
        |s: &str| terminated(word, space1)(s).is_err()
    } else {
        |s: &str| s.len() == 0
    };
    let mut arg_count = 0;
    let mut arg_key = || match B::ARGS.get(arg_count) {
        Some(key) => {
            arg_count += 1;
            key.clone()
        }
        _ => ArgKey::Unknown,
    };
    'main: loop {
        input = strip_leading_space(input);
        if done_parsing(input) {
            break;
        }
        // We encountered a '--' so everything should be interpreted as an argument.
        if double_dash {
            (input, arg) = word(input)?;
            builder.add_arg(arg_key(), arg).map_err(err)?;
            continue;
        }
        // Check for '--'.
        if let Ok(ret) = eol(input) {
            (input, _) = ret;
            double_dash = true;
            continue;
        }
        // Try to parse the argument as long as it doesn't start with '-'.
        if let Ok(ret) = verify(word, |s: &str| !s.starts_with('-'))(input) {
            (input, arg) = ret;
            builder.add_arg(arg_key(), arg).map_err(err)?;
            continue;
        }
        // Try to parse any options.
        for opt in B::OPTS {
            match opt.parse(input) {
                // Successfully parsed the option.
                Ok(ret) => {
                    (input, arg) = ret;
                    let opt = opt.to_owned();
                    if completion && input.len() == 0 {
                        let completion = builder.get_completion(Completion::OptValue(opt));
                        return Ok((arg, (builder, completion)));
                    }
                    builder.add_opt(opt, arg).map_err(err)?;
                    continue 'main;
                }
                // Non-recoverable error (e.g. the key parsed but not the value).
                Err(ret @ Failure(_)) if !completion => return Err(ret),
                Err(Failure(err)) if completion => {
                    let completion = builder.get_completion(Completion::OptValue(opt.to_owned()));
                    return Ok((err.word, (builder, completion)));
                }
                // Recoverable error, do nothing and try the next parser.
                _ => (),
            }
        }
        // Nothing successfully parsed the input, return an error.
        return Err(nom::Err::Failure(ParseError {
            kind: ParseErrorKind::Unknown,
            word: input,
        }));
    }
    if !completion {
        return Ok((input, (builder, None)));
    }
    let completion = match (double_dash, input.starts_with('-')) {
        (true, _) => builder.get_completion(Completion::Arg(arg_key())),
        (_, true) => builder.get_completion(Completion::OptKey),
        (_, false) => builder.get_completion(Completion::Arg(arg_key())),
    };
    Ok((input, (builder, completion)))
}

#[cfg(test)]
mod test {
    use super::create_request::*;
    use super::*;
    use super::{ParseError, ParseErrorKind};
    use maplit::hashmap;
    use nom::Err::{Error, Failure};

    #[test]
    fn test_print() {
        let print = |i| parse_literal(i, CommandKey::Print.completions());
        assert_eq!(print("print"), Ok(("", "print")));
        assert_eq!(print("get"), Ok(("", "get")));
        assert_eq!(print("show"), Ok(("", "show")));
        assert_eq!(print("p"), Ok(("", "p")));
        assert!(matches!(print("gets"), Err(_)));
    }

    #[test]
    fn test_header() {
        struct test {
            input: &'static str,
            expected: &'static str,
        }
        let tests = &[
            ("-H foo", "foo"),
            ("-H foo", "foo"),
            ("--header foo", "foo"),
            ("-H=foo", "foo"),
            ("--header=foo", "foo"),
            ("--header=--foo", "--foo"),
            ("-H=--foo", "--foo"),
        ];
        let opt_header = |input: &'static str| OptKey::Header.parse(input);
        for (input, expected) in tests {
            assert_eq!(opt_header(input), Ok(("", *expected)));
        }
        assert!(matches!(opt_header("-H"), Err(_)));
        assert!(matches!(opt_header("--header"), Err(_)));
        assert!(matches!(opt_header("--header "), Err(_)));
        assert!(matches!(opt_header("--header="), Err(_)));
        assert!(matches!(opt_header("--header -H"), Err(_)));
        assert!(matches!(opt_header("--header --foo"), Err(_)));
        assert!(matches!(opt_header("--oops"), Err(_)));
    }

    #[test]
    fn test_create_request() {
        let create_request =
            |input| parse_subcommand::<CreateRequestBuilder>(input).and_then(|b| b.try_into());
        assert_eq!(
            create_request("foo bar"),
            Ok(CreateRequest {
                name: "foo".to_string(),
                url: "bar".to_string(),
                method: None,
                headers: vec![],
                body: None,
            })
        );
        assert_eq!(
            create_request("foo -H yay bar"),
            Ok(CreateRequest {
                name: "foo".to_string(),
                url: "bar".to_string(),
                method: None,
                headers: vec!["yay".to_string()],
                body: None,
            })
        );
        assert_eq!(
            create_request("--method baz -- foo bar"),
            Ok(CreateRequest {
                name: "foo".to_string(),
                url: "bar".to_string(),
                method: Some("baz".to_string()),
                headers: vec![],
                body: None,
            })
        );
        assert_eq!(
            create_request("-- --foo --bar"),
            Ok(CreateRequest {
                name: "--foo".to_string(),
                url: "--bar".to_string(),
                method: None,
                headers: vec![],
                body: None,
            })
        );
        assert!(matches!(create_request("foo bar baz"), Err(_)));
        assert!(matches!(create_request("--foo --bar"), Err(_)));
        assert!(matches!(create_request("--foo"), Err(_)));
        assert!(matches!(create_request("foo bar -X"), Err(_)));
        assert!(matches!(create_request("foo"), Err(_)));
    }

    #[test]
    fn test_create_request_complete() {
        let create_request_completion =
            |input| parse_subcommand_completion::<CreateRequestBuilder>(input);
        assert_eq!(
            create_request_completion("foo"),
            Ok((
                "foo",
                (
                    CreateRequestBuilder {
                        name: None,
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::Arg(ArgKey::Name))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo bar"),
            Ok((
                "bar",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::Arg(ArgKey::URL))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo "),
            Ok((
                "",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::Arg(ArgKey::URL))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo -"),
            Ok((
                "-",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::OptKey)
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo -H f"),
            Ok((
                "f",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::OptValue(OptKey::Header))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo -H "),
            Ok((
                "",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::OptValue(OptKey::Header))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo --he"),
            Ok((
                "--he",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::OptKey)
                ),
            ))
        );
        assert_eq!(
            create_request_completion("-- --foo"),
            Ok((
                "--foo",
                (
                    CreateRequestBuilder {
                        name: None,
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::Arg(ArgKey::Name))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo bar baz"),
            Ok((
                "baz",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: Some("bar".to_string()),
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    None
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo -H bar baz"),
            Ok((
                "baz",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec!["bar".to_string()],
                        body: None,
                    },
                    Some(Completion::Arg(ArgKey::URL))
                ),
            ))
        );
        assert_eq!(
            create_request_completion("foo -H 'bar  "),
            Ok((
                "'bar  ",
                (
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    },
                    Some(Completion::OptValue(OptKey::Header))
                ),
            ))
        );
        assert!(matches!(create_request_completion("foo bar baz "), Err(_)));
    }

    #[test]
    fn test_parse_command_kind() {
        assert_eq!(
            parse_command_kind("create request foo bar baz", false),
            Ok(("foo bar baz", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("c req foo bar baz", false),
            Ok(("foo bar baz", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("  c   req   foo bar baz", false),
            Ok(("foo bar baz", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("create request ", false),
            Ok(("", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("create request", false),
            Ok(("", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("create req", false),
            Ok(("", CommandKind::CreateRequest))
        );
        assert_eq!(
            parse_command_kind("create foo", false),
            Err(Some((
                "foo",
                Completion::Command(&[CommandKey::Request, CommandKey::Variable])
            )))
        );
        assert_eq!(
            parse_command_kind("foo", false),
            Err(Some(("foo", Completion::Command(super::CMDS))))
        );
        assert_eq!(
            parse_command_kind("", false),
            Err(Some(("", Completion::Command(super::CMDS))))
        );
        assert_eq!(parse_command_kind("create foo ", false), Err(None));
        assert_eq!(parse_command_kind("foo bar", false), Err(None));
        assert_eq!(parse_command_kind("foo ", false), Err(None));
        assert_eq!(
            parse_command_kind("p v", false),
            Ok(("", CommandKind::PrintVariables))
        );
        assert_eq!(
            parse_command_kind("p v", true),
            Err(Some((
                "v",
                Completion::Command(&[CommandKey::Requests, CommandKey::Variables])
            )))
        );
    }
}
