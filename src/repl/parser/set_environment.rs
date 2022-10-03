use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SetEnvironment {
    pub environment: Option<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SetEnvironmentBuilder {
    pub environment: Option<String>,
}

impl CmdLineBuilder for SetEnvironmentBuilder {
    const ARGS: &'static [ArgKey] = &[ArgKey::Name];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Set the REPL's environment for variable substitution";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Name => Ok(self.environment = Some(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Environments are derived from existing variables. When the REPL has an");
        println!("    environment set, the variables from that environment will be used for");
        println!("    request variable substitution.");
        println!("\nUSAGE:\n    set environment [name]");
        println!("\nARGS:");
        println!("    [name]    Name of the environment (default: clears environment)");
        println!("\n");
    }
}

impl From<SetEnvironmentBuilder> for SetEnvironment {
    fn from(builder: SetEnvironmentBuilder) -> Self {
        SetEnvironment {
            environment: builder.environment,
        }
    }
}
