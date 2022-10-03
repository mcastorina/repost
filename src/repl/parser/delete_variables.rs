use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};
use crate::cmd;
use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeleteVariables {
    pub name_or_ids: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeleteVariablesBuilder {
    pub name_or_ids: Vec<String>,
}

impl CmdLineBuilder for DeleteVariablesBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Delete variables";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Unknown => Ok(self.name_or_ids.push(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
    #[rustfmt::skip]
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Delete variables across all environments. If an environment is set in the REPL, only");
        println!("    variables from that environment will be deleted.");
        println!("\nUSAGE:\n    delete variables [name|id]...");
        println!("\nARGS:");
        println!("    [name|id]    Name or integer ID of the variable to delete");
        println!("\n");
    }
}

impl From<DeleteVariablesBuilder> for DeleteVariables {
    fn from(builder: DeleteVariablesBuilder) -> Self {
        DeleteVariables {
            name_or_ids: builder.name_or_ids,
        }
    }
}

impl From<DeleteVariables> for cmd::DeleteVariablesArgs {
    fn from(args: DeleteVariables) -> Self {
        Self {
            name_or_ids: args.name_or_ids,
        }
    }
}
