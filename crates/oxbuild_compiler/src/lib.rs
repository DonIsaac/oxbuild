#![allow(dead_code, unused_imports, unused_variables)]
mod options;

use oxc::{
    ast::{ast::Program, Trivias},
    codegen::CodegenReturn,
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
    transformer::{Transformer, TransformerReturn},
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
) -> Result<CompiledOutput, Vec<OxcDiagnostic>> {
    let source_text = fs::read_to_string(source_path).unwrap();
    let source_type = SourceType::from_path(source_path).unwrap();
    let source_name = source_path.as_os_str().to_str().unwrap();
    let allocator = Allocator::default();

    /* ========================== PARSE ========================== */
    let ParserReturn {
        mut program,
        trivias,
        mut errors,
        panicked,
    } = Parser::new(&allocator, &source_text, source_type.clone()).parse();

    if panicked {
        debug_assert!(!errors.is_empty());
        return Err(errors);
    }

    let SemanticBuilderReturn {
        semantic,
        errors: semantic_errors,
    } = SemanticBuilder::new(&source_text)
        .with_trivias(trivias.clone())
        .with_check_syntax_error(true)
        .build(&program);
    errors.extend(semantic_errors);
    if !errors.is_empty() {
        return Err(errors);
    }

    /* ========================== TRANSFORM ========================== */

    let CodegenReturn {
        source_text: id,
        source_map: id_map,
    } = isolated_declarations(
        &allocator,
        &program,
        &source_text,
        source_name,
        trivias.clone(),
    )?;

    let CodegenReturn {
        source_text: output_text,
        source_map,
    } = transform(&allocator, semantic, &mut program, source_path);

    Ok(CompiledOutput {
        source_text: output_text,
        source_map,
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
    } = IsolatedDeclarations::new(allocator).build(program);

    if !errors.is_empty() {
        return Err(errors);
    }

    // let CodegenReturn { source_text, source_map, .. } = Codegen::new()
    let result = Codegen::new()
        .with_source_text(source_text)
        .with_capacity(source_text.len())
        .enable_comment(
            source_text,
            trivias,
            CommentOptions {
                preserve_annotate_comments: false,
                ..Default::default()
            },
        )
        .enable_source_map(source_name, source_text)
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

    let transformer = Transformer::new(
        &allocator,
        &source_path,
        semantic.source_type().clone(),
        source_text,
        trivias.clone(),
        Default::default(),
    );
    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    let TransformerReturn {
        errors,
        symbols,
        scopes,
    } = transformer.build_with_symbols_and_scopes(symbols, scopes, program);

    let codegen = Codegen::new()
        .enable_comment(&source_text, trivias.clone(), Default::default())
        .with_capacity(source_text.len())
        .enable_source_map(source_path.as_os_str().to_str().unwrap(), &source_text)
        .with_mangler(Some(Default::default()));

    codegen.build(program)
}
