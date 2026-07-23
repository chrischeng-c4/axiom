// HANDWRITE-BEGIN gap="missing-generator:python-ir-typescript-target" tracker="#2302" reason="TypeScript lowering validates the reviewed IR before output exists."
//! Deterministic TypeScript target emission from the target-neutral Python TD IR.
//!
//! It emits only product source and target-native unit tests. External-contract
//! source stays outside this output because EC remains independently authored.

use super::python_td::{PythonTdDeclarationKind, PythonTdIr, PythonTdModule, PythonTdRole};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TypeScriptTdTarget {
    pub files: Vec<TypeScriptTdTargetFile>,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TypeScriptTdTargetFile {
    pub path: String,
    pub digest: String,
}

pub fn emit_python_td_typescript_target(
    ir: &PythonTdIr,
    output_root: &Path,
) -> Result<TypeScriptTdTarget> {
    let mut files = BTreeMap::new();
    let mut roles = BTreeSet::new();
    for module in ir
        .modules
        .iter()
        .filter(|module| module.path.starts_with("src/"))
    {
        let name = module_name(module)?;
        let role = ddd_role(&module.role);
        let path = format!("src/{role}/{name}.ts");
        if files.insert(path.clone(), render_module(module)?).is_some() {
            bail!("unsupported duplicate TypeScript module path `{path}`");
        }
        files
            .entry(format!("src/{role}/index.ts"))
            .or_insert_with(String::new)
            .push_str(&format!("export * from './{name}.js';\n"));
        roles.insert(role);
    }
    if files.is_empty() {
        bail!("Python TD IR has no src/* modules to lower to TypeScript");
    }
    files.insert(
        "src/index.ts".into(),
        roles
            .into_iter()
            .map(|role| format!("export * from './{role}/index.js';\n"))
            .collect(),
    );
    files.insert("package.json".into(), package_json());
    files.insert("tsconfig.json".into(), tsconfig_json());
    files.insert(
        "tests/generated_inventory.test.mjs".into(),
        inventory_test(),
    );

    let manifest = manifest(&files);
    // Rendering and validation finish before a target path is created, so an
    // unsupported declaration cannot leave a partially applied package.
    for (path, content) in &files {
        let output = output_root.join(path);
        fs::create_dir_all(output.parent().expect("generated file has parent"))
            .with_context(|| format!("create {}", output.display()))?;
        fs::write(&output, content).with_context(|| format!("write {}", output.display()))?;
    }
    Ok(TypeScriptTdTarget {
        digest: digest(&serde_json::to_vec(&manifest)?),
        files: manifest,
    })
}

fn module_name(module: &PythonTdModule) -> Result<&str> {
    let name = module
        .path
        .trim_end_matches(".py")
        .rsplit('/')
        .next()
        .unwrap_or_default();
    if !identifier(name) || module.declarations.is_empty() {
        bail!(
            "unsupported Python TD module `{}` for TypeScript target",
            module.id
        );
    }
    Ok(name)
}

fn ddd_role(role: &PythonTdRole) -> &'static str {
    match role {
        PythonTdRole::Domain => "domain",
        PythonTdRole::Application => "application",
        PythonTdRole::Infrastructure => "infrastructure",
        PythonTdRole::Interface => "interface",
        _ => "support",
    }
}

fn render_module(module: &PythonTdModule) -> Result<String> {
    let mut output = String::from("// CODEGEN-BEGIN python-ir-typescript-target\n");
    output.push_str("interface GeneratedModuleContract {}\n");
    output.push_str("class GeneratedError extends Error {}\n");
    for declaration in &module.declarations {
        if !identifier(&declaration.name) {
            bail!(
                "unsupported Python TD declaration `{}` in `{}` for TypeScript target",
                declaration.name,
                module.id
            );
        }
        if !declaration.annotations.is_empty() || !declaration.decorators.is_empty() {
            output.push_str(&format!(
                "// python metadata: {:?} {:?}\n",
                declaration.annotations, declaration.decorators
            ));
        }
        match &declaration.kind {
            PythonTdDeclarationKind::Class => output.push_str(&format!(
                "export class {} implements GeneratedModuleContract {{}}\n",
                declaration.name
            )),
            PythonTdDeclarationKind::Function if declaration.is_async => output.push_str(&format!(
                "export async function {}(): Promise<never> {{ throw new GeneratedError('generated TD declaration requires implementation'); }}\n",
                declaration.name
            )),
            PythonTdDeclarationKind::Function => output.push_str(&format!(
                "export function {}(): never {{ throw new GeneratedError('generated TD declaration requires implementation'); }}\n",
                declaration.name
            )),
        }
    }
    output.push_str("// CODEGEN-END python-ir-typescript-target\n");
    Ok(output)
}

fn package_json() -> String {
    "{\n  \"name\": \"generated-python-td-typescript-target\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {\n    \"test\": \"node --test tests/generated_inventory.test.mjs\",\n    \"typecheck\": \"tsc --noEmit\"\n  },\n  \"devDependencies\": {\n    \"typescript\": \"^5.0.0\"\n  }\n}\n".into()
}

fn tsconfig_json() -> String {
    "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"NodeNext\",\n    \"moduleResolution\": \"NodeNext\",\n    \"strict\": true,\n    \"noEmit\": true\n  },\n  \"include\": [\"src/**/*.ts\"]\n}\n".into()
}

fn inventory_test() -> String {
    "import assert from 'node:assert/strict';\nimport test from 'node:test';\nimport { existsSync } from 'node:fs';\n\ntest('generated TypeScript source inventory is present', () => {\n  assert.equal(existsSync(new URL('../src/index.ts', import.meta.url)), true);\n});\n".into()
}

fn manifest(files: &BTreeMap<String, String>) -> Vec<TypeScriptTdTargetFile> {
    files
        .iter()
        .map(|(path, content)| TypeScriptTdTargetFile {
            path: path.clone(),
            digest: digest(content.as_bytes()),
        })
        .collect()
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphabetic() || byte == b'_' || (index > 0 && byte.is_ascii_digit())
        })
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
// HANDWRITE-END
