use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Environment<'a> {
    pub name: Cow<'a, str>,
}

impl<'a> Environment<'a> {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self { name: name.into() }
    }
}

impl<'a, T: Into<Cow<'a, str>>> From<T> for Environment<'a> {
    fn from(s: T) -> Self {
        Environment::new(s)
    }
}

impl<'a> AsRef<str> for Environment<'a> {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
