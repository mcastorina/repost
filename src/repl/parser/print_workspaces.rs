use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintWorkspaces {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintWorkspacesBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintWorkspacesBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Print available workspaces";

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

impl From<PrintWorkspacesBuilder> for PrintWorkspaces {
    fn from(builder: PrintWorkspacesBuilder) -> Self {
        PrintWorkspaces {
            filters: builder.filters,
        }
    }
}
