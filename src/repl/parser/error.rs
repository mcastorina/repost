use crate::error::Error;
use nom;

pub type IResult<'a, O> = nom::IResult<&'a str, O, ParseError<&'a str>>;

#[derive(Debug, PartialEq, Default)]
pub struct ParseError<I> {
    pub kind: ParseErrorKind,
    pub word: I,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ParseErrorKind {
    #[default]
    Unknown,
    InvalidArg,
    InvalidOpt,
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(input: I, _kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ParseErrorKind::default(),
            word: input,
        }
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<ParseError<I>> for Error {
    fn from(err: ParseError<I>) -> Self {
        match err.kind {
            ParseErrorKind::Unknown => Error::ParseError("unknown"),
            ParseErrorKind::InvalidArg => Error::ParseError("invalid argument"),
            ParseErrorKind::InvalidOpt => Error::ParseError("invalid option"),
        }
    }
}

impl<I: Default> From<nom::Err<ParseError<I>>> for ParseError<I> {
    fn from(err: nom::Err<ParseError<I>>) -> Self {
        match err {
            nom::Err::Error(err) => err,
            nom::Err::Failure(err) => err,
            nom::Err::Incomplete(_) => ParseError::default(),
        }
    }
}

impl<I: Default> From<nom::Err<ParseError<I>>> for Error {
    fn from(err: nom::Err<ParseError<I>>) -> Self {
        let parse_error: ParseError<I> = err.into();
        parse_error.into()
    }
}
