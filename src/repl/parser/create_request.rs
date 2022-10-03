use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};
use crate::cmd;
use crate::error::Error;
use reqwest::Method;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CreateRequest {
    pub name: String,
    pub url: String,
    pub method: Option<String>,
    pub headers: Vec<String>,
    // TODO: blob body
    pub body: Option<String>,
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
    const OPTS: &'static [OptKey] = &[OptKey::Header, OptKey::Method];
    const HELP: &'static str = "Create a named HTTP request";

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Name => Ok(self.name = Some(arg.into())),
            ArgKey::URL => Ok(self.url = Some(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
    fn add_opt<S: Into<String>>(&mut self, key: OptKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            OptKey::Header => self.headers.push(arg.into()),
            OptKey::Method => self.method = Some(arg.into()),
            _ => {
                return Err(ParseError {
                    kind: ParseErrorKind::InvalidOpt,
                    word: arg,
                })
            }
        }
        Ok(())
    }
    fn get_completion(&self, kind: Completion) -> Option<Completion> {
        match kind {
            Completion::Arg(ArgKey::Unknown) => None,
            _ => Some(kind),
        }
    }
    #[rustfmt::skip]
    fn usage(&self) {
        println!("{}\n", Self::HELP);
        println!("    Create a named request to run. Variables may be used in the form {{variable}}");
        println!("    in the URL, headers, and body. If method is not provided, it will be inferred");
        println!("    from the name of the request, defaulting to GET.");
        println!("\nUSAGE:\n    create request [OPTIONS] <name> <url>");
        println!("\nARGS:");
        println!("    <name>    Name of the request");
        println!("    <url>     HTTP request URL");
        println!("\nOPTIONS:");
        println!("    -H, --header <headers>...    HTTP request headers");
        println!("    -m, --method <method>        HTTP request method");
        println!("\n");
    }
}

impl TryFrom<CreateRequestBuilder> for CreateRequest {
    type Error = Error;
    fn try_from(builder: CreateRequestBuilder) -> Result<Self, Self::Error> {
        if builder.name.is_none() {
            return Err(Error::ParseError("Missing required argument: NAME"));
        }
        if builder.url.is_none() {
            return Err(Error::ParseError("Missing required argument: URL"));
        }
        Ok(CreateRequest {
            name: builder.name.unwrap(),
            url: builder.url.unwrap(),
            headers: builder.headers,
            method: builder.method,
            body: builder.body,
        })
    }
}

impl TryFrom<CreateRequest> for cmd::CreateRequestArgs {
    type Error = Error;
    fn try_from(args: CreateRequest) -> Result<Self, Self::Error> {
        let header = |h: String| {
            h.split_once(':')
                .map(|(k, v)| (k.to_string(), v.trim_start().to_string()))
                .ok_or(Error::ParseError("Invalid header"))
        };
        Ok(Self {
            name: args.name,
            url: args.url,
            headers: args
                .headers
                .into_iter()
                .map(header)
                .collect::<Result<Vec<_>, _>>()?,
            method: args
                .method
                .and_then(|m| Method::from_bytes(m.as_bytes()).ok())
                .unwrap_or(Method::GET),
            body: args.body,
        })
    }
}
