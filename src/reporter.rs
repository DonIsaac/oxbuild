use oxc::diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult},
    DiagnosticService,
};

// re-export in case we want to modify/wrap/replace/whatever in the future
pub use oxc::diagnostics::{DiagnosticSender, GraphicalReportHandler};

#[must_use]
pub struct Reporter {
    inner: DiagnosticService,
}
struct GraphicalReporter {
    inner: GraphicalReportHandler,
}
impl Default for GraphicalReporter {
    fn default() -> Self {
        let inner = GraphicalReportHandler::new();
        Self { inner }
    }
}
impl DiagnosticReporter for GraphicalReporter {
    fn render_error(&mut self, error: oxc::diagnostics::Error) -> Option<String> {
        let mut buf = String::new();
        self.inner.render_report(&mut buf, error.as_ref()).ok()?;
        Some(buf)
    }
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        None // todo
    }
}
impl GraphicalReporter {
    fn into_reporter(self) -> Box<dyn DiagnosticReporter> {
        Box::new(self)
    }
}

impl Reporter {
    pub fn new() -> (Self, DiagnosticSender) {
        trace!("Creating diagnostics reporter");
        let inner = DiagnosticService::new(GraphicalReporter::default().into_reporter());
        let sender = inner.sender().clone();
        (Self { inner }, sender)
    }

    pub fn run(&mut self) {
        let mut out = std::io::stdout();
        self.inner.run(&mut out);
    }

    #[inline]
    pub fn errors_count(&self) -> usize {
        0 // todo
    }

    #[inline]
    pub fn warnings_count(&self) -> usize {
        0 // todo
    }
}
