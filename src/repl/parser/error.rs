use nom;

use std::collections::HashMap;

pub type IResult<'a, O> = nom::IResult<&'a str, O, ParseError<'a>>;

#[derive(Debug, PartialEq)]
pub struct ParseError<'a> {
    pub state: Option<ParseState<'a>>,
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct ParseState<'a> {
    pub last_option: Option<&'a str>,
    pub options: HashMap<&'a str, Vec<&'a str>>,
    pub args: Vec<&'a str>,
    pub next: &'a str,
    pub rest: &'a str,
}

impl<'a> ParseError<'a> {
    pub fn new(message: String) -> Self {
        Self {
            message: Some(message),
            state: None,
        }
    }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<&str> for ParseError<'a> {
    fn from_error_kind(_input: &str, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind))
    }

    fn append(_input: &str, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
