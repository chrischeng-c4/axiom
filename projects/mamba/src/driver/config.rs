use crate::pkgmanage::manifest::MambaConfig;

// @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R4
/// Compiler configuration.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub backend: Backend,
    pub emit: Option<EmitMode>,
    pub opt_level: OptLevel,
    /// Optional project-mode configuration loaded from `mamba.toml`.
    pub project_config: Option<MambaConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cranelift,
    CraneliftJit,
    Llvm,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitMode {
    Ast,
    Hir,
    Mir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Cranelift,
            emit: None,
            opt_level: OptLevel::O0,
            project_config: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::pkgmanage::manifest::schema::{ProjectConfig, CrateConfig, CrateEntry};

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn cfg_with_expose(crate_name: &str, symbols: &[&str]) -> MambaConfig {
        let mut crates = HashMap::new();
        crates.insert(
            crate_name.to_string(),
            CrateEntry::Config(CrateConfig {
                crate_name: None,
                version: Some("0.1.0".to_string()),
                path: None,
                expose: symbols.iter().map(|s| s.to_string()).collect(),
                module: None,
            }),
        );
        MambaConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                entry_point: Some("main.py".to_string()),
            },
            entry_point: None,
            crates,
            expose: Default::default(),
            build: Default::default(),
            paths: Default::default(),
        }
    }

    fn cfg_no_expose() -> MambaConfig {
        MambaConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                entry_point: Some("main.py".to_string()),
            },
            entry_point: None,
            crates: Default::default(),
            expose: Default::default(),
            build: Default::default(),
            paths: Default::default(),
        }
    }

    // ── is_symbol_exposed ─────────────────────────────────────────────────────

    #[test]
    fn expose_absent_crate_allows_any_symbol() {
        let cfg = cfg_no_expose();
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "BaseModel"));
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "Field"));
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "InternalSecret"));
    }

    #[test]
    fn expose_empty_list_allows_any_symbol() {
        // Empty expose list means "expose everything"
        let mut crates = HashMap::new();
        crates.insert(
            "cclab-schema-mamba".to_string(),
            CrateEntry::Config(CrateConfig {
                crate_name: None,
                version: Some("0.1.0".to_string()),
                path: None,
                expose: vec![],
                module: None,
            }),
        );
        let cfg = MambaConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                entry_point: None,
            },
            entry_point: None,
            crates,
            expose: Default::default(),
            build: Default::default(),
            paths: Default::default(),
        };
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "BaseModel"));
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "InternalSecret"));
    }

    #[test]
    fn expose_nonempty_list_allows_listed_symbol() {
        let cfg = cfg_with_expose("cclab-schema-mamba", &["BaseModel"]);
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "BaseModel"));
    }

    #[test]
    fn expose_nonempty_list_blocks_unlisted_symbol() {
        let cfg = cfg_with_expose("cclab-schema-mamba", &["BaseModel"]);
        assert!(
            !cfg.is_symbol_exposed("cclab-schema-mamba", "Field"),
            "Field should be blocked when not in expose list"
        );
        assert!(
            !cfg.is_symbol_exposed("cclab-schema-mamba", "InternalSecret"),
            "InternalSecret should be blocked when not in expose list"
        );
    }

    #[test]
    fn expose_allows_multiple_listed_symbols() {
        let cfg = cfg_with_expose("cclab-schema-mamba", &["BaseModel", "Field"]);
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "BaseModel"));
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "Field"));
        assert!(
            !cfg.is_symbol_exposed("cclab-schema-mamba", "Validator"),
            "Validator not in list -- should be blocked"
        );
    }

    #[test]
    fn expose_crate_restriction_does_not_affect_other_crates() {
        let cfg = cfg_with_expose("cclab-schema-mamba", &["BaseModel"]);
        // A crate not mentioned in the crates map has no restrictions.
        assert!(cfg.is_symbol_exposed("pgkit-binding", "Connection"));
        assert!(cfg.is_symbol_exposed("pgkit-binding", "AnySymbol"));
    }

    // ── MambaConfig::discover ─────────────────────────────────────────────────

    #[test]
    fn discover_finds_config_in_start_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mamba.toml"),
            "[project]\nname = \"test\"\nversion = \"0.1.0\"\nentry_point = \"src/main.py\"\n",
        )
        .unwrap();

        let (cfg, found_path) =
            MambaConfig::discover(dir.path()).expect("should find mamba.toml");
        assert_eq!(cfg.entry_point(), Some("src/main.py"));
        assert_eq!(found_path, dir.path().join("mamba.toml"));
    }

    #[test]
    fn discover_walks_up_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mamba.toml"),
            "[project]\nname = \"test\"\nversion = \"0.1.0\"\nentry_point = \"app.py\"\n",
        )
        .unwrap();

        // Start discovery from a deeply-nested sub-directory.
        let sub = dir.path().join("src").join("api").join("v2");
        std::fs::create_dir_all(&sub).unwrap();

        let (cfg, found_path) =
            MambaConfig::discover(&sub).expect("should walk up and find mamba.toml");
        assert_eq!(cfg.entry_point(), Some("app.py"));
        assert_eq!(found_path, dir.path().join("mamba.toml"));
    }

    #[test]
    fn discover_returns_none_in_isolated_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("isolated_no_toml_test");
        std::fs::create_dir_all(&sub).unwrap();

        let _result = MambaConfig::discover(&sub);
    }

    // ── MambaConfig::from_file ────────────────────────────────────────────────

    #[test]
    fn from_file_parses_project_and_crates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.toml");
        std::fs::write(
            &path,
            "[project]\nname = \"app\"\nversion = \"0.1.0\"\nentry_point = \"app.py\"\n\
             [crates.cclab-schema-mamba]\n\
             version = \"0.1.0\"\n\
             expose = [\"BaseModel\", \"Field\"]\n",
        )
        .unwrap();

        let cfg = MambaConfig::from_file(&path).unwrap();
        assert_eq!(cfg.entry_point(), Some("app.py"));
        assert!(cfg.crates.contains_key("cclab-schema-mamba"));
        let cr = &cfg.crates["cclab-schema-mamba"];
        assert!(cr.expose_list().contains(&"BaseModel".to_string()));
        assert!(cr.expose_list().contains(&"Field".to_string()));
    }

    #[test]
    fn from_file_error_on_missing_file() {
        let result =
            MambaConfig::from_file(std::path::Path::new("/nonexistent/path/mamba.toml"));
        assert!(result.is_err(), "missing file should produce an error");
    }

    #[test]
    fn from_file_error_on_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not [ valid toml!!!").unwrap();
        let result = MambaConfig::from_file(&path);
        assert!(result.is_err(), "invalid TOML should produce a parse error");
    }

    // ── Flat format backward compat (#1134) ──────────────────────────────────

    #[test]
    fn discover_flat_format_mamba_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mamba.toml"),
            "entry_point = \"src/main.py\"\n[crates]\ncclab-schema-mamba = \"0.1.0\"\n",
        )
        .unwrap();

        let (cfg, _) = MambaConfig::discover(dir.path()).expect("should find flat mamba.toml");
        assert_eq!(cfg.entry_point(), Some("src/main.py"));
        assert!(cfg.crates.contains_key("cclab-schema-mamba"));
    }
}
