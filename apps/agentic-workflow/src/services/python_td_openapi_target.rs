//! Shared OpenAPI generation adapter for Python-authored TD native targets.
//!
//! The Python TD compiler statically captures one explicit target profile per
//! language. This adapter invokes the shared pure generator and returns an
//! owned in-memory write set; target emitters still perform the one atomic
//! materialization step.

use super::python_td::{PythonTdCodegen, PythonTdModule, NATIVE_TARGET_OWNER};
use anyhow::{Context, Result};
use openapi_codegen::{
    generate_for_target, GenOptions, HttpClient, Lang, TargetProfile, TargetRequirements,
    MANIFEST_FILE,
};
use std::path::PathBuf;

#[derive(Debug)]
pub struct GeneratedOpenApiTarget {
    pub directory: String,
    pub files: Vec<(String, String)>,
    pub requirements: TargetRequirements,
}

pub fn generate_openapi_target(
    module: &PythonTdModule,
    lang: Lang,
) -> Result<Option<GeneratedOpenApiTarget>> {
    let Some(PythonTdCodegen::OpenApi(contract)) = module.codegen.as_ref() else {
        return Ok(None);
    };
    let target_id = match lang {
        Lang::Py => &contract.python_target,
        Lang::Ts => &contract.typescript_target,
        Lang::Rust => &contract.rust_target,
    };
    let target = TargetProfile::from_id(target_id)
        .with_context(|| format!("parse OpenAPI target `{target_id}` for {}", module.id))?;
    let options = GenOptions {
        lang,
        target: Some(target),
        spec_path: PathBuf::from(&contract.document_path),
        out_dir: PathBuf::new(),
        client_name: contract.client_name.clone(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: false,
    };
    let output = generate_for_target(&contract.document, &options, target)
        .with_context(|| format!("generate OpenAPI target `{target_id}` for {}", module.id))?;
    let mut manifest = serde_json::to_value(
        output
            .manifest()
            .expect("explicit target generation always has a manifest"),
    )?;
    let mut files = output
        .files
        .into_iter()
        .map(|file| {
            let comment = match lang {
                Lang::Py => "#",
                Lang::Ts | Lang::Rust => "//",
            };
            (
                file.rel_path,
                format!("{comment} {NATIVE_TARGET_OWNER}\n{}", file.contents),
            )
        })
        .collect::<Vec<_>>();
    manifest
        .as_object_mut()
        .expect("generation manifest serializes as an object")
        .insert(
            "x-aw-codegen-owner".to_string(),
            serde_json::Value::String(NATIVE_TARGET_OWNER.to_string()),
        );
    files.push((
        MANIFEST_FILE.to_string(),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    ));
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let stem = module
        .path
        .trim_end_matches(".py")
        .rsplit('/')
        .next()
        .unwrap_or("openapi");
    Ok(Some(GeneratedOpenApiTarget {
        directory: format!("{stem}_openapi"),
        files,
        requirements: target.requirements(),
    }))
}
