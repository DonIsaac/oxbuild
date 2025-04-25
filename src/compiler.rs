use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc::{
    codegen::{CodegenOptions, CodegenReturn},
    diagnostics::OxcDiagnostic,
    isolated_declarations::IsolatedDeclarationsOptions,
    parser::ParseOptions,
    span::SourceType,
    transformer::{JsxOptions, JsxRuntime, TransformOptions},
    CompilerInterface,
};
use oxc_sourcemap::SourceMap;

use crate::workspace::{Package, TsConfigJsx};

pub struct CompilerOptions {
    parse: ParseOptions,
    transform: TransformOptions,
    id: Option<IsolatedDeclarationsOptions>,
    // todo
    // minify: MinifierOptions,
    codegen: CodegenOptions,
    src: PathBuf,
    dist: PathBuf,
}

#[derive(Debug, Default)]
pub struct CompiledOutput {
    pub source_text: String,
    pub source_map: Option<SourceMap>,
    pub declarations: Option<String>,
    pub declarations_map: Option<SourceMap>,
}

pub struct OxBuild {
    pub options: Arc<CompilerOptions>,
    errors: Cell<Vec<OxcDiagnostic>>,
    results: Cell<CompiledOutput>,
}

impl OxBuild {
    pub fn new(options: Arc<CompilerOptions>) -> Self {
        Self {
            options,
            errors: Default::default(),
            results: Default::default(),
        }
    }
    fn take(&mut self) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
        if !self.errors.get_mut().is_empty() {
            return Err(self.errors.take());
        }
        Ok(self.results.take())
    }
    pub fn run(
        &mut self,
        source_text: &str,
        source_path: &Path,
    ) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
        let source_type = SourceType::from_path(source_path).unwrap();
        self.compile(source_text, source_type, source_path);
        self.take()
    }
}

impl CompilerInterface for OxBuild {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.get_mut().extend(errors);
    }
    fn check_semantic_error(&self) -> bool {
        true
    }
    fn parse_options(&self) -> ParseOptions {
        self.options.parse
    }
    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.options.transform)
    }
    fn enable_sourcemap(&self) -> bool {
        true
    }
    fn isolated_declaration_options(&self) -> Option<IsolatedDeclarationsOptions> {
        self.options.id
    }
    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(self.options.codegen.clone())
    }
    fn after_codegen(&mut self, ret: CodegenReturn) {
        let results = self.results.get_mut();
        results.source_text = ret.code;
        results.source_map = ret.map;
    }
    fn after_isolated_declarations(&mut self, ret: CodegenReturn) {
        let results = self.results.get_mut();
        results.declarations = Some(ret.code);
        results.declarations_map = ret.map;
    }
}

impl From<Package> for CompilerOptions {
    fn from(package: Package) -> Self {
        let mut transform = TransformOptions::default();
        let mut id: Option<IsolatedDeclarationsOptions> = None;

        let mut src: Option<PathBuf> = None;
        let mut dist: Option<PathBuf> = None;

        if let Some(co) = package.tsconfig.and_then(|t| t.compiler_options) {
            if let Some(target) = co.target {
                transform = TransformOptions::from_target(&target).unwrap()
            }
            if let Some(factory) = co.jsx_factory.as_ref() {
                transform.typescript.jsx_pragma = factory.clone().into();
            }
            if let Some(fragment) = co.jsx_fragment_factory.as_ref() {
                transform.typescript.jsx_pragma_frag = fragment.clone().into();
            }

            transform.decorator.legacy = co.experimental_decorators.unwrap_or(false);
            transform.decorator.emit_decorator_metadata =
                co.emit_decorator_metadata.unwrap_or(false);

            transform.jsx = JsxOptions {
                jsx_plugin: co.jsx.is_some_and(|jsx| jsx != TsConfigJsx::Preserve),
                pragma_frag: co.jsx_fragment_factory,
                pragma: co.jsx_factory,
                development: co.jsx.is_some_and(|jsx| jsx.is_dev()),
                runtime: co.jsx.map_or(JsxRuntime::default(), |jsx| match jsx {
                    TsConfigJsx::React => JsxRuntime::Classic,
                    TsConfigJsx::ReactJsx | TsConfigJsx::ReactJsxDev => JsxRuntime::Automatic,
                    _ => JsxRuntime::default(),
                }),
                ..JsxOptions::default()
            };

            if co.isolated_declarations.is_some_and(std::convert::identity) {
                id = Some(IsolatedDeclarationsOptions {
                    strip_internal: co.strip_internal.unwrap_or_default(),
                });
            }

            if let Some(root_dir) = co.root_dir {
                src = Some(package.root_dir.join(root_dir));
            }
            if let Some(out_dir) = co.out_dir {
                dist = Some(package.root_dir.join(out_dir));
            }
        }
        debug_assert!(package.root_dir.is_absolute());
        let src = src.unwrap_or_else(|| package.root_dir.join("src"));
        let dist = dist.unwrap_or_else(|| package.root_dir.join("dist"));
        transform.cwd = package.root_dir;

        Self {
            parse: ParseOptions::default(),
            transform,
            id,
            // minify: MinifierOptions::default(),
            codegen: CodegenOptions::default(),
            src,
            dist,
        }
    }
}

impl CompilerOptions {
    pub fn root_dir(&self) -> &PathBuf {
        &self.transform.cwd
    }
    pub fn src(&self) -> &PathBuf {
        &self.src
    }
    pub fn dist(&self) -> &PathBuf {
        &self.dist
    }
}
