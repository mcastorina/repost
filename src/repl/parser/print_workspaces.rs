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
    #[rustfmt::skip]
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Print a list of workspaces found in the configured data directory. A");
        println!("    workspace is where all requests and variables are stored. If no filters");
        println!("    are provided, all workspaces are printed. If one or more more filters are");
        println!("    provided, the output will show workspaces that match any of the given");
        println!("    filters.");
        println!("\nUSAGE:\n    print workspaces [filter]...");
        println!("\nARGS:");
        println!("    [filter]...    Print workspaces matching the filter");
        println!("\n");
    }
}

impl From<PrintWorkspacesBuilder> for PrintWorkspaces {
    fn from(builder: PrintWorkspacesBuilder) -> Self {
        PrintWorkspaces {
            filters: builder.filters,
        }
    }
}
