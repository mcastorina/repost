use super::{ArgKey, CmdLineBuilder, Completion, OptKey};

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

    fn add_arg<S: Into<String>>(&mut self, key: ArgKey, arg: S) -> Result<(), ()> {
        match key {
            ArgKey::Name => Ok(self.name = Some(arg.into())),
            ArgKey::Unknown => Ok(self.env_vals.push(arg.into())),
            _ => Err(()),
        }
    }
}

impl TryFrom<CreateVariableBuilder> for CreateVariable {
    type Error = ();
    fn try_from(builder: CreateVariableBuilder) -> Result<Self, Self::Error> {
        if builder.name.is_none() || builder.env_vals.len() == 0 {
            return Err(());
        }
        Ok(CreateVariable {
            name: builder.name.unwrap(),
            env_vals: builder.env_vals,
        })
    }
}
