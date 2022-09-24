use super::IResult;
use super::{opt_header, opt_method};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

#[derive(Debug, PartialEq, Eq, Clone)]
struct CreateRequest {
    name: String,
    url: String,
    method: Option<String>,
    headers: Vec<String>,
    // TODO: blob body
    body: Option<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateRequestBuilder {
    pub name: Option<String>,
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Vec<String>,
    // TODO: blob body
    pub body: Option<String>,
}

impl CmdLineBuilder for CreateRequestBuilder {
    const ARGS: &'static [ArgKey] = &[ArgKey::Name, ArgKey::URL];
    const OPT_PARSERS: &'static [fn(&str) -> IResult<(OptKey, &str)>] = &[opt_header, opt_method];

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()> {
        match key {
            ArgKey::Name => Ok(self.name = Some(arg.into())),
            ArgKey::URL => Ok(self.url = Some(arg.into())),
            _ => Err(()),
        }
    }
    fn add_opt<S: Into<String>>(&mut self, key: OptKey, arg: S) -> Result<(), ()> {
        match key {
            OptKey::Header => self.headers.push(arg.into()),
            OptKey::Method => self.method = Some(arg.into()),
            _ => return Err(()),
        }
        Ok(())
    }
    fn get_completion(&self, kind: Completion) -> Option<Completion> {
        match kind {
            Completion::Arg(ArgKey::Unknown) => None,
            _ => Some(kind),
        }
    }
}

impl TryFrom<CreateRequestBuilder> for CreateRequest {
    type Error = ();
    fn try_from(builder: CreateRequestBuilder) -> Result<Self, Self::Error> {
        todo!()
    }
}
