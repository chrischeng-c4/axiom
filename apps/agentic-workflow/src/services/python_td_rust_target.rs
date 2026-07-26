// HANDWRITE-BEGIN gap="missing-generator:python-ir-rust-target" tracker="#2301" reason="Rust lowering validates the reviewed IR before output exists."
use super::python_td::{PythonTdDeclarationKind, PythonTdIr};
use anyhow::{bail, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RustTdTarget {
    pub files: Vec<RustTdTargetFile>,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RustTdTargetFile {
    pub path: String,
    pub digest: String,
}

pub fn emit_python_td_rust_target(ir: &PythonTdIr, root: &Path) -> Result<RustTdTarget> {
    let mut files = BTreeMap::new();
    let mut roles = BTreeSet::new();
    let mut openapi_requirements = None;
    for module in ir.modules.iter().filter(|m| m.path.starts_with("src/")) {
        let name = module
            .path
            .trim_end_matches(".py")
            .rsplit('/')
            .next()
            .unwrap_or_default();
        // Python package markers carry import topology only. They have no
        // target-native declaration, so emitting an empty Rust module for
        // them would add noise without preserving any TD contract.
        if module.declarations.is_empty() && name == "__init__" {
            continue;
        }
        if !ident(name) || module.declarations.is_empty() {
            bail!(
                "unsupported Python TD module `{}` for Rust target",
                module.id
            );
        }
        let role = match &module.role {
            super::python_td::PythonTdRole::Domain => "domain",
            super::python_td::PythonTdRole::Application => "application",
            super::python_td::PythonTdRole::Infrastructure => "infrastructure",
            super::python_td::PythonTdRole::Interface => "interface",
            _ => "support",
        };
        if let Some(generated) = super::python_td_openapi_target::generate_openapi_target(
            module,
            openapi_codegen::Lang::Rust,
        )? {
            if openapi_requirements
                .replace(generated.requirements)
                .is_some_and(|existing| existing != generated.requirements)
            {
                bail!("Python TD OpenAPI modules must select one Rust target profile");
            }
            let directory = generated.directory;
            if !ident(&directory) {
                bail!(
                    "unsupported Python TD OpenAPI module directory `{directory}` for Rust target"
                );
            }
            for (relative, content) in generated.files {
                let path = format!("src/{role}/{directory}/{relative}");
                if files.insert(path.clone(), content).is_some() {
                    bail!("unsupported duplicate Rust OpenAPI path `{path}`");
                }
            }
            files
                .entry(format!("src/{role}/mod.rs"))
                .or_insert_with(|| format!("// {}\n", super::python_td::NATIVE_TARGET_OWNER))
                .push_str(&format!("pub mod {directory};\n"));
            roles.insert(role);
            continue;
        }
        let mut body = format!(
            "// {}\n// CODEGEN-BEGIN python-ir-rust-target\n",
            super::python_td::NATIVE_TARGET_OWNER
        );
        body.push_str("pub trait GeneratedModuleContract {}\n");
        for d in &module.declarations {
            if !ident(&d.name) {
                bail!(
                    "unsupported Python TD declaration `{}` in `{}` for Rust target",
                    d.name,
                    module.id
                );
            }
            // Framework decorators and Python type annotations remain stable TD
            // metadata until a later runtime-specific lowering owns them.
            if !d.annotations.is_empty() || !d.decorators.is_empty() {
                body.push_str(&format!(
                    "// python metadata: {:?} {:?}\n",
                    d.annotations, d.decorators
                ));
            }
            match &d.kind {
                PythonTdDeclarationKind::Class => {
                    body.push_str(&format!("#[derive(Default)] pub struct {};\n", d.name));
                    body.push_str(&format!(
                        "impl GeneratedModuleContract for {} {{}}\n",
                        d.name
                    ));
                }
                PythonTdDeclarationKind::Function if d.is_async => body.push_str(&format!(
                    "pub async fn {}() -> Result<(), GeneratedError> {{ Err(GeneratedError) }}\n",
                    d.name
                )),
                PythonTdDeclarationKind::Function => body.push_str(&format!(
                    "pub fn {}() -> Result<(), GeneratedError> {{ Err(GeneratedError) }}\n",
                    d.name
                )),
            }
        }
        body.push_str(
            "#[derive(Debug)] pub struct GeneratedError;\n// CODEGEN-END python-ir-rust-target\n",
        );
        let module_path = format!("src/{role}/{name}.rs");
        if files.insert(module_path.clone(), body).is_some() {
            bail!("unsupported duplicate Rust module path `{module_path}`");
        }
        files
            .entry(format!("src/{role}/mod.rs"))
            .or_insert_with(|| format!("// {}\n", super::python_td::NATIVE_TARGET_OWNER))
            .push_str(&format!("pub mod {name};\n"));
        roles.insert(role);
    }
    if files.is_empty() {
        bail!("Python TD IR has no src/* modules to lower to Rust");
    }
    let lib = format!(
        "// {}\n{}",
        super::python_td::NATIVE_TARGET_OWNER,
        roles
            .into_iter()
            .map(|role| format!("pub mod {role};\n"))
            .collect::<String>()
    );
    files.insert("src/lib.rs".into(), lib);
    files.insert(
        "tests/generated_inventory.rs".into(),
        format!(
            "// {}\n#[test]\nfn generated_source_inventory_is_present() {{\n    assert!(std::path::Path::new(\"src/lib.rs\").is_file());\n}}\n",
            super::python_td::NATIVE_TARGET_OWNER
        ),
    );
    files.insert("Cargo.toml".into(), render_cargo_toml(openapi_requirements));
    let manifest = files
        .iter()
        .map(|(p, c)| RustTdTargetFile {
            path: p.clone(),
            digest: digest(c.as_bytes()),
        })
        .collect::<Vec<_>>();
    super::python_td::materialize_owned_target(root, "rust", &files)?;
    Ok(RustTdTarget {
        digest: digest(&serde_json::to_vec(&manifest)?),
        files: manifest,
    })
}
fn render_cargo_toml(openapi_requirements: Option<openapi_codegen::TargetRequirements>) -> String {
    let edition = openapi_requirements
        .map(|requirements| requirements.language_standard)
        .unwrap_or("2021");
    let mut output = format!(
        "# {}\n[package]\nname=\"generated-python-td-rust-target\"\nversion=\"0.1.0\"\nedition=\"{edition}\"\n",
        super::python_td::NATIVE_TARGET_OWNER
    );
    if let Some(requirements) = openapi_requirements {
        output.push_str("\n[dependencies]\n");
        for dependency in requirements.runtime_dependencies {
            let declaration = match *dependency {
                "reqwest" => {
                    "reqwest = { version = \"0.12\", features = [\"blocking\", \"json\"] }"
                }
                "serde" => "serde = { version = \"1\", features = [\"derive\"] }",
                "serde_json" => "serde_json = \"1\"",
                dependency => dependency,
            };
            output.push_str(declaration);
            output.push('\n');
        }
    }
    output
}
fn ident(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
// HANDWRITE-END
