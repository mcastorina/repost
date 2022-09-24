mod error;
mod create_request;

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
use create_request::CreateRequestBuilder;

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

macro_rules! opt {
    ($name:ident, $key:expr, $($( $lit:expr )+$(,)?)*) => {
        fn $name(input: &str) -> IResult<(OptKey, &str)> {
            let (rest, _) = terminated(alt(($($( tag($lit), )*)*)), eoo)(input)?;
            let (rest, sep) = cut(alt((space1, tag("="))))(rest)?;
            match sep {
                "=" => cut(word)(rest),
                _ => cut(verify(word, |s: &str| !s.starts_with('-')))(rest),
            }.map(|(rest, val)| (rest, ($key, val)))
        }
    }
}

macro_rules! flag {
    ($name:ident, $($( $lit:expr )+$(,)?)*) => {
        fn $name(input: &str) -> IResult<()> {
            map(terminated(alt(($($( tag($lit), )*)*)), eow), |_| ())(input)
        }
    }
}

macro_rules! parser {
    ($name:ident, $builder:ident) => {
        fn $name(mut input: &str, completion: bool) -> IResult<$builder> {
            let mut builder = $builder::default();
            let mut double_dash = false;
            let mut arg: &str;
            let err = |_| nom::Err::Error(ParseError::default());
            let done_parsing = if completion {
                |s: &str| terminated(word, space1)(s).is_err()
            } else {
                |s: &str| s.len() == 0
            };
            'main : loop {
                input = trim_leading_space(input);
                if done_parsing(input) {
                    break;
                }
                if double_dash {
                    (input, arg) = word(input)?;
                    builder.add_arg(arg).map_err(err)?;
                    continue;
                }
                if let Ok(ret) = eol(input) {
                    (input, _) = ret;
                    double_dash = true;
                    continue;
                }
                for opt in $builder::PARSERS {
                    match opt(input) {
                        Ok(ret) => {
                            let key;
                            (input, (key, arg)) = ret;
                            if completion && done_parsing(input) {
                                builder.set_completion(Completion::OptValue(key));
                                return Ok((arg, builder));
                            }
                            builder.add_opt(key, arg).map_err(err)?;
                            continue 'main;
                        }
                        Err(ret @ Failure(_)) => {
                            return Err(ret);
                        }
                        _ => (),
                    }
                }
                if let Ok(ret) = verify(word, |s: &str| !s.starts_with('-'))(input) {
                    (input, arg) = ret;
                    builder.add_arg(arg).map_err(err)?;
                    continue;
                }
                return Err(nom::Err::Error(ParseError{
                    kind: ParseErrorKind::Unknown,
                    word: input,
                }))
            }
            if completion {
                match (double_dash, input.starts_with('-')) {
                    (true, _) => builder.set_completion(Completion::Arg),
                    (_, true) => builder.set_completion(Completion::OptKey),
                    (_, false) => builder.set_completion(Completion::Arg),
                }
            }
            Ok((input, builder))
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
enum OptKey {
    Header,
    Method,
}

opt!(opt_header, OptKey::Header, "--header", "-H");
opt!(opt_method, OptKey::Method, "--method", "-m");

trait CmdLineBuilder {
    const PARSERS: &'static [fn(&str) -> IResult<(OptKey, &str)>];
    fn add_arg<S: Into<String>>(&mut self, arg: S) -> Result<(), ()>;
    fn add_opt<S: Into<String>>(&mut self, key: OptKey, arg: S) -> Result<(), ()>;
    fn set_completion(&mut self, kind: Completion);
}

#[derive(Debug, PartialEq, Clone)]
enum Completion {
    Arg,
    OptKey,
    OptValue(OptKey),
}

fn trim_leading_space(s: &str) -> &str {
    space0::<_, ParseError<&str>>(s).expect("stripping leading whitespace").0
}

fn parse<B>(mut input: &str, completion: bool) -> IResult<B>
where
    B: CmdLineBuilder + Default
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
    'main : loop {
        input = trim_leading_space(input);
        if done_parsing(input) {
            break;
        }
        if double_dash {
            (input, arg) = word(input)?;
            builder.add_arg(arg).map_err(err)?;
            continue;
        }
        if let Ok(ret) = eol(input) {
            (input, _) = ret;
            double_dash = true;
            continue;
        }
        for opt in B::PARSERS {
            match opt(input) {
                Ok(ret) => {
                    let key;
                    (input, (key, arg)) = ret;
                    if completion && done_parsing(input) {
                        builder.set_completion(Completion::OptValue(key));
                        return Ok((arg, builder));
                    }
                    builder.add_opt(key, arg).map_err(err)?;
                    continue 'main;
                }
                Err(ret @ Failure(_)) => {
                    return Err(ret);
                }
                _ => (),
            }
        }
        if let Ok(ret) = verify(word, |s: &str| !s.starts_with('-'))(input) {
            (input, arg) = ret;
            builder.add_arg(arg).map_err(err)?;
            continue;
        }
        return Err(nom::Err::Error(ParseError{
            kind: ParseErrorKind::Unknown,
            word: input,
        }))
    }
    if completion {
        match (double_dash, input.starts_with('-')) {
            (true, _) => builder.set_completion(Completion::Arg),
            (_, true) => builder.set_completion(Completion::OptKey),
            (_, false) => builder.set_completion(Completion::Arg),
        }
    }
    Ok((input, builder))
}

fn create_request(mut input: &str) -> IResult<CreateRequestBuilder> {
    parse(input, false)
}

fn create_request_completion(mut input: &str) -> IResult<CreateRequestBuilder> {
    parse(input, true)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{ParseError, ParseErrorKind};
    use maplit::hashmap;
    use nom::Err::{Error, Failure};
    use super::create_request::*;

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
        for (input, expected) in tests {
            assert_eq!(opt_header(input), Ok(("", (OptKey::Header, *expected))));
        }
        assert!(matches!(opt_header("-H"), Err(_)));
        assert!(matches!(opt_header("--header"), Err(_)));
        assert!(matches!(opt_header("--header -H"), Err(_)));
        assert!(matches!(opt_header("--header --foo"), Err(_)));
        assert!(matches!(opt_header("--oops"), Err(_)));
    }

    #[test]
    fn test_create_request() {
        assert_eq!(create_request("foo bar"), Ok(("", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: Some("bar".to_string()),
            method: None,
            headers: vec![],
            body: None,
            completion: None,
        })));
        assert_eq!(create_request("foo -H yay bar"), Ok(("", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: Some("bar".to_string()),
            method: None,
            headers: vec!["yay".to_string()],
            body: None,
            completion: None,
        })));
        assert_eq!(create_request("--method baz -- foo bar"), Ok(("", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: Some("bar".to_string()),
            method: Some("baz".to_string()),
            headers: vec![],
            body: None,
            completion: None,
        })));
        assert_eq!(create_request("-- --foo --bar"), Ok(("", CreateRequestBuilder{
            name: Some("--foo".to_string()),
            url: Some("--bar".to_string()),
            method: None,
            headers: vec![],
            body: None,
            completion: None,
        })));
        assert!(matches!(create_request("foo bar baz"), Err(_)));
        assert!(matches!(create_request("--foo --bar"), Err(_)));
        assert!(matches!(create_request("--foo"), Err(_)));
        assert!(matches!(create_request("foo bar -X"), Err(_)));
    }

    #[test]
    fn test_create_request_complete() {
        assert_eq!(create_request_completion("foo"), Ok(("foo", CreateRequestBuilder{
            name: None,
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::ArgName),
        })));
        assert_eq!(create_request_completion("foo bar"), Ok(("bar", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::ArgURL),
        })));
        assert_eq!(create_request_completion("foo "), Ok(("", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::ArgURL),
        })));
        assert_eq!(create_request_completion("foo -"), Ok(("-", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::OptKey),
        })));
        assert_eq!(create_request_completion("foo -H f"), Ok(("f", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::HeaderValue),
        })));
        assert_eq!(create_request_completion("foo --he"), Ok(("--he", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::OptKey),
        })));
        assert_eq!(create_request_completion("-- --foo"), Ok(("--foo", CreateRequestBuilder{
            name: None,
            url: None,
            method: None,
            headers: vec![],
            body: None,
            completion: Some(CreateRequestCompletion::ArgName),
        })));
        assert_eq!(create_request_completion("foo bar baz"), Ok(("baz", CreateRequestBuilder{
            name: Some("foo".to_string()),
            url: Some("bar".to_string()),
            method: None,
            headers: vec![],
            body: None,
            completion: None,
        })));
        assert!(matches!(create_request_completion("foo bar baz "), Err(_)));
    }
}
