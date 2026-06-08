// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub mod alias;
pub mod package;

/// Module resolver implementing Node.js resolution algorithm
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
pub struct ModuleResolver {
    options: ResolveOptions,
}

/// Module resolution options
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Base directories to search for modules
    pub base_dirs: Vec<PathBuf>,

    /// Extensions to try when resolving
    pub extensions: Vec<String>,

    /// Whether to resolve index files
    pub resolve_index: bool,

    /// Alias mappings (e.g., "@" -> "src")
    pub alias: Vec<(String, PathBuf)>,

    /// External modules that should not be bundled
    pub externals: HashSet<String>,

    /// When true, treat ALL bare package specifiers (not starting with ./ or ../)
    /// as external. Used for lib builds where node_modules deps should not be bundled.
    pub externalize_all_packages: bool,

    /// Ordered export conditions applied when resolving `exports` fields in
    /// package.json.  The resolver iterates export-object keys in their JSON
    /// insertion order and accepts the first key that is a member of this slice.
    ///
    /// Default: `["import", "browser", "default"]` (browser ESM dev mode).
    /// Override via `jet.config.toml` `[resolve] conditions` for build mode.
    ///
    // @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R1
    // @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
    pub conditions: Vec<String>,
}

/// Resolved module information
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Full path to the module
    pub path: PathBuf,

    /// Module type
    pub kind: ResolveKind,

    /// Whether this is an external module
    pub is_external: bool,
}

/// Module resolution kind
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveKind {
    /// Relative import (./foo, ../bar)
    Relative,

    /// Absolute import (/foo/bar)
    Absolute,

    /// Package import (react, lodash)
    Package,

    /// Alias import (@/components)
    Alias,
}

/// Parse package specifier into package name and subpath
fn parse_package_specifier(specifier: &str) -> (String, Option<String>) {
    if specifier.starts_with('@') {
        let parts: Vec<&str> = specifier.splitn(3, '/').collect();
        match parts.len() {
            2 => (specifier.to_string(), None),
            3 => {
                let package_name = format!("{}/{}", parts[0], parts[1]);
                let subpath = format!("./{}", parts[2]);
                (package_name, Some(subpath))
            }
            _ => (specifier.to_string(), None),
        }
    } else {
        match specifier.split_once('/') {
            Some((pkg, rest)) => (pkg.to_string(), Some(format!("./{}", rest))),
            None => (specifier.to_string(), None),
        }
    }
}

fn is_singleton_package(package_name: &str) -> bool {
    matches!(package_name, "react" | "react-dom")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
impl ModuleResolver {
    /// Create a new module resolver
    pub fn new(options: ResolveOptions) -> Result<Self> {
        Ok(Self { options })
    }

    /// Resolve a module specifier
    pub fn resolve(&self, specifier: &str, from: &Path) -> Result<ResolvedModule> {
        tracing::debug!("Resolving '{}' from {:?}", specifier, from);

        if self.is_external(specifier) {
            return Ok(ResolvedModule {
                path: PathBuf::from(specifier),
                kind: ResolveKind::Package,
                is_external: true,
            });
        }

        let kind = self.detect_kind(specifier);

        let path = match kind {
            ResolveKind::Relative => self.resolve_relative(specifier, from)?,
            ResolveKind::Absolute => self.resolve_absolute(specifier)?,
            ResolveKind::Package => self.resolve_package(specifier, from)?,
            ResolveKind::Alias => self.resolve_alias(specifier, from)?,
        };

        Ok(ResolvedModule {
            path,
            kind,
            is_external: false,
        })
    }

    fn detect_kind(&self, specifier: &str) -> ResolveKind {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            ResolveKind::Relative
        } else if specifier.starts_with('/') {
            ResolveKind::Absolute
        } else if self.is_alias(specifier) {
            ResolveKind::Alias
        } else {
            ResolveKind::Package
        }
    }

    fn is_alias(&self, specifier: &str) -> bool {
        self.options
            .alias
            .iter()
            .any(|(prefix, _)| specifier.starts_with(prefix))
    }

    fn is_external(&self, specifier: &str) -> bool {
        // When externalize_all_packages is set, treat all bare specifiers as external.
        // Bare specifiers don't start with './', '../', or '/'.
        if self.options.externalize_all_packages
            && !specifier.starts_with('.')
            && !specifier.starts_with('/')
        {
            return true;
        }

        self.options.externals.contains(specifier)
            || self
                .options
                .externals
                .iter()
                .any(|ext| specifier.starts_with(&format!("{}/", ext)))
    }

    fn resolve_relative(&self, specifier: &str, from: &Path) -> Result<PathBuf> {
        let base_dir = from.parent().unwrap_or(Path::new("."));
        let candidate = base_dir.join(specifier);
        self.try_extensions(&candidate)
    }

    fn resolve_absolute(&self, specifier: &str) -> Result<PathBuf> {
        let candidate = PathBuf::from(specifier);
        self.try_extensions(&candidate)
    }

    fn resolve_package(&self, specifier: &str, from: &Path) -> Result<PathBuf> {
        let (package_name, subpath) = parse_package_specifier(specifier);

        let mut current = from.parent();
        let mut hoisted_singleton = None;

        while let Some(dir) = current {
            let node_modules = dir.join("node_modules");
            if node_modules.exists() {
                let package_dir = node_modules.join(&package_name);
                if package_dir.exists() {
                    if let Ok(resolved) = self.resolve_package_dir(&package_dir, subpath.as_deref())
                    {
                        if is_singleton_package(&package_name) {
                            hoisted_singleton = Some(resolved);
                            current = dir.parent();
                            continue;
                        }
                        return Ok(resolved);
                    }
                }
            }
            current = dir.parent();
        }

        if let Some(resolved) = hoisted_singleton {
            return Ok(resolved);
        }

        anyhow::bail!("Cannot resolve package: {}", specifier)
    }

    fn resolve_package_dir(&self, package_dir: &Path, subpath: Option<&str>) -> Result<PathBuf> {
        let package_json = package_dir.join("package.json");

        if package_json.exists() {
            let cond_refs: Vec<&str> = self.options.conditions.iter().map(|s| s.as_str()).collect();
            if let Ok(Some(export_path)) =
                package::resolve_exports(&package_json, subpath, &cond_refs)
            {
                let resolved_path =
                    package_dir.join(export_path.trim_start_matches('.').trim_start_matches('/'));
                if let Ok(resolved) = self.try_extensions(&resolved_path) {
                    return Ok(resolved);
                }
                if resolved_path.exists() {
                    return Ok(resolved_path);
                }
            }
        }

        if let Some(sub) = subpath {
            let subpath_resolved =
                package_dir.join(sub.trim_start_matches('.').trim_start_matches('/'));
            if let Ok(resolved) = self.try_extensions(&subpath_resolved) {
                return Ok(resolved);
            }
        }

        if subpath.is_none() || subpath == Some(".") {
            if package_json.exists() {
                if let Ok(main) = package::get_package_main(&package_json) {
                    let main_path = package_dir.join(main);
                    if let Ok(resolved) = self.try_extensions(&main_path) {
                        return Ok(resolved);
                    }
                }
            }

            if self.options.resolve_index {
                let index = package_dir.join("index");
                if let Ok(resolved) = self.try_extensions(&index) {
                    return Ok(resolved);
                }
            }
        }

        anyhow::bail!(
            "Cannot resolve package directory: {:?} with subpath: {:?}",
            package_dir,
            subpath
        )
    }

    fn resolve_alias(&self, specifier: &str, _from: &Path) -> Result<PathBuf> {
        for (prefix, target) in &self.options.alias {
            if specifier.starts_with(prefix) {
                let rest = &specifier[prefix.len()..];
                let candidate = target.join(rest.trim_start_matches('/'));
                return self.try_extensions(&candidate);
            }
        }

        anyhow::bail!("No matching alias for: {}", specifier)
    }

    fn try_extensions(&self, base: &Path) -> Result<PathBuf> {
        if base.exists() && base.is_file() {
            return Ok(base.to_path_buf());
        }

        for ext in &self.options.extensions {
            let with_ext = base.with_extension(ext.trim_start_matches('.'));
            if with_ext.exists() && with_ext.is_file() {
                return Ok(with_ext);
            }
        }

        if base.is_dir() {
            let package_json = base.join("package.json");
            if package_json.exists() {
                if let Ok(main) = package::get_package_main(&package_json) {
                    let main_path = base.join(main);
                    if let Ok(resolved) = self.try_extensions(&main_path) {
                        return Ok(resolved);
                    }
                }
            }

            if self.options.resolve_index {
                for ext in &self.options.extensions {
                    let index = base.join(format!("index.{}", ext.trim_start_matches('.')));
                    if index.exists() && index.is_file() {
                        return Ok(index);
                    }
                }
            }
        }

        anyhow::bail!("Cannot resolve: {:?}", base)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            base_dirs: vec![PathBuf::from(".")],
            extensions: vec![
                "js".to_string(),
                "jsx".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "json".to_string(),
            ],
            resolve_index: true,
            alias: Vec::new(),
            externals: HashSet::new(),
            externalize_all_packages: false,
            conditions: vec![
                "import".to_string(),
                "browser".to_string(),
                "default".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_kind() {
        let resolver = ModuleResolver::new(ResolveOptions::default()).unwrap();

        assert_eq!(resolver.detect_kind("./foo"), ResolveKind::Relative);
        assert_eq!(resolver.detect_kind("../bar"), ResolveKind::Relative);
        assert_eq!(resolver.detect_kind("/abs/path"), ResolveKind::Absolute);
        assert_eq!(resolver.detect_kind("react"), ResolveKind::Package);
    }

    #[test]
    fn test_is_external() {
        let mut options = ResolveOptions::default();
        options.externals.insert("react".to_string());
        options.externals.insert("react-dom".to_string());

        let resolver = ModuleResolver::new(options).unwrap();

        assert!(resolver.is_external("react"));
        assert!(resolver.is_external("react-dom"));
        assert!(resolver.is_external("react-dom/client"));
        assert!(!resolver.is_external("./foo"));
    }

    #[test]
    fn test_parse_package_specifier() {
        assert_eq!(
            parse_package_specifier("react"),
            ("react".to_string(), None)
        );
        assert_eq!(
            parse_package_specifier("react/jsx-runtime"),
            ("react".to_string(), Some("./jsx-runtime".to_string()))
        );
        assert_eq!(
            parse_package_specifier("@babel/core"),
            ("@babel/core".to_string(), None)
        );
        assert_eq!(
            parse_package_specifier("@babel/core/lib/config"),
            ("@babel/core".to_string(), Some("./lib/config".to_string()))
        );
    }

    #[test]
    fn test_package_subpath_directory_uses_nested_package_json_main() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pkg = tmp
            .path()
            .join("node_modules")
            .join("dom-helpers")
            .join("addClass");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::create_dir_all(pkg.parent().unwrap().join("esm")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{
              "name": "dom-helpers/addClass",
              "private": true,
              "main": "../esm/addClass.js"
            }"#,
        )
        .unwrap();
        std::fs::write(pkg.parent().unwrap().join("esm").join("addClass.js"), "").unwrap();
        let from = tmp.path().join("src").join("main.js");
        std::fs::create_dir_all(from.parent().unwrap()).unwrap();
        std::fs::write(&from, "").unwrap();

        let resolver = ModuleResolver::new(ResolveOptions::default()).unwrap();
        let resolved = resolver.resolve("dom-helpers/addClass", &from).unwrap();

        let resolved = std::fs::canonicalize(resolved.path).unwrap();
        assert!(resolved.ends_with("dom-helpers/esm/addClass.js"));
    }

    #[test]
    fn test_parse_package_specifier_edge_cases() {
        assert_eq!(
            parse_package_specifier("lodash/fp/map"),
            ("lodash".to_string(), Some("./fp/map".to_string()))
        );
        assert_eq!(
            parse_package_specifier("@org/pkg/a/b/c"),
            ("@org/pkg".to_string(), Some("./a/b/c".to_string()))
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Monorepo walk-up: resolver finds node_modules at workspace root
    // ──────────────────────────────────────────────────────────────────

    /// Simulate an Nx monorepo layout:
    ///
    ///   workspace_root/                ← contains nx.json
    ///     node_modules/react/          ← packages installed here
    ///       package.json
    ///       index.js
    ///     apps/demo/src/App.tsx        ← importing file
    ///
    /// The resolver must walk up from `apps/demo/src/` and find
    /// `node_modules/react` at the workspace root.
    #[test]
    fn test_resolver_walks_up_to_monorepo_root_node_modules() {
        use tempfile::tempdir;

        let workspace = tempdir().unwrap();
        let ws_root = workspace.path();

        // Create workspace root marker
        std::fs::write(ws_root.join("nx.json"), r#"{"affected":{}}"#).unwrap();

        // Create react package at workspace root node_modules
        let react_dir = ws_root.join("node_modules").join("react");
        std::fs::create_dir_all(&react_dir).unwrap();
        std::fs::write(
            react_dir.join("package.json"),
            r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(
            react_dir.join("index.js"),
            "exports.createElement = function() {}; exports.useState = function() {};",
        )
        .unwrap();

        // Create deeply nested app source file
        let src_dir = ws_root.join("apps").join("demo").join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let entry_file = src_dir.join("App.tsx");
        std::fs::write(
            &entry_file,
            r#"import React from 'react'; export default function App() { return null; }"#,
        )
        .unwrap();

        let resolver = ModuleResolver::new(ResolveOptions {
            extensions: vec![
                "js".to_string(),
                "jsx".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
            ],
            resolve_index: true,
            ..Default::default()
        })
        .unwrap();

        // Resolve 'react' from the nested app file
        let result = resolver.resolve("react", &entry_file);

        assert!(
            result.is_ok(),
            "Should resolve 'react' from workspace root node_modules, got: {:?}",
            result.err()
        );
        let resolved = result.unwrap();
        assert!(
            !resolved.is_external,
            "react must NOT be treated as external"
        );
        assert!(
            resolved
                .path
                .to_string_lossy()
                .contains("node_modules/react"),
            "Resolved path must be inside node_modules/react, got: {:?}",
            resolved.path
        );
    }

    /// Verify that the resolver correctly skips directories that don't have
    /// the target package in their node_modules and keeps walking up.
    #[test]
    fn test_resolver_skips_intermediate_node_modules_without_package() {
        use tempfile::tempdir;

        let workspace = tempdir().unwrap();
        let ws_root = workspace.path();

        // Intermediate node_modules WITHOUT react (has a different package)
        let intermediate_nm = ws_root.join("apps").join("demo").join("node_modules");
        std::fs::create_dir_all(intermediate_nm.join("lodash")).unwrap();
        std::fs::write(
            intermediate_nm.join("lodash").join("package.json"),
            r#"{"name":"lodash","version":"4.0.0","main":"lodash.js"}"#,
        )
        .unwrap();
        std::fs::write(
            intermediate_nm.join("lodash").join("lodash.js"),
            "exports.merge = function() {};",
        )
        .unwrap();

        // React only at workspace root
        let react_dir = ws_root.join("node_modules").join("react");
        std::fs::create_dir_all(&react_dir).unwrap();
        std::fs::write(
            react_dir.join("package.json"),
            r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(
            react_dir.join("index.js"),
            "exports.createElement = function() {};",
        )
        .unwrap();

        // Source file nested inside apps/demo/src/
        let src_dir = ws_root.join("apps").join("demo").join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let entry_file = src_dir.join("index.tsx");
        std::fs::write(&entry_file, "import React from 'react';").unwrap();

        let resolver = ModuleResolver::new(ResolveOptions {
            extensions: vec!["js".to_string(), "ts".to_string(), "tsx".to_string()],
            resolve_index: true,
            ..Default::default()
        })
        .unwrap();

        // 'react' must be found at workspace root even though intermediate
        // node_modules exists (it only has lodash, not react)
        let result = resolver.resolve("react", &entry_file);
        assert!(
            result.is_ok(),
            "react should be found at workspace root despite intermediate node_modules: {:?}",
            result.err()
        );

        // 'lodash' must be found at the intermediate level (closer wins)
        let lodash_result = resolver.resolve("lodash", &entry_file);
        assert!(
            lodash_result.is_ok(),
            "lodash should be found at intermediate node_modules: {:?}",
            lodash_result.err()
        );
        assert!(
            lodash_result
                .unwrap()
                .path
                .to_string_lossy()
                .contains("apps/demo/node_modules"),
            "lodash must resolve from the closer intermediate node_modules"
        );
    }

    #[test]
    fn test_singleton_react_prefers_hoisted_root_package() {
        use tempfile::tempdir;

        let workspace = tempdir().unwrap();
        let ws_root = workspace.path();

        let root_react = ws_root.join("node_modules").join("react");
        std::fs::create_dir_all(&root_react).unwrap();
        std::fs::write(
            root_react.join("package.json"),
            r#"{"name":"react","version":"18.3.1","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(root_react.join("index.js"), "exports.version = 'root';").unwrap();

        let nested_react = ws_root
            .join("node_modules")
            .join("react-dom")
            .join("node_modules")
            .join("react");
        std::fs::create_dir_all(&nested_react).unwrap();
        std::fs::write(
            nested_react.join("package.json"),
            r#"{"name":"react","version":"18.3.1","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(nested_react.join("index.js"), "exports.version = 'nested';").unwrap();

        let importer = ws_root
            .join("node_modules")
            .join("react-dom")
            .join("index.js");
        std::fs::write(&importer, "require('react');").unwrap();

        let resolver = ModuleResolver::new(ResolveOptions::default()).unwrap();
        let resolved = resolver.resolve("react", &importer).unwrap();

        assert_eq!(resolved.path, root_react.join("index.js"));
    }

    #[test]
    fn test_non_singleton_package_keeps_nearest_node_resolution() {
        use tempfile::tempdir;

        let workspace = tempdir().unwrap();
        let ws_root = workspace.path();

        let root_pkg = ws_root.join("node_modules").join("scheduler");
        std::fs::create_dir_all(&root_pkg).unwrap();
        std::fs::write(
            root_pkg.join("package.json"),
            r#"{"name":"scheduler","version":"0.1.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(root_pkg.join("index.js"), "exports.version = 'root';").unwrap();

        let nested_pkg = ws_root
            .join("node_modules")
            .join("react-dom")
            .join("node_modules")
            .join("scheduler");
        std::fs::create_dir_all(&nested_pkg).unwrap();
        std::fs::write(
            nested_pkg.join("package.json"),
            r#"{"name":"scheduler","version":"0.2.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(nested_pkg.join("index.js"), "exports.version = 'nested';").unwrap();

        let importer = ws_root
            .join("node_modules")
            .join("react-dom")
            .join("index.js");
        std::fs::write(&importer, "require('scheduler');").unwrap();

        let resolver = ModuleResolver::new(ResolveOptions::default()).unwrap();
        let resolved = resolver.resolve("scheduler", &importer).unwrap();

        assert_eq!(resolved.path, nested_pkg.join("index.js"));
    }

    // ──────────────────────────────────────────────────────────────────
    // T8: dev mode default conditions
    // ──────────────────────────────────────────────────────────────────

    /// T8: When ResolveOptions::default(), conditions is [import, browser, default].
    // REQ: R4
    #[test]
    fn test_dev_mode_default_conditions() {
        let options = ResolveOptions::default();
        assert_eq!(
            options.conditions,
            vec![
                "import".to_string(),
                "browser".to_string(),
                "default".to_string()
            ],
            "Default conditions must be [import, browser, default] for browser ESM dev mode"
        );
    }
}
// CODEGEN-END
