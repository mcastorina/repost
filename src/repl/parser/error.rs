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
    Fixed(&'static [&'static str]),
    InvalidArg,
    InvalidOpt,
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(input: I, _kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ParseErrorKind::Unknown,
            word: input,
        }
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
