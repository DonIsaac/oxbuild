mod options;
mod oxc_options;

use std::{mem, ops::ControlFlow, path::Path};

use oxc::{
    codegen::{CodegenOptions, CodegenReturn},
    diagnostics::{Diagnostics, OxcDiagnostic},
    isolated_declarations::IsolatedDeclarationsOptions,
    parser::ParserReturn,
    span::SourceType,
    transformer::TransformOptions,
    CompilerInterface,
};

pub use options::CompileOptions;

use self::oxc_options::{
    to_codegen_options, to_isolated_declarations_options, to_transform_options,
};

static_assertions::assert_impl_all!(CompileOptions: Send, Sync);

#[derive(Debug, Clone)]
pub struct CompiledOutput {
    pub source_text: String,
    pub source_map_json: Option<String>,
    pub declarations: Option<String>,
    pub declarations_map_json: Option<String>,
}

pub fn compile(
    options: &CompileOptions,
    source_path: &Path,
    source_text: &str,
) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
    let source_type = SourceType::from_path(source_path).unwrap();
    let mut compiler = Compiler::new(options);
    compiler.compile(source_text, source_type, source_path);
    compiler.finish()
}

#[derive(Default)]
struct CompiledOutputBuilder {
    source_text: Option<String>,
    source_map_json: Option<String>,
    declarations: Option<String>,
    declarations_map_json: Option<String>,
}

impl CompiledOutputBuilder {
    fn finish(self) -> CompiledOutput {
        CompiledOutput {
            source_text: self.source_text.unwrap_or_default(),
            source_map_json: self.source_map_json,
            declarations: self.declarations,
            declarations_map_json: self.declarations_map_json,
        }
    }
}

struct Compiler<'a> {
    options: &'a CompileOptions,
    transform_options: TransformOptions,
    codegen_options: CodegenOptions,
    output: CompiledOutputBuilder,
    diagnostics: Vec<OxcDiagnostic>,
}

impl<'a> Compiler<'a> {
    fn new(options: &'a CompileOptions) -> Self {
        Self {
            options,
            transform_options: to_transform_options(options),
            codegen_options: to_codegen_options(),
            output: CompiledOutputBuilder::default(),
            diagnostics: Vec::new(),
        }
    }

    fn finish(self) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
        if self.diagnostics.is_empty() {
            Ok(self.output.finish())
        } else {
            Err(self.diagnostics)
        }
    }
}

impl CompilerInterface for Compiler<'_> {
    fn handle_errors(&mut self, diagnostics: Diagnostics) {
        self.diagnostics.extend(diagnostics);
    }

    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.transform_options)
    }

    fn isolated_declaration_options(&self) -> Option<IsolatedDeclarationsOptions> {
        self.options
            .declarations_options()
            .map(to_isolated_declarations_options)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(self.codegen_options.clone())
    }

    fn enable_sourcemap(&self) -> bool {
        true
    }

    fn after_parse(&mut self, parser_return: &mut ParserReturn) -> ControlFlow<()> {
        if parser_return.panicked || !parser_return.diagnostics.is_empty() {
            let diagnostics = mem::take(&mut parser_return.diagnostics);
            self.handle_errors(diagnostics);
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    }

    fn after_isolated_declarations(&mut self, ret: CodegenReturn<'_>) {
        self.output.declarations = Some(ret.code);
        // Historically declaration sourcemaps were not emitted, so keep that behavior
        // even though the shared codegen options can now produce one.
        self.output.declarations_map_json = None;
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.output.source_text = Some(ret.code);
        self.output.source_map_json = ret.map.map(|map| map.to_json_string());
    }
}
