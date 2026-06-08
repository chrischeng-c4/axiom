---
id: projects-sdd-src-services-tech-stack-service-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Standardized projects/agentic-workflow/src/services/tech_stack_service.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/tech_stack_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `infer_tech_stack` | projects/agentic-workflow/src/services/tech_stack_service.rs | function | pub | 52 | infer_tech_stack(project_root: &Path) -> TechStack |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/services/tech_stack_service.rs -->
```rust
//! Tech stack inference service.
//!
//! Auto-detects project tech stack from manifest files (Cargo.toml, pyproject.toml, package.json).
//! Result is cached per project root within a workflow run.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::models::tech_stack::{
    DesignSystem, DesignSystemRegistryEntry, Language, TechStack, DESIGN_SYSTEM_REGISTRY,
};

/// Framework detection rules per language.
const RUST_FRAMEWORKS: &[(&str, &str)] = &[
    ("axum", "axum"),
    ("actix-web", "actix"),
    ("rocket", "rocket"),
    ("warp", "warp"),
];

const PYTHON_FRAMEWORKS: &[(&str, &str)] = &[
    ("fastapi", "fastapi"),
    ("django", "django"),
    ("flask", "flask"),
    ("starlette", "starlette"),
];

const JS_FRAMEWORKS: &[(&str, &str)] = &[
    ("react", "react"),
    ("react-dom", "react"),
    ("next", "react"),
    ("vue", "vue"),
    ("nuxt", "vue"),
    ("svelte", "svelte"),
    ("@sveltejs/kit", "svelte"),
    ("@angular/core", "angular"),
];

/// Thread-safe cache for tech stack inference results.
static CACHE: std::sync::LazyLock<Mutex<HashMap<String, TechStack>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Infer the project tech stack from manifest files.
///
/// Reads manifests in priority order: Cargo.toml → pyproject.toml → package.json.
/// Design system detection runs on package.json regardless of primary language.
/// Results are cached per project root.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/tech_stack_service.md#source
pub fn infer_tech_stack(project_root: &Path) -> TechStack {
    let key = project_root.to_string_lossy().to_string();

    // Check cache
    if let Ok(cache) = CACHE.lock() {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let result = detect_tech_stack(project_root);

    // Store in cache
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(key, result.clone());
    }

    result
}

fn detect_tech_stack(project_root: &Path) -> TechStack {
    let mut ts = TechStack::default();

    // Probe manifests in priority order
    let cargo_toml = project_root.join("Cargo.toml");
    let pyproject_toml = project_root.join("pyproject.toml");
    let package_json = project_root.join("package.json");

    if cargo_toml.exists() {
        if let Some((lang, fw)) = parse_cargo_toml(&cargo_toml) {
            ts.language = Some(lang);
            ts.framework = fw;
        }
    } else if pyproject_toml.exists() {
        if let Some((lang, fw)) = parse_pyproject_toml(&pyproject_toml) {
            ts.language = Some(lang);
            ts.framework = fw;
        }
    } else if package_json.exists() {
        if let Some((lang, fw)) = parse_package_json_framework(&package_json) {
            ts.language = Some(lang);
            ts.framework = fw;
        }
    }

    // Design system detection always scans package.json (even for Rust monorepos with JS tooling)
    if package_json.exists() {
        ts.design_system = detect_design_system(&package_json);
    }

    ts
}

fn parse_cargo_toml(path: &Path) -> Option<(Language, Option<String>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let doc: toml::Value = content.parse().ok()?;

    let deps = doc.get("dependencies").and_then(|d| d.as_table())?;
    let framework = RUST_FRAMEWORKS
        .iter()
        .find(|(pkg, _)| deps.contains_key(*pkg))
        .map(|(_, fw)| fw.to_string());

    Some((Language::Rust, framework))
}

fn parse_pyproject_toml(path: &Path) -> Option<(Language, Option<String>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let doc: toml::Value = content.parse().ok()?;

    // Try [project.dependencies] (PEP 621)
    let deps_str = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        // Fallback: [tool.poetry.dependencies]
        .or_else(|| {
            doc.get("tool")
                .and_then(|t| t.get("poetry"))
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_table())
                .map(|t| t.keys().cloned().collect::<Vec<_>>().join(" "))
        })?;

    let framework = PYTHON_FRAMEWORKS
        .iter()
        .find(|(pkg, _)| deps_str.contains(pkg))
        .map(|(_, fw)| fw.to_string());

    Some((Language::Python, framework))
}

fn parse_package_json_framework(path: &Path) -> Option<(Language, Option<String>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&content).ok()?;

    let all_deps = collect_js_deps(&doc);
    let lang = if all_deps.contains_key("typescript") || all_deps.contains_key("ts-node") {
        Language::TypeScript
    } else {
        Language::JavaScript
    };

    let framework = JS_FRAMEWORKS
        .iter()
        .find(|(pkg, _)| all_deps.contains_key(*pkg))
        .map(|(_, fw)| fw.to_string());

    Some((lang, framework))
}

fn detect_design_system(package_json_path: &Path) -> Option<DesignSystem> {
    let content = std::fs::read_to_string(package_json_path).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&content).ok()?;

    let all_deps = collect_js_deps(&doc);

    DESIGN_SYSTEM_REGISTRY
        .iter()
        .find(|entry: &&DesignSystemRegistryEntry| all_deps.contains_key(entry.package))
        .map(|entry| DesignSystem {
            library: entry.library.to_string(),
            provides_tokens: entry.provides_tokens,
            provides_components: entry.provides_components,
        })
}

/// Collect all dependencies + devDependencies from package.json.
fn collect_js_deps(doc: &serde_json::Value) -> HashMap<String, ()> {
    let mut deps = HashMap::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(obj) = doc.get(key).and_then(|v| v.as_object()) {
            for k in obj.keys() {
                deps.insert(k.clone(), ());
            }
        }
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_project(files: &[(&str, &str)]) -> TempDir {
        let tmp = TempDir::new().unwrap();
        for (name, content) in files {
            let path = tmp.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, content).unwrap();
        }
        tmp
    }

    #[test]
    fn test_react_with_mui() {
        let tmp = setup_project(&[(
            "package.json",
            r#"{"dependencies":{"react":"^18","@mui/material":"^5"}}"#,
        )]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.language, Some(Language::JavaScript));
        assert_eq!(ts.framework.as_deref(), Some("react"));
        let ds = ts.design_system.unwrap();
        assert_eq!(ds.library, "mui");
        assert!(ds.provides_tokens);
        assert!(ds.provides_components);
    }

    #[test]
    fn test_react_with_antd() {
        let tmp = setup_project(&[(
            "package.json",
            r#"{"dependencies":{"react":"^18","antd":"^5"}}"#,
        )]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.framework.as_deref(), Some("react"));
        let ds = ts.design_system.unwrap();
        assert_eq!(ds.library, "antd");
        assert!(!ds.provides_tokens);
        assert!(ds.provides_components);
    }

    #[test]
    fn test_react_no_design_system() {
        let tmp = setup_project(&[("package.json", r#"{"dependencies":{"react":"^18"}}"#)]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.framework.as_deref(), Some("react"));
        assert!(ts.design_system.is_none());
    }

    #[test]
    fn test_rust_axum() {
        let tmp = setup_project(&[(
            "Cargo.toml",
            r#"[dependencies]
axum = "0.7"
"#,
        )]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.language, Some(Language::Rust));
        assert_eq!(ts.framework.as_deref(), Some("axum"));
        assert!(ts.design_system.is_none());
    }

    #[test]
    fn test_python_fastapi() {
        let tmp = setup_project(&[(
            "pyproject.toml",
            r#"[project]
dependencies = ["fastapi>=0.100", "uvicorn"]
"#,
        )]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.language, Some(Language::Python));
        assert_eq!(ts.framework.as_deref(), Some("fastapi"));
    }

    #[test]
    fn test_no_manifest() {
        let tmp = TempDir::new().unwrap();
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts, TechStack::default());
    }

    #[test]
    fn test_malformed_package_json() {
        let tmp = setup_project(&[("package.json", "not json{{{")]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts, TechStack::default());
    }

    #[test]
    fn test_design_system_in_devdeps() {
        let tmp = setup_project(&[(
            "package.json",
            r#"{"devDependencies":{"@chakra-ui/react":"^2"}}"#,
        )]);
        let ts = detect_tech_stack(tmp.path());
        let ds = ts.design_system.unwrap();
        assert_eq!(ds.library, "chakra");
    }

    #[test]
    fn test_rust_monorepo_with_js_design_system() {
        let tmp = setup_project(&[
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/*\"]\n[dependencies]\naxum = \"0.7\"\n",
            ),
            ("package.json", r#"{"dependencies":{"@mui/material":"^5"}}"#),
        ]);
        let ts = detect_tech_stack(tmp.path());
        assert_eq!(ts.language, Some(Language::Rust));
        assert_eq!(ts.framework.as_deref(), Some("axum"));
        // Design system still detected from package.json
        let ds = ts.design_system.unwrap();
        assert_eq!(ds.library, "mui");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/tech_stack_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete tech stack inference service module.
```
