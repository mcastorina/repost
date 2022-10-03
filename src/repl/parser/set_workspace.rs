use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SetWorkspace {
    pub workspace: Option<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SetWorkspaceBuilder {
    pub workspace: Option<String>,
}

impl CmdLineBuilder for SetWorkspaceBuilder {
    const ARGS: &'static [ArgKey] = &[ArgKey::Name];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Set the REPL's workspace";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Name => Ok(self.workspace = Some(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
}

impl From<SetWorkspaceBuilder> for SetWorkspace {
    fn from(builder: SetWorkspaceBuilder) -> Self {
        SetWorkspace {
            workspace: builder.workspace,
        }
    }
}
