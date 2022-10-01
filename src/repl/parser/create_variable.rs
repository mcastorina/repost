use super::error::{ParseError, ParseErrorKind};
use super::{ArgKey, CmdLineBuilder, Completion, OptKey};
use crate::cmd;
use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CreateVariable {
    pub name: String,
    pub env_vals: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateVariableBuilder {
    pub name: Option<String>,
    pub env_vals: Vec<String>,
}

impl CmdLineBuilder for CreateVariableBuilder {
    const ARGS: &'static [ArgKey] = &[ArgKey::Name];
    const OPTS: &'static [OptKey] = &[];

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ParseError<S>> {
        match key {
            ArgKey::Name => Ok(self.name = Some(arg.into())),
            ArgKey::Unknown => Ok(self.env_vals.push(arg.into())),
            _ => Err(ParseError {
                kind: ParseErrorKind::InvalidArg,
                word: arg,
            }),
        }
    }
}

impl TryFrom<CreateVariableBuilder> for CreateVariable {
    type Error = Error;
    fn try_from(builder: CreateVariableBuilder) -> Result<Self, Self::Error> {
        if builder.name.is_none() {
            return Err(Error::ParseError("Missing required argument: NAME"));
        }
        if builder.env_vals.len() == 0 {
            return Err(Error::ParseError("Expected at least one ENV=VAL argument"));
        }
        Ok(CreateVariable {
            name: builder.name.unwrap(),
            env_vals: builder.env_vals,
        })
    }
}

impl TryFrom<CreateVariable> for cmd::CreateVariableArgs {
    type Error = Error;
    fn try_from(args: CreateVariable) -> Result<Self, Self::Error> {
        let env_val = |h: String| {
            h.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or(Error::ParseError("Invalid environment value pair"))
        };
        Ok(Self {
            name: args.name,
            env_vals: args
                .env_vals
                .into_iter()
                .map(env_val)
                .collect::<Result<Vec<_>, _>>()?,
            source: "user".to_string(),
        })
    }
}
