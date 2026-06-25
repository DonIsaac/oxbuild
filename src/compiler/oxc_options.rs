use oxc::{
    codegen::CodegenOptions,
    isolated_declarations::IsolatedDeclarationsOptions,
    transformer::{JsxOptions, TransformOptions},
};

use crate::options::DeclarationsOptions;

use super::CompileOptions;

pub(super) fn to_transform_options(options: &CompileOptions) -> TransformOptions {
    TransformOptions {
        cwd: options.root_dir().to_path_buf(),
        jsx: JsxOptions {
            jsx_plugin: true,
            display_name_plugin: true,
            jsx_source_plugin: true,
            development: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(super) fn to_codegen_options() -> CodegenOptions {
    CodegenOptions::default()
}

pub(super) fn to_isolated_declarations_options(
    options: &DeclarationsOptions,
) -> IsolatedDeclarationsOptions {
    IsolatedDeclarationsOptions {
        strip_internal: options.strip_internal,
    }
}
