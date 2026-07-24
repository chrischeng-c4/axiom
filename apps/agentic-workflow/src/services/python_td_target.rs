//! Deterministic native-Python target emission from [`PythonTdIr`].
//!
//! This target writes only source and unit-test artifacts. External contracts
//! are deliberately excluded: EC remains an independently authored verifier.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-ddd-python-source-generation.md#logic

use super::python_td::{PythonTdDeclarationKind, PythonTdIr};
use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdTarget {
    pub files: Vec<PythonTdTargetFile>,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdTargetFile {
    pub path: String,
    pub digest: String,
}

pub fn emit_python_td_target(ir: &PythonTdIr, output_root: &Path) -> Result<PythonTdTarget> {
    let mut files = BTreeMap::<String, String>::new();
    for module in &ir.modules {
        if !module.path.starts_with("src/") {
            continue;
        }
        let source = render_module(module);
        files.insert(module.path.clone(), source);
        add_package_markers(&mut files, &module.path);
    }
    files.insert("pyproject.toml".to_string(), render_pyproject());
    let unit_path = "tests/unit/test_generated_inventory.py".to_string();
    files.insert(unit_path.clone(), render_unit_test(ir));

    let manifest = files
        .iter()
        .map(|(relative, content)| PythonTdTargetFile {
            path: relative.clone(),
            digest: digest(content.as_bytes()),
        })
        .collect::<Vec<_>>();
    super::python_td::materialize_owned_target(output_root, "python", &files)?;
    let digest = digest(&serde_json::to_vec(&manifest)?);
    Ok(PythonTdTarget {
        files: manifest,
        digest,
    })
}

fn add_package_markers(files: &mut BTreeMap<String, String>, module_path: &str) {
    let source_relative = module_path
        .strip_prefix("src/")
        .expect("caller filters Python TD source modules");
    let mut parts = source_relative.split('/').collect::<Vec<_>>();
    parts.pop();
    for length in 1..=parts.len() {
        let marker = format!("src/{}/__init__.py", parts[..length].join("/"));
        files.entry(marker).or_insert_with(|| {
            format!(
                "# {}\n\"\"\"Generated Python TD package.\"\"\"\n",
                super::python_td::NATIVE_TARGET_OWNER
            )
        });
    }
}

fn render_pyproject() -> String {
    format!(
        "# {}\n[build-system]\nrequires = [\"setuptools>=68\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"generated-python-td-target\"\nversion = \"0.1.0\"\n\n[tool.setuptools.packages.find]\nwhere = [\"src\"]\n",
        super::python_td::NATIVE_TARGET_OWNER
    )
}

fn render_module(module: &super::python_td::PythonTdModule) -> String {
    let mut output = format!(
        "# {}\n\"\"\"Generated from {}.\"\"\"\n\n",
        super::python_td::NATIVE_TARGET_OWNER,
        module.id
    );
    for declaration in &module.declarations {
        match declaration.kind {
            PythonTdDeclarationKind::Class => {
                output.push_str(&format!("class {}:\n    pass\n\n", declaration.name));
            }
            PythonTdDeclarationKind::Function => {
                let prefix = if declaration.is_async { "async " } else { "" };
                output.push_str(&format!(
                    "{prefix}def {}(*args, **kwargs):\n    # HANDWRITE-BEGIN gap=\"python-td-function-body\"\n    raise NotImplementedError(\"generated TD declaration requires a function-body implementation\")\n    # HANDWRITE-END gap=\"python-td-function-body\"\n\n",
                    declaration.name
                ));
            }
        }
    }
    if module.declarations.is_empty() {
        output.push_str("pass\n");
    }
    output
}

fn render_unit_test(ir: &PythonTdIr) -> String {
    let mut output = format!(
        "# {}\nimport importlib\nimport sys\nimport unittest\nfrom pathlib import Path\n\nsys.path.insert(0, str(Path(__file__).parents[2] / 'src'))\n\nclass GeneratedInventoryTest(unittest.TestCase):\n",
        super::python_td::NATIVE_TARGET_OWNER
    );
    let mut count = 0;
    for module in &ir.modules {
        if !module.path.starts_with("src/") || module.declarations.is_empty() {
            continue;
        }
        let name = module
            .path
            .trim_start_matches("src/")
            .trim_end_matches(".py")
            .replace('/', ".");
        output.push_str(&format!(
            "    def test_{}(self):\n        module = importlib.import_module({name:?})\n",
            module.id.replace([':', '.', '/', '-'], "_")
        ));
        for declaration in &module.declarations {
            output.push_str(&format!(
                "        self.assertTrue(hasattr(module, {:?}))\n",
                declaration.name
            ));
        }
        count += 1;
    }
    if count == 0 {
        output.push_str("    def test_empty_inventory(self):\n        self.assertTrue(True)\n");
    }
    output
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
