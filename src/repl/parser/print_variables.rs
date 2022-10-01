use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintVariables {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintVariablesBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintVariablesBuilder {
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

impl From<PrintVariablesBuilder> for PrintVariables {
    fn from(builder: PrintVariablesBuilder) -> Self {
        PrintVariables {
            filters: builder.filters,
        }
    }
}
