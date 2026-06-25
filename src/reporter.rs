use std::io::Write;

use oxc::diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult},
    DiagnosticService, Error, GraphicalReportHandler,
};

// re-export in case we want to modify/wrap/replace/whatever in the future
pub use oxc::diagnostics::DiagnosticSender;

#[must_use]
pub struct Reporter {
    inner: DiagnosticService,
}

impl Reporter {
    pub fn new() -> (Self, DiagnosticSender) {
        trace!("Creating diagnostics reporter");
        let (inner, sender) = DiagnosticService::new(Box::new(GraphicalReporter::default()));
        (Self { inner }, sender)
    }

    pub fn run(&mut self, writer: &mut dyn Write) -> DiagnosticResult {
        self.inner.run(writer)
    }
}

#[derive(Default)]
struct GraphicalReporter {
    handler: GraphicalReportHandler,
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler
            .render_report(&mut output, error.as_ref())
            .ok()?;
        Some(output)
    }
}
