mod create_request;
mod error;

use create_request::{CreateRequest, CreateRequestBuilder};
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

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    CreateRequest(CreateRequest),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Builder {
    CreateRequestBuilder(CreateRequestBuilder),
}

pub fn parse_command(input: &str) -> Result<Command, ()> {
    enum Kind {
        CreateRequest,
    }
    let (rest, kind) = alt((map(tuple((create, space1, request)), |_| {
        Kind::CreateRequest
    }),))(input)
    .map_err(|_| ())?;
    Ok(match kind {
        Kind::CreateRequest => Command::CreateRequest(create_request(rest)?),
    })
}

pub fn parse_completion(input: &str) -> Result<(Builder, (&str, Option<Completion>)), ()> {
    enum Kind {
        CreateRequest,
    }
    let (rest, kind) = alt((map(tuple((create, space1, request)), |_| {
        Kind::CreateRequest
    }),))(input)
    .map_err(|_| ())?;
    Ok(match kind {
        Kind::CreateRequest => {
            let (s, (builder, completion)) = create_request_completion(rest).map_err(|_| ())?;
            (Builder::CreateRequestBuilder(builder), (s, completion))
        }
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

macro_rules! literal {
    ($name:ident, $($( $lit:expr )+$(,)?)*) => {
        fn $name(input: &str) -> IResult<&str> {
        all(
            terminated(alt(($($( tag($lit), )*)*)), eow),
            ParseErrorKind::Fixed(&[ $($( $lit, )*)* ]),
        )(input)
    }};
}

macro_rules! flag {
    ($name:ident, $($( $lit:expr )+$(,)?)*) => {
        fn $name(input: &str) -> IResult<()> {
            map(terminated(alt(($($( tag($lit), )*)*)), eow), |_| ())(input)
        }
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

literal!(print, "print", "get", "show", "p");
literal!(create, "create", "new", "add", "c");
literal!(requests, "requests", "reqs");
literal!(request, "request", "req", "r");
literal!(variables, "variables", "vars");
literal!(variable, "variable", "var", "v");
literal!(environments, "environments", "envs");
literal!(environment, "environment", "env", "e");
literal!(workspaces, "workspaces");
literal!(workspace, "workspace", "ws", "w");

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
    fn completions<'a>(&'a self) -> &'static [&'static str] {
        match &self {
            OptKey::Header => &["--header", "-H"],
            OptKey::Method => &["--method", "-m"],
        }
    }
    fn parse<'a>(&'a self, input: &'a str) -> IResult<&str> {
        parse_opt(input, self.completions())
    }
}

fn parse_opt<'a>(input: &'a str, variants: &'static [&'static str]) -> IResult<'a, &'a str> {
    for variant in variants {
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

trait CmdLineBuilder {
    const ARGS: &'static [ArgKey];
    const OPT_PARSERS: &'static [OptKey];
    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()>;
    fn add_opt<S: Into<String>>(&mut self, key: OptKey, arg: S) -> Result<(), ()>;
    fn get_completion(&self, kind: Completion) -> Option<Completion>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Completion {
    Arg(ArgKey),
    OptKey,
    OptValue(OptKey),
}

fn trim_leading_space(s: &str) -> &str {
    space0::<_, ParseError<&str>>(s)
        .expect("stripping leading whitespace")
        .0
}

fn parse_subcommand<B>(mut input: &str, completion: bool) -> IResult<(B, Option<Completion>)>
where
    B: CmdLineBuilder + Default,
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
        input = trim_leading_space(input);
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
        for opt in B::OPT_PARSERS {
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
                Err(ret @ Failure(_)) => {
                    if !completion {
                        return Err(ret);
                    }
                    let completion = builder.get_completion(Completion::OptValue(opt.to_owned()));
                    return Ok(("", (builder, completion)));
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

pub fn create_request(input: &str) -> Result<CreateRequest, ()> {
    let parser = |i| parse_subcommand(i, false);
    let (_, builder): (_, CreateRequestBuilder) = map(parser, |(b, _)| b)(input).map_err(|_| ())?;
    Ok(builder.try_into()?)
}

pub fn create_request_completion(
    input: &str,
) -> IResult<(CreateRequestBuilder, Option<Completion>)> {
    parse_subcommand(input, true)
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
        assert_eq!(print("print"), Ok(("", "print")));
        assert_eq!(print("get"), Ok(("", "get")));
        assert_eq!(print("show"), Ok(("", "show")));
        assert_eq!(print("p"), Ok(("", "p")));
        assert_eq!(
            print("gets"),
            Err(Error(ParseError {
                kind: ParseErrorKind::Fixed(&["print", "get", "show", "p"]),
                word: "gets"
            }))
        );
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
        let opt_header = |input: &'static str| parse_opt(input, &["--header", "-H"]);
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
        assert!(matches!(create_request_completion("foo bar baz "), Err(_)));
    }
}
