#![allow(dead_code, unused_imports, unused_variables)]
mod options;

use oxc::{
    ast::{ast::Program, Trivias},
    codegen::CodegenReturn,
    isolated_declarations::IsolatedDeclarationsOptions,
    transformer::{ES2015Options, JsxOptions},
};
use std::{fs, path::Path};

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CommentOptions},
    diagnostics::OxcDiagnostic,
    isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsReturn},
    minifier::Minifier,
    parser::{Parser, ParserReturn},
    semantic::{Semantic, SemanticBuilder, SemanticBuilderReturn},
    sourcemap::SourceMap,
    span::SourceType,
    transformer::{TransformOptions, Transformer, TransformerReturn},
};

pub use options::CompileOptions;

static_assertions::assert_impl_all!(CompileOptions: Send, Sync);

#[derive(Debug, Clone)]
pub struct CompiledOutput {
    pub source_text: String,
    pub source_map: Option<SourceMap>,
    pub declarations: String,
    pub declarations_map: Option<SourceMap>,
}

pub fn compile(
    options: &CompileOptions,
    source_path: &Path,
    source_text: &str,
) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
    // is this js? ts? tsx?
    let source_type = SourceType::from_path(source_path).unwrap();
    // get the name as a pretty string
    let source_name = source_path.as_os_str().to_str().unwrap();
    // needed by oxc to allocate memory.
    let allocator = Allocator::default();

    /* ========================== PARSE ========================== */
    let ParserReturn {
        mut program,
        trivias,
        mut errors,
        panicked,
    } = Parser::new(&allocator, source_text, source_type).parse();

    if panicked {
        debug_assert!(!errors.is_empty());
        return Err(errors);
    }

    let SemanticBuilderReturn {
        semantic,
        errors: semantic_errors,
    } = SemanticBuilder::new(source_text)
        .with_trivias(trivias.clone())
        .with_check_syntax_error(true)
        .build(&program);
    errors.extend(semantic_errors);
    if !errors.is_empty() {
        return Err(errors);
    }

    /* ========================== TRANSFORM ========================== */

    // produce .d.ts files
    let CodegenReturn {
        code: id,
        map: id_map,
    } = isolated_declarations(
        &allocator,
        &program,
        source_text,
        source_name,
        trivias.clone(),
    )?;

    let CodegenReturn {
        code: output_text,
        map: source_map,
    } = transform(&allocator, semantic, &mut program, source_path);

    Ok(CompiledOutput {
        source_text: output_text,
        source_map,
        // declarations: String::new(),
        // declarations_map: None,
        declarations: id,
        declarations_map: id_map,
    })
}

fn isolated_declarations<'a>(
    allocator: &'a Allocator,
    program: &Program<'a>,
    source_text: &'a str,
    source_name: &'a str,
    trivias: Trivias,
) -> Result<CodegenReturn, Vec<OxcDiagnostic>> {
    let IsolatedDeclarationsReturn {
        program, errors, ..
    } = IsolatedDeclarations::new(
        allocator,
        source_text,
        &trivias,
        // TODO: get from tsconfig.json
        IsolatedDeclarationsOptions {
            strip_internal: false,
        },
    )
    .build(program);

    if !errors.is_empty() {
        return Err(errors);
    }

    let result = Codegen::new()
        .with_source_text(source_text)
        .with_capacity(source_text.len())
        .enable_comment(
            source_text,
            trivias,
            CommentOptions {
                preserve_annotate_comments: false,
            },
        )
        // .enable_source_map(source_name, source_text)
        .build(&program);

    Ok(result)
}

fn transform<'a>(
    allocator: &'a Allocator,
    semantic: Semantic<'a>,
    program: &mut Program<'a>,
    source_path: &Path,
) -> CodegenReturn {
    let trivias = semantic.trivias().clone();
    let source_text = semantic.source_text();

    let options = TransformOptions {
        react: JsxOptions {
            jsx_plugin: true,
            display_name_plugin: true,
            jsx_source_plugin: true,
            development: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let transformer = Transformer::new(
        allocator,
        source_path,
        // *semantic.source_type(),
        source_text,
        trivias.clone(),
        options,
    );
    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    let TransformerReturn {
        errors,
        symbols,
        scopes,
    } = transformer.build_with_symbols_and_scopes(symbols, scopes, program);

    let codegen = Codegen::new()
        .enable_comment(source_text, trivias.clone(), Default::default())
        .with_capacity(source_text.len())
        .enable_source_map(source_path.as_os_str().to_str().unwrap(), source_text);
    //.with_mangler(Some(Default::default()));

    codegen.build(program)
}
