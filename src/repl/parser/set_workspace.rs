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
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    A workspace is where all requests and variables are stored and is written");
        println!("    to disk for persistence in the configured data directory. The 'playground'");
        println!("    workspace uses in-memory storage and will be lost when repost exits.");
        println!("\nUSAGE:\n    set workspace [name]");
        println!("\nARGS:");
        println!("    [name]    Name of the workspace (default: playground)");
        println!("\n");
    }
}

impl From<SetWorkspaceBuilder> for SetWorkspace {
    fn from(builder: SetWorkspaceBuilder) -> Self {
        SetWorkspace {
            workspace: builder.workspace,
        }
    }
}
