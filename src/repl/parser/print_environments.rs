use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintEnvironments {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintEnvironmentsBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintEnvironmentsBuilder {
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

impl TryFrom<PrintEnvironmentsBuilder> for PrintEnvironments {
    type Error = ();
    fn try_from(builder: PrintEnvironmentsBuilder) -> Result<Self, Self::Error> {
        Ok(PrintEnvironments {
            filters: builder.filters,
        })
    }
}
