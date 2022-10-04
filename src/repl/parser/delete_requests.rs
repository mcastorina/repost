use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};
use crate::cmd;
use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeleteRequests {
    pub names: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeleteRequestsBuilder {
    pub names: Vec<String>,
}

impl CmdLineBuilder for DeleteRequestsBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Delete requests";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Unknown => Ok(self.names.push(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
    #[rustfmt::skip]
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Delete requests by unique name.");
        println!("\nUSAGE:\n    delete requests [name]...");
        println!("\nARGS:");
        println!("    [name]    Unique name of the request to delete");
        println!("\n");
    }
}

impl From<DeleteRequestsBuilder> for DeleteRequests {
    fn from(builder: DeleteRequestsBuilder) -> Self {
        DeleteRequests {
            names: builder.names,
        }
    }
}

impl From<DeleteRequests> for cmd::DeleteRequestsArgs {
    fn from(args: DeleteRequests) -> Self {
        Self {
            names: args.names,
        }
    }
}

