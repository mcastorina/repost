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
    const HELP: &'static str = "Print existing variables in the workspace";

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
        println!("    Print information about variables in the current workspace. If no filters");
        println!("    are provided, all variables are printed. If one or more more filters are");
        println!("    provided, the output will show variables that match any of the given filters.");
        println!("\nUSAGE:\n    print variables [filter]...");
        println!("\nARGS:");
        println!("    [filter]...    Print variables matching the filter");
        println!("\n");
    }
}

impl From<PrintVariablesBuilder> for PrintVariables {
    fn from(builder: PrintVariablesBuilder) -> Self {
        PrintVariables {
            filters: builder.filters,
        }
    }
}
