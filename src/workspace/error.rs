use miette::Diagnostic;
use std::fmt::{self, Display};

#[derive(Debug)]
pub(super) struct AnyhowWrap {
    inner: anyhow::Error,
}
impl From<anyhow::Error> for AnyhowWrap {
    fn from(inner: anyhow::Error) -> Self {
        Self { inner }
    }
}
impl Display for AnyhowWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
impl std::error::Error for AnyhowWrap {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}
impl Diagnostic for AnyhowWrap {}
