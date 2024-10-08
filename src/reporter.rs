use oxc::diagnostics::DiagnosticService;

// re-export in case we want to modify/wrap/replace/whatever in the future
pub use oxc::diagnostics::DiagnosticSender;

#[must_use]
pub struct Reporter {
    inner: DiagnosticService,
}

impl Reporter {
    pub fn new() -> (Self, DiagnosticSender) {
        trace!("Creating diagnostics reporter");
        let inner = DiagnosticService::default();
        let sender = inner.sender().clone();
        (Self { inner }, sender)
    }

    pub fn run(&mut self) {
        self.inner.run();
    }

    #[inline]
    pub fn errors_count(&self) -> usize {
        self.inner.errors_count()
    }

    #[inline]
    pub fn warnings_count(&self) -> usize {
        self.inner.warnings_count()
    }
}
