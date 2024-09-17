use std::{error::Error as StdError, fmt, ops};

use miette::Diagnostic;

#[derive(Debug)]
pub struct AnyError(anyhow::Error);

impl ops::Deref for AnyError {
    type Target = anyhow::Error;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<anyhow::Error> for AnyError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl Diagnostic for AnyError {}

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for AnyError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
