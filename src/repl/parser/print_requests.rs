use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintRequests {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintRequestsBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintRequestsBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Unknown => Ok(self.filters.push(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
}

impl TryFrom<PrintRequestsBuilder> for PrintRequests {
    type Error = ();
    fn try_from(builder: PrintRequestsBuilder) -> Result<Self, Self::Error> {
        Ok(PrintRequests {
            filters: builder.filters,
        })
    }
}