use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintRequests {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintRequestsBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintRequestsBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];
    const HELP: &'static str = "Print existing requests in the workspace";

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
        println!("    Print information about requests in the current workspace. If no filters");
        println!("    are provided, all requests are printed. If one or more more filters are");
        println!("    provided, the output will show requests that match any of the given filters.");
        println!("\nUSAGE:\n    print requests [filter]...");
        println!("\nARGS:");
        println!("    [filter]...    Print requests matching the filter");
        println!("\n");
    }
}

impl From<PrintRequestsBuilder> for PrintRequests {
    fn from(builder: PrintRequestsBuilder) -> Self {
        PrintRequests {
            filters: builder.filters,
        }
    }
}
