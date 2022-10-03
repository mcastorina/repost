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
    const HELP: &'static str = "Print existing environments in the workspace";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Unknown => Ok(self.filters.push(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Environments are derived from existing variables. If no filters are");
        println!("    provided, all environments are printed. If one or more more filters");
        println!("    are provided, the output will show environments that match any of the");
        println!("    given filters.");
        println!("\nUSAGE:\n    print environments [filter]...");
        println!("\nARGS:");
        println!("    [filter]...    Print environments matching the filter");
        println!("\n");
    }
}

impl From<PrintEnvironmentsBuilder> for PrintEnvironments {
    fn from(builder: PrintEnvironmentsBuilder) -> Self {
        PrintEnvironments {
            filters: builder.filters,
        }
    }
}
