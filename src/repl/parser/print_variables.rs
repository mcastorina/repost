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

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()> {
        match key {
            ArgKey::Unknown => Ok(self.filters.push(arg.into())),
            _ => Err(()),
        }
    }
}

impl TryFrom<PrintVariablesBuilder> for PrintVariables {
    type Error = ();
    fn try_from(builder: PrintVariablesBuilder) -> Result<Self, Self::Error> {
        Ok(PrintVariables {
            filters: builder.filters,
        })
    }
}
