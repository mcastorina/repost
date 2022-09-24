use super::IResult;
use super::{CmdLineBuilder, Completion, OptKey};
use super::{opt_header, opt_method};

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
    pub completion: Option<CreateRequestCompletion>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CreateRequestCompletion {
    ArgName,
    ArgURL,
    OptKey,
    HeaderValue,
    MethodValue,
}

impl CmdLineBuilder for CreateRequestBuilder {
    const OPT_PARSERS: &'static [fn(&str) -> IResult<(OptKey, &str)>] =
        &[opt_header, opt_method];

    fn add_arg<S: Into<String>>(&mut self, arg: S) -> Result<(), ()> {
        match (&self.name, &self.url) {
            (Some(_), Some(_)) => Err(()),
            (None, _) => Ok(self.name = Some(arg.into())),
            (_, None) => Ok(self.url = Some(arg.into())),
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
    fn set_completion(&mut self, kind: Completion) {
        self.completion = match kind {
            Completion::Arg => match (&self.name, &self.url) {
                    (Some(_), Some(_)) => None,
                    (None, _) => Some(CreateRequestCompletion::ArgName),
                    (_, None) => Some(CreateRequestCompletion::ArgURL),
                }
            Completion::OptKey => Some(CreateRequestCompletion::OptKey),
            Completion::OptValue(key) => Some(match key {
                OptKey::Header => CreateRequestCompletion::HeaderValue,
                OptKey::Method => CreateRequestCompletion::MethodValue,
                _ => unreachable!(),
            })
        }
    }
}

impl TryFrom<CreateRequestBuilder> for CreateRequest {
    type Error = ();
    fn try_from(builder: CreateRequestBuilder) -> Result<Self, Self::Error> {
        todo!()
    }
}
