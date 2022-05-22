mod error;

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

fn all<'a, O, F>(
    mut parser: F,
    kind: ParseErrorKind,
) -> impl FnMut(&'a str) -> nom::IResult<&'a str, O, ParseError<&'a str>>
where
    F: nom::Parser<&'a str, O, ParseError<&'a str>>,
{
    move |input: &str| match parser.parse(input.clone()) {
        Err(_) => match word(input) {
            Ok((_, word)) => Err(Error(ParseError { kind: kind.clone(), word: word })),
            _ => Err(Error(ParseError { kind: kind.clone(), word: input })),
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

fn verb(input: &str) -> IResult<CmdKind> {
    // get the sub command
    enum SubCmdKind {
        Print,
        Create,
    }
    let (rest, kind) = all(
        alt((
            map(print, |_| SubCmdKind::Print),
            map(create, |_| SubCmdKind::Create),
        )),
        ParseErrorKind::Fixed(&["print", "get", "show", "p", "create", "new", "add", "c"]),
    )(input)?;

    // we expect at least one space
    let (rest, _) = space1(rest)?;

    // parse the second command based on the first
    match kind {
        SubCmdKind::Print => all(
            alt((
                map(alt((requests, request)), |_| CmdKind::PrintRequests),
                map(alt((variables, variable)), |_| CmdKind::PrintVariables),
                map(alt((environments, environment)), |_| {
                    CmdKind::PrintEnvironments
                }),
                map(alt((workspaces, workspace)), |_| CmdKind::PrintWorkspaces),
            )),
            ParseErrorKind::Fixed(&[
                "requests", "reqs", "request", "req", "r",
                "variables", "vars", "variable", "var", "v",
                "environments", "envs", "environment", "env", "e",
                "workspaces", "workspace", "ws", "w",
            ]),
        )(rest),
        SubCmdKind::Create => all(
            alt((
                map(alt((requests, request)), |_| CmdKind::CreateRequest),
                map(alt((variables, variable)), |_| CmdKind::CreateVariable),
            )),
            ParseErrorKind::Fixed(&["request", "req", "r", "variable", "var", "v"]),
        )(rest),
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateRequestBuilder {
    name: Option<String>,
    url: Option<String>,
    method: Option<String>,
    headers: Vec<String>,
    // TODO: blob body
    body: Option<String>,
}

impl TryFrom<CreateRequestBuilder> for CreateRequest {
    type Error = ParseErrorKind;
    fn try_from(builder: CreateRequestBuilder) -> Result<Self, Self::Error> {
        todo!()
    }
}

macro_rules! create_request_args_err {
    ($parser:expr, $input:expr, $err:expr) => {
        $parser($input).map_err(|_: nom::Err<ParseError<_>>| Error(ParseError{
            word: $input,
            kind: $err,
        }))
    };
}

fn create_request_args(input: &str) -> IResult<CreateRequest> {
    let mut builder = CreateRequestBuilder::default();
    // loop over each word
    let (rest, arg) = create_request_args_err!(word, input, ParseErrorKind::CreateRequest(builder.clone()))?;
    builder.name = Some(arg.to_string());
    let (rest, _) = create_request_args_err!(space1, rest, ParseErrorKind::CreateRequest(builder.clone()))?;
    todo!()
}

#[cfg(test)]
mod test {
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
    fn test_verb() {
        assert_eq!(verb("create request"), Ok(("", CmdKind::CreateRequest)));
        assert_eq!(verb("c r"), Ok(("", CmdKind::CreateRequest)));
        assert_eq!(verb("print ws"), Ok(("", CmdKind::PrintWorkspaces)));
        assert_eq!(verb("show vars"), Ok(("", CmdKind::PrintVariables)));
        assert_eq!(
            verb("foo bar"),
            Err(Error(ParseError {
                kind: ParseErrorKind::Fixed(&[
                    "print", "get", "show", "p", "create", "new", "add", "c",
                ]),
                word: "foo"
            }))
        );
        assert_eq!(
            verb("create bar"),
            Err(Error(ParseError {
                kind: ParseErrorKind::Fixed(&["request", "req", "r", "variable", "var", "v",]),
                word: "bar"
            }))
        );
        assert_eq!(
            verb("create"),
            Err(Error(ParseError {
                kind: ParseErrorKind::Unknown,
                word: "",
            })),
        );
    }

    #[test]
    fn test_create_request_args() {
        assert_eq!(
            create_request_args(""),
            Err(Error(ParseError {
                word: "",
                kind: ParseErrorKind::CreateRequest(
                    CreateRequestBuilder {
                        name: None,
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    }
                )
            })),
        );

        assert_eq!(
            create_request_args("foo"),
            Err(Error(ParseError {
                word: "",
                kind: ParseErrorKind::CreateRequest(
                    CreateRequestBuilder {
                        name: Some("foo".to_string()),
                        url: None,
                        method: None,
                        headers: vec![],
                        body: None,
                    }
                )
            })),
        );
    }
}
