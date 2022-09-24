use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintRequest {
    pub filters: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PrintRequestBuilder {
    pub filters: Vec<String>,
}

impl CmdLineBuilder for PrintRequestBuilder {
    const ARGS: &'static [ArgKey] = &[];
    const OPTS: &'static [OptKey] = &[];

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()> {
        match key {
            ArgKey::Unknown => Ok(self.filters.push(arg.into())),
            _ => Err(()),
        }
    }
}

impl TryFrom<PrintRequestBuilder> for PrintRequest {
    type Error = ();
    fn try_from(builder: PrintRequestBuilder) -> Result<Self, Self::Error> {
        Ok(PrintRequest {
            filters: builder.filters,
        })
    }
}
