//! Deterministic typed-IR mutation enumeration for Python TD artifacts.
//!
//! Mutations are applied only to cloned [`PythonTdIr`] values. The sidecar
//! never rewrites authoring source text.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-td-mutation-operators.md#logic

use super::python_td::{PythonTdDeclarationKind, PythonTdIr, PythonTdSpan};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const PYTHON_TD_MUTATION_SCHEMA: &str = "aw.python-td-mutation.v1";

const SUPPORTED_SCOPES: [PythonTdMutationScope; 4] = [
    PythonTdMutationScope::Semantic,
    PythonTdMutationScope::Python,
    PythonTdMutationScope::Rust,
    PythonTdMutationScope::TypeScript,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdMutationScope {
    Semantic,
    Python,
    Rust,
    TypeScript,
}

impl PythonTdMutationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
        }
    }
}

pub fn supported_python_td_mutation_scopes() -> &'static [PythonTdMutationScope] {
    &SUPPORTED_SCOPES
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdMutationOperator {
    RenameDeclaration,
    RemoveDeclaration,
    ToggleAsync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PythonTdMutationDescriptor {
    pub schema_version: String,
    pub id: String,
    pub input_semantic_digest: String,
    pub scope: PythonTdMutationScope,
    pub operator: PythonTdMutationOperator,
    pub module_id: String,
    pub declaration_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdMutant {
    pub descriptor: PythonTdMutationDescriptor,
    pub mutated_semantic_digest: String,
    pub ir: PythonTdIr,
}

pub fn enumerate_python_td_mutation_descriptors(
    ir: &PythonTdIr,
) -> Result<Vec<PythonTdMutationDescriptor>> {
    let mut descriptors = Vec::new();
    for scope in SUPPORTED_SCOPES {
        for module in &ir.modules {
            for declaration in &module.declarations {
                descriptors.push(descriptor(
                    ir,
                    scope,
                    PythonTdMutationOperator::RenameDeclaration,
                    &module.id,
                    &declaration.id,
                )?);
                if module.declarations.len() > 1 {
                    descriptors.push(descriptor(
                        ir,
                        scope,
                        PythonTdMutationOperator::RemoveDeclaration,
                        &module.id,
                        &declaration.id,
                    )?);
                }
                if declaration.kind == PythonTdDeclarationKind::Function {
                    descriptors.push(descriptor(
                        ir,
                        scope,
                        PythonTdMutationOperator::ToggleAsync,
                        &module.id,
                        &declaration.id,
                    )?);
                }
            }
        }
    }

    let unique_ids = descriptors
        .iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<BTreeSet<_>>();
    if unique_ids.len() != descriptors.len() {
        bail!("Python TD mutation enumeration produced duplicate mutant ids");
    }
    Ok(descriptors)
}

pub fn enumerate_python_td_mutants(ir: &PythonTdIr) -> Result<Vec<PythonTdMutant>> {
    enumerate_python_td_mutation_descriptors(ir)?
        .into_iter()
        .map(|descriptor| apply_python_td_mutation(ir, descriptor))
        .collect()
}

pub fn apply_python_td_mutation(
    ir: &PythonTdIr,
    descriptor: PythonTdMutationDescriptor,
) -> Result<PythonTdMutant> {
    if descriptor.schema_version != PYTHON_TD_MUTATION_SCHEMA {
        bail!(
            "unsupported Python TD mutation schema `{}`",
            descriptor.schema_version
        );
    }
    if descriptor.input_semantic_digest != ir.semantic_digest {
        bail!(
            "Python TD mutation `{}` input digest does not match the supplied IR",
            descriptor.id
        );
    }
    let expected = descriptor_id(
        &descriptor.input_semantic_digest,
        descriptor.scope,
        descriptor.operator,
        &descriptor.module_id,
        &descriptor.declaration_id,
    )?;
    if descriptor.id != expected {
        bail!(
            "Python TD mutation descriptor id is stale: expected `{expected}`, got `{}`",
            descriptor.id
        );
    }

    let mut mutated = ir.clone();
    let module = mutated
        .modules
        .iter_mut()
        .find(|module| module.id == descriptor.module_id)
        .with_context(|| {
            format!(
                "Python TD mutation `{}` references missing module `{}`",
                descriptor.id, descriptor.module_id
            )
        })?;
    let declaration_index = module
        .declarations
        .iter()
        .position(|declaration| declaration.id == descriptor.declaration_id)
        .with_context(|| {
            format!(
                "Python TD mutation `{}` references missing declaration `{}`",
                descriptor.id, descriptor.declaration_id
            )
        })?;
    match descriptor.operator {
        PythonTdMutationOperator::RenameDeclaration => {
            let declaration = &mut module.declarations[declaration_index];
            let original_name = declaration.name.clone();
            declaration.name.push_str("__mutated");
            declaration.id.push_str("__mutated");
            if let Some(super::python_td::PythonTdCodegen::OpenApi(openapi)) =
                module.codegen.as_mut()
            {
                if openapi.client_name == original_name {
                    openapi.client_name = declaration.name.clone();
                }
            }
        }
        PythonTdMutationOperator::RemoveDeclaration => {
            let removed = module.declarations.remove(declaration_index);
            if module
                .codegen
                .as_ref()
                .is_some_and(|codegen| match codegen {
                    super::python_td::PythonTdCodegen::OpenApi(openapi) => {
                        openapi.client_name == removed.name
                    }
                })
            {
                module.codegen = None;
            }
        }
        PythonTdMutationOperator::ToggleAsync => {
            let declaration = &mut module.declarations[declaration_index];
            if declaration.kind != PythonTdDeclarationKind::Function {
                bail!(
                    "Python TD mutation `{}` cannot toggle async on a non-function declaration",
                    descriptor.id
                );
            }
            declaration.is_async = !declaration.is_async;
        }
    }
    mutated.semantic_digest = canonical_mutated_digest(&mutated)?;
    if mutated.semantic_digest == ir.semantic_digest {
        bail!(
            "Python TD mutation `{}` did not change the semantic digest",
            descriptor.id
        );
    }
    Ok(PythonTdMutant {
        descriptor,
        mutated_semantic_digest: mutated.semantic_digest.clone(),
        ir: mutated,
    })
}

fn descriptor(
    ir: &PythonTdIr,
    scope: PythonTdMutationScope,
    operator: PythonTdMutationOperator,
    module_id: &str,
    declaration_id: &str,
) -> Result<PythonTdMutationDescriptor> {
    Ok(PythonTdMutationDescriptor {
        schema_version: PYTHON_TD_MUTATION_SCHEMA.to_string(),
        id: descriptor_id(
            &ir.semantic_digest,
            scope,
            operator,
            module_id,
            declaration_id,
        )?,
        input_semantic_digest: ir.semantic_digest.clone(),
        scope,
        operator,
        module_id: module_id.to_string(),
        declaration_id: declaration_id.to_string(),
    })
}

fn descriptor_id(
    input_semantic_digest: &str,
    scope: PythonTdMutationScope,
    operator: PythonTdMutationOperator,
    module_id: &str,
    declaration_id: &str,
) -> Result<String> {
    let canonical = serde_json::to_vec(&(
        PYTHON_TD_MUTATION_SCHEMA,
        input_semantic_digest,
        scope,
        operator,
        module_id,
        declaration_id,
    ))
    .context("serialize Python TD mutation descriptor")?;
    Ok(format!(
        "mutant:{}:sha256:{:x}",
        scope.as_str(),
        Sha256::digest(canonical)
    ))
}

fn canonical_mutated_digest(ir: &PythonTdIr) -> Result<String> {
    let mut modules = ir.modules.clone();
    for module in &mut modules {
        if module.artifact_id.is_some() {
            module.path.clear();
        }
        for declaration in &mut module.declarations {
            declaration.span = PythonTdSpan {
                line: 0,
                column: 0,
                byte_start: 0,
                byte_end: 0,
            };
        }
    }
    let canonical =
        serde_json::to_vec(&modules).context("serialize mutated canonical Python TD IR")?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
}
