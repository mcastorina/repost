mod create_request;
mod create_variable;
mod error;
mod print_environments;
mod print_requests;
mod print_variables;
mod print_workspaces;
mod set_environment;

use crate::error::Error;
use create_request::{CreateRequest, CreateRequestBuilder};
use create_variable::{CreateVariable, CreateVariableBuilder};
use error::{IResult, ParseError, ParseErrorKind};
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take},
    character::complete::{alphanumeric1, none_of, one_of, space0, space1},
    combinator::{cut, eof, map, peek, recognize, verify},
    sequence::{delimited, terminated, tuple},
};
use print_environments::{PrintEnvironments, PrintEnvironmentsBuilder};
use print_requests::{PrintRequests, PrintRequestsBuilder};
use print_variables::{PrintVariables, PrintVariablesBuilder};
use print_workspaces::{PrintWorkspaces, PrintWorkspacesBuilder};
use set_environment::{SetEnvironment, SetEnvironmentBuilder};

macro_rules! commands {
    ($($( ($( $word:ident )+) => ($kind:ident, $builder:ident) )+$(,)?)*) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum Command {
            $($( $kind($kind), )*)*
        }

        #[derive(Debug, PartialEq, Clone)]
        enum CommandKind {
            $($( $kind, )*)*
            Help,
        }

        impl CommandKind {
            const KINDS: &'static [Self] = &[
                $($( Self::$kind, )*)*
                Self::Help,
            ];
            fn keys(&self) -> &'static [CommandKey] {
                use CommandKey::*;
                match self {
                    $($( Self::$kind => &[$( $word, )*], )*)*
                    Self::Help => &[Help],
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

        #[derive(Debug, PartialEq, Clone)]
        pub enum Builder {
            $($( $builder($builder), )*)*
        }

        impl Builder {
            pub fn opts(&self) -> &'static [OptKey] {
                match self {
                    $($(
                        Self::$builder(_) => $builder::OPTS,
                    )*)*
                }
            }
        }
        pub fn parse_command(input: &str) -> Result<Command, Error> {
            let (rest, kind) = parse_command_kind(input, false).map_err(|_| Error::ParseError("parse_command error"))?;
            Ok(match kind {
                $($(
                    CommandKind::$kind => {
                        Command::$kind(parse_subcommand::<$builder>(rest)?.try_into()?)
                    }
                )*)*
                CommandKind::Help => {
                    // TODO: better help message
                    println!("\nUSAGE:\n    [COMMAND]\n\nCOMMANDS:");
                    for kind in CommandKind::KINDS {
                        print!("    ");
                        for key in kind.keys() {
                            print!("{} ", key.completions()[0]);
                        }
                        println!();
                    }
                    println!();
                    // TODO: indicate help message was printed
                    return Err(Error::ParseError("help"));
                }
            })
        }
        pub fn parse_completion(input: &str) -> Result<(Option<Builder>, Option<(&str, Completion)>), Error> {
            let (rest, kind) = match parse_command_kind(input, true) {
                Ok(ok) => ok,
                Err(err) => return Ok((None, err)),
            };
            Ok(match kind {
                $($(
                    CommandKind::$kind => {
                        let (s, (builder, completion)) = parse_subcommand_completion(rest)?;
                        (
                            Some(Builder::$builder(builder)),
                            completion.map(|c| (s, c)),
                        )
                    }
                )*)*
                CommandKind::Help => return Err(Error::ParseError("help")),
            })
        }
    }
}

macro_rules! command_keys {
    ($($( $key:ident => $lits:expr )+$(,)?)*) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum CommandKey {
            $($( $key, )*)*
        }
        impl CommandKey {
            pub fn completions<'a>(&'a self) -> &'static [&'static str] {
                match self {
                    $($( CommandKey::$key => &$lits, )*)*
                }
            }
            fn parse<'a>(&'a self, input: &'a str) -> IResult<Self> {
                map(
                    |i| parse_literal(i, self.completions()),
                    |_| self.to_owned(),
                )(input)
            }
        }
    }
}

macro_rules! opt_keys {
    ($($( $key:ident => $lits:expr )+$(,)?)*) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum OptKey {
            Unknown,
            $($( $key, )*)*
        }

        impl OptKey {
            pub fn completions<'a>(&'a self) -> &'static [&'static str] {
                match &self {
                    OptKey::Unknown => &[],
                    $($( OptKey::$key => &$lits, )*)*
                }
            }
            fn parse<'a>(&'a self, input: &'a str) -> IResult<&str> {
                if *self == OptKey::Unknown {
                    // TODO: Parse value too.
                    return cut(recognize(delimited(
                        alt((tag("--"), tag("-"))),
                        alphanumeric1,
                        eoo,
                    )))(input);
                }
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
    }
}

macro_rules! command_parsing {
    ($($( $key:ident => [ $($( $subkey:ident )+$(,)?)* ] )+$(,)?)*) => {
        // TODO: HashMap of CommandKey => &[CommandKey]
        const CMDS: &'static [CommandKey] = &[ $($( CommandKey::$key, )*)* ];

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
                $($( CommandKey::$key => &[ $($( CommandKey::$subkey, )*)* ], )*)*
                _ => unreachable!(),
            };

            Err(match terminated(word, space1)(rest) {
                Ok(_) => None,
                _ => Some((rest, Completion::Command(sub_cmds))),
            })
        }

    }
}

commands!(
    (Create Request) => (CreateRequest, CreateRequestBuilder),
    (Create Variable) => (CreateVariable, CreateVariableBuilder),
    (Print Requests) => (PrintRequests, PrintRequestsBuilder),
    (Print Variables) => (PrintVariables, PrintVariablesBuilder),
    (Print Environments) => (PrintEnvironments, PrintEnvironmentsBuilder),
    (Print Workspaces) => (PrintWorkspaces, PrintWorkspacesBuilder),
    (Set Environment) => (SetEnvironment, SetEnvironmentBuilder),
);

command_parsing!(
    Help => [],
    Create => [Request, Variable],
    Print => [Requests, Variables, Environments, Workspaces],
    Set => [Environment],
);

command_keys!(
    Help => ["help", "h", "what", "wut", "?"],
    Create => ["create", "new", "add", "c"],
    Print => ["print", "get", "show", "p"],
    Request => ["request", "req", "r"],
    Requests => ["requests", "request", "reqs", "req", "r"],
    Variable => ["variable", "var", "v"],
    Variables => ["variables", "variable", "vars", "var", "v"],
    Environment => ["environment", "env", "e"],
    Environments => ["environments", "environment", "envs", "env", "e"],
    Workspace => ["workspace", "ws", "w"],
    Workspaces => ["workspaces", "workspace", "ws", "w"],
    Set => ["set", "use"],
);

opt_keys!(
    Help => ["--help", "-h"],
    Header => ["--header", "-H"],
    Method => ["--method", "-m"],
);

#[derive(Debug, PartialEq, Clone)]
pub enum ArgKey {
    Unknown,
    Name,
    URL,
}

trait CmdLineBuilder: Default {
    const ARGS: &'static [ArgKey];
    const OPTS: &'static [OptKey];
    fn add_arg<S: Into<String>>(&mut self, _: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        Err(ParseError {
            kind: ParseErrorKind::InvalidArg,
            word: arg,
        })
    }
    fn add_opt<S: Into<String>>(&mut self, _: OptKey, arg: S) -> Result<(), ParseError<S>> {
        Err(ParseError {
            kind: ParseErrorKind::InvalidArg,
            word: arg,
        })
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

fn word(input: &str) -> IResult<&str> {
    // return an error if the input is empty
    take(1_usize)(input)?;

    let esc_single = escaped(none_of("\\'"), '\\', tag("'"));
    let esc_double = escaped(none_of("\\\""), '\\', tag("\""));
    let esc_space = escaped(none_of("\\ \t'\""), '\\', one_of(" \t'\""));
    alt((
        terminated(
            delimited(tag("'"), alt((esc_single, tag(""))), tag("'")),
            eow,
        ),
        terminated(
            delimited(tag("\""), alt((esc_double, tag(""))), tag("\"")),
            eow,
        ),
        terminated(esc_space, eow),
    ))(input)
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

fn parse_subcommand<B>(input: &str) -> Result<B, ParseError<&str>>
where
    B: CmdLineBuilder,
{
    let parser = |i| _parse_subcommand(i, false);
    let (_, builder): (_, B) = map(parser, |(b, _)| b)(input)?;
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
    let err = |e| nom::Err::Error(dbg!(e));
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
                Err(ret @ nom::Err::Failure(_)) if !completion => return Err(ret),
                Err(nom::Err::Failure(err)) if completion => {
                    let completion = builder.get_completion(Completion::OptValue(opt.to_owned()));
                    return Ok((err.word, (builder, completion)));
                }
                // Recoverable error, do nothing and try the next parser.
                _ => (),
            }
        }
        // TODO: Try to parse any flags.
        // Nothing successfully parsed the input, try adding it as an unknown option.
        // TODO: get the key and value from parsing
        // let flag;
        // (input, flag, arg) = OptKey::parse_unknown(input)?;
        (input, arg) = dbg!(OptKey::Unknown.parse(input))?;
        builder.add_opt(OptKey::Unknown, arg).map_err(err)?;
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
        assert!(matches!(
            parse_command_kind("p v", true),
            Err(Some(("v", _))),
        ));
    }
}
