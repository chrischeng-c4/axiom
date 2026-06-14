// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
// CODEGEN-BEGIN
//! Path alias resolver for the Jet bundler.
//!
//! Combines alias entries from two sources (in priority order):
//!
//! 1. `tsconfig.json` → `compilerOptions.paths` (lower priority)
//! 2. `jet.toml` → `alias` map (higher priority; overrides tsconfig)
//!
//! The resulting entries are used to populate [`crate::resolver::ResolveOptions::alias`]
//! so that both the JIT dev server and production build resolve `@/foo` → `./src/foo`
//! identically.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolved path alias entries.
///
/// Each entry is `(prefix, target_path)` where `prefix` is the alias key
/// (e.g. `"@/"`) and `target_path` is the absolute directory it maps to.
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
#[derive(Debug, Clone, Default)]
pub struct AliasResolver {
    /// Sorted by descending prefix length (longest-prefix wins).
    pub entries: Vec<(String, PathBuf)>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
impl AliasResolver {
    /// Build an `AliasResolver` by merging `tsconfig.json` paths and the
    /// `jet.toml` alias map.
    ///
    /// `config_aliases` entries take precedence — any tsconfig path whose
    /// prefix matches a jet-config key is replaced.
    pub fn load(project_root: &Path, config_aliases: &HashMap<String, String>) -> Self {
        let mut entries: Vec<(String, PathBuf)> = Vec::new();

        // 1. Load tsconfig.json compilerOptions.paths (base priority)
        if let Some(tsconfig_entries) = load_tsconfig_paths(project_root) {
            entries.extend(tsconfig_entries);
        }

        // 2. Apply jet.toml aliases (overrides any tsconfig entry for the same prefix)
        for (alias_key, target) in config_aliases {
            // Normalise: strip glob suffix ("@/*" → "@/")
            let prefix = alias_key.trim_end_matches('*').to_string();
            let target_path_str = target.trim_end_matches('*');
            let target_path = project_root.join(target_path_str);

            // Remove existing entry with the same normalised prefix.
            entries.retain(|(k, _)| k != &prefix);
            entries.push((prefix, target_path));
        }

        // Sort by descending prefix length so longest match wins.
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        Self { entries }
    }

    /// Return entries in the format expected by [`crate::resolver::ResolveOptions::alias`].
    pub fn to_resolve_aliases(&self) -> Vec<(String, PathBuf)> {
        self.entries.clone()
    }

    /// Returns `true` if no alias entries are configured.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ─── tsconfig.json parsing ────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
struct TsConfig {
    #[serde(rename = "compilerOptions", default)]
    compiler_options: CompilerOptions,
}

#[derive(Debug, Deserialize, Default)]
struct CompilerOptions {
    /// `compilerOptions.paths` — map of alias → list of path patterns.
    #[serde(default)]
    paths: HashMap<String, Vec<String>>,

    /// `compilerOptions.baseUrl` — base directory for path resolution.
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// REQ-JET-05: tsconfig.json compilerOptions.paths are loaded as alias entries.
    #[test]
    fn alias_resolves_tsconfig_paths() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        let tsconfig = r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["./src/*"]
                }
            }
        }"#;
        std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();

        let resolver = AliasResolver::load(project_root, &HashMap::new());

        assert!(!resolver.is_empty(), "Should load tsconfig paths");
        let entries = resolver.to_resolve_aliases();
        assert_eq!(entries.len(), 1);
        // Glob suffix stripped: "@/*" → "@/"
        assert_eq!(entries[0].0, "@/");
        // Path resolved relative to baseUrl (".")
        assert_eq!(entries[0].1, project_root.join("./src/"));
    }

    /// Multiple tsconfig.json paths are all loaded.
    #[test]
    fn alias_resolves_multiple_tsconfig_paths() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        let tsconfig = r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["./src/*"],
                    "~/*": ["./lib/*"]
                }
            }
        }"#;
        std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();

        let resolver = AliasResolver::load(project_root, &HashMap::new());
        assert_eq!(resolver.to_resolve_aliases().len(), 2);
    }

    /// REQ-JET-06: jet.toml alias takes precedence over tsconfig.json paths for the same prefix.
    #[test]
    fn alias_config_overrides_tsconfig() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        // tsconfig defines @/ → ./tsconfig-src/
        let tsconfig = r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["./tsconfig-src/*"]
                }
            }
        }"#;
        std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();

        // jet config defines @/ → ./jet-src/ (should win)
        let mut config_aliases = HashMap::new();
        config_aliases.insert("@/*".to_string(), "./jet-src/".to_string());

        let resolver = AliasResolver::load(project_root, &config_aliases);
        let entries = resolver.to_resolve_aliases();

        assert_eq!(
            entries.len(),
            1,
            "Config override should replace tsconfig entry"
        );
        assert_eq!(entries[0].0, "@/");
        assert_eq!(
            entries[0].1,
            project_root.join("./jet-src/"),
            "jet.config alias should override tsconfig path"
        );
    }

    /// REQ-JET-06: jet.config alias is loaded even without a tsconfig.json.
    #[test]
    fn alias_config_only_no_tsconfig() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        let mut config_aliases = HashMap::new();
        config_aliases.insert("@/".to_string(), "./src/".to_string());

        let resolver = AliasResolver::load(project_root, &config_aliases);
        assert!(!resolver.is_empty());
        assert_eq!(resolver.entries[0].0, "@/");
    }

    /// No tsconfig and no config aliases → empty resolver.
    #[test]
    fn alias_empty_when_no_sources() {
        let dir = tempfile::tempdir().unwrap();
        let resolver = AliasResolver::load(dir.path(), &HashMap::new());
        assert!(resolver.is_empty());
    }

    /// GH #3157 — Absent tsconfig.json is the legitimate "no aliases"
    /// path; it must remain silent (no panic, no log, just empty).
    #[test]
    fn load_tsconfig_paths_returns_none_when_file_absent() {
        let dir = tempfile::tempdir().unwrap();
        let paths = load_tsconfig_paths(dir.path());
        assert!(
            paths.is_none(),
            "missing tsconfig.json must yield None silently"
        );
    }

    /// GH #3157 — Malformed JSON must not panic and must not silently
    /// "succeed" with zero aliases the way `.ok()?` did. It returns None
    /// here so the resolver still functions, but `eprintln!` surfaces the
    /// diagnostic to the developer (verified by manual inspection of the
    /// new code path — not asserted in the test to avoid coupling to
    /// stderr capture).
    #[test]
    fn load_tsconfig_paths_returns_none_when_json_malformed() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        // Unterminated string + missing brace — clearly malformed.
        let bad = r#"{ "compilerOptions": { "paths": { "@/*"   "#;
        std::fs::write(project_root.join("tsconfig.json"), bad).unwrap();

        let paths = load_tsconfig_paths(project_root);
        assert!(
            paths.is_none(),
            "malformed tsconfig must yield None without panicking"
        );

        // Higher-level loader must keep functioning (empty resolver, no panic).
        let resolver = AliasResolver::load(project_root, &HashMap::new());
        assert!(
            resolver.is_empty(),
            "malformed tsconfig must not crash AliasResolver::load"
        );
    }

    /// GH #3157 — Valid tsconfig.json with a `paths` entry yields
    /// `Some` non-empty entries. Pins that the new error-handling path
    /// didn't accidentally regress the happy path.
    #[test]
    fn load_tsconfig_paths_returns_entries_for_valid_tsconfig() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        let tsconfig = r#"{
            "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["./src/*"]
                }
            }
        }"#;
        std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();

        let paths =
            load_tsconfig_paths(project_root).expect("valid tsconfig must yield Some entries");
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].0, "@/");
    }

    /// Entries are sorted longest-prefix-first so longest match wins.
    #[test]
    fn alias_entries_sorted_longest_first() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        let mut config_aliases = HashMap::new();
        config_aliases.insert("@/".to_string(), "./src/".to_string());
        config_aliases.insert("@/components/".to_string(), "./src/components/".to_string());

        let resolver = AliasResolver::load(project_root, &config_aliases);
        let entries = resolver.to_resolve_aliases();

        assert_eq!(entries.len(), 2);
        // Longer prefix must come first
        assert!(
            entries[0].0.len() >= entries[1].0.len(),
            "Entries should be sorted longest-prefix-first"
        );
    }
}

// ─── tsconfig.json parsing ────────────────────────────────────────────────────

/// Attempt to load path aliases from `{project_root}/tsconfig.json`.
///
/// Returns `None` in three cases (all yielding "no aliases from tsconfig"):
///
/// 1. The file genuinely doesn't exist (silent — most projects).
/// 2. The file exists but is unreadable (e.g. permission denied) — logged at
///    WARN so a misconfigured project surfaces a diagnostic rather than
///    silently losing every `@/foo` import.
/// 3. The file exists but its JSON is malformed — printed to stderr so a
///    developer running `jet dev` or `jet build` notices the typo instead
///    of debugging a flood of "module not found" errors.
///
/// GH #3157 — replaced the prior `.ok()?` chain that swallowed cases 2 and 3
/// indistinguishably from case 1.
fn load_tsconfig_paths(project_root: &Path) -> Option<Vec<(String, PathBuf)>> {
    let tsconfig_path = project_root.join("tsconfig.json");

    let content = match std::fs::read_to_string(&tsconfig_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(
                target: "jet::resolver::tsconfig",
                "tsconfig at {:?} exists but could not be read: {e}; \
                 path aliases will not be loaded (GH #3157)",
                tsconfig_path
            );
            return None;
        }
    };
    let tsconfig: TsConfig = match serde_json::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "[jet resolver] tsconfig.json at {} could not be parsed: \
                 {e}. Path aliases (compilerOptions.paths) will not be \
                 loaded for this project (GH #3157).",
                tsconfig_path.display()
            );
            return None;
        }
    };

    let base_url = tsconfig
        .compiler_options
        .base_url
        .as_deref()
        .map(|b| project_root.join(b))
        .unwrap_or_else(|| project_root.to_path_buf());

    let mut entries = Vec::new();

    for (alias, path_patterns) in &tsconfig.compiler_options.paths {
        let first_pattern = match path_patterns.first() {
            Some(p) => p,
            None => continue,
        };

        // Normalise alias: strip glob suffix ("@/*" → "@/")
        let prefix = alias.trim_end_matches('*').to_string();
        // Normalise path: strip glob suffix ("./src/*" → "./src/")
        let path_str = first_pattern.trim_end_matches('*');
        let target_path = base_url.join(path_str);

        entries.push((prefix, target_path));
    }

    Some(entries)
}
// CODEGEN-END
