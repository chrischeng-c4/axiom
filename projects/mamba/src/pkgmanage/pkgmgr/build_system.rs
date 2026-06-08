// build_system.rs — pyproject.toml `[build-system]` table reader (PEP 517).
//
// Every PEP 517 build frontend (pip, uv, build, hatchling, …) starts an
// sdist or wheel build by looking at `[build-system]` to find:
//
//     * `requires`       — build-time dependencies (PEP 518)
//     * `build-backend`  — Python dotted path to the backend module
//     * `backend-path`   — extra `sys.path` entries for in-tree backends
//
// When the table is missing entirely, PEP 517 says the build frontend
// MUST fall back to legacy setuptools (`requires = ["setuptools",
// "wheel"]`, no `build-backend`). We expose that fallback as
// `BuildSystem::legacy_setuptools`.

use toml::Value;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed `[build-system]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSystem {
    /// PEP 518 build-time dependencies, in source order. Each entry is
    /// kept verbatim as a PEP 508 requirement string.
    pub requires: Vec<String>,
    /// Dotted path to the backend module, e.g. `setuptools.build_meta`
    /// or `hatchling.build`. None means "legacy setuptools fallback".
    pub build_backend: Option<String>,
    /// Extra paths added to `sys.path` so in-tree backends can be
    /// imported during build. Used by hatchling self-host, flit-core
    /// bootstrap, etc.
    pub backend_path: Option<Vec<String>>,
}

impl BuildSystem {
    /// PEP 517 legacy fallback used when `[build-system]` is missing.
    pub fn legacy_setuptools() -> Self {
        BuildSystem {
            requires: vec!["setuptools".into(), "wheel".into()],
            build_backend: None,
            backend_path: None,
        }
    }
}

/// Parse `[build-system]` from the body of a pyproject.toml. Returns the
/// PEP 517 legacy fallback when the table is absent.
pub fn parse_build_system(src: &str) -> Result<BuildSystem, IndexError> {
    let doc: Value = toml::from_str(src).map_err(|e| IndexError::ParseError {
        url: "pyproject.toml".into(),
        detail: format!("invalid TOML: {e}"),
    })?;
    let table = match doc.get("build-system") {
        Some(v) => v.as_table().ok_or_else(|| IndexError::ParseError {
            url: "pyproject.toml".into(),
            detail: "[build-system] must be a TOML table".into(),
        })?,
        None => return Ok(BuildSystem::legacy_setuptools()),
    };

    // `requires` is required when `[build-system]` is present.
    let requires_v = table
        .get("requires")
        .ok_or_else(|| IndexError::ParseError {
            url: "pyproject.toml".into(),
            detail: "[build-system] is missing the required 'requires' key".into(),
        })?;
    let requires = string_array(requires_v, "[build-system].requires")?;

    let build_backend = match table.get("build-backend") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(_) => {
            return Err(IndexError::ParseError {
                url: "pyproject.toml".into(),
                detail: "[build-system].build-backend must be a string".into(),
            });
        }
        None => None,
    };

    let backend_path = match table.get("backend-path") {
        Some(v) => Some(string_array(v, "[build-system].backend-path")?),
        None => None,
    };

    Ok(BuildSystem {
        requires,
        build_backend,
        backend_path,
    })
}

fn string_array(v: &Value, name: &str) -> Result<Vec<String>, IndexError> {
    let arr = v.as_array().ok_or_else(|| IndexError::ParseError {
        url: "pyproject.toml".into(),
        detail: format!("{name} must be an array"),
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, entry) in arr.iter().enumerate() {
        match entry.as_str() {
            Some(s) => out.push(s.to_string()),
            None => {
                return Err(IndexError::ParseError {
                    url: "pyproject.toml".into(),
                    detail: format!("{name}[{i}] must be a string"),
                });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_table_returns_legacy_setuptools() {
        let bs = parse_build_system("[project]\nname = \"x\"\n").unwrap();
        assert_eq!(bs, BuildSystem::legacy_setuptools());
        assert_eq!(bs.requires, vec!["setuptools", "wheel"]);
        assert!(bs.build_backend.is_none());
    }

    #[test]
    fn full_table_parses() {
        let src = "\
[build-system]
requires = [\"hatchling>=1.18\", \"hatch-vcs\"]
build-backend = \"hatchling.build\"
backend-path = [\"src\", \"vendor\"]
";
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.requires, vec!["hatchling>=1.18", "hatch-vcs"]);
        assert_eq!(bs.build_backend.as_deref(), Some("hatchling.build"));
        assert_eq!(
            bs.backend_path.as_deref(),
            Some(&["src".into(), "vendor".into()][..])
        );
    }

    #[test]
    fn requires_only_omits_backend() {
        let src = "[build-system]\nrequires = [\"setuptools>=68\"]\n";
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.requires, vec!["setuptools>=68"]);
        assert!(bs.build_backend.is_none());
        assert!(bs.backend_path.is_none());
    }

    #[test]
    fn empty_requires_array_accepted() {
        // A no-deps backend (rare, but legal).
        let src = "[build-system]\nrequires = []\nbuild-backend = \"flit_core.buildapi\"\n";
        let bs = parse_build_system(src).unwrap();
        assert!(bs.requires.is_empty());
        assert_eq!(bs.build_backend.as_deref(), Some("flit_core.buildapi"));
    }

    #[test]
    fn invalid_toml_rejected() {
        let err = parse_build_system("[build-system\nrequires = [\"x\"]").unwrap_err();
        assert!(err.to_string().contains("invalid TOML"));
    }

    #[test]
    fn build_system_not_a_table_rejected() {
        let err = parse_build_system("build-system = 1\n").unwrap_err();
        assert!(err.to_string().contains("must be a TOML table"));
    }

    #[test]
    fn missing_requires_when_table_present_rejected() {
        let err = parse_build_system("[build-system]\nbuild-backend = \"x\"\n").unwrap_err();
        assert!(err.to_string().contains("'requires' key"));
    }

    #[test]
    fn requires_not_an_array_rejected() {
        let err = parse_build_system("[build-system]\nrequires = \"setuptools\"\n").unwrap_err();
        assert!(err.to_string().contains("requires must be an array"));
    }

    #[test]
    fn requires_array_with_non_string_rejected() {
        let err = parse_build_system("[build-system]\nrequires = [\"ok\", 42]\n").unwrap_err();
        assert!(err.to_string().contains("requires[1]"));
    }

    #[test]
    fn build_backend_not_a_string_rejected() {
        let err = parse_build_system("[build-system]\nrequires = [\"x\"]\nbuild-backend = 1\n")
            .unwrap_err();
        assert!(err.to_string().contains("build-backend must be a string"));
    }

    #[test]
    fn backend_path_not_an_array_rejected() {
        let err =
            parse_build_system("[build-system]\nrequires = [\"x\"]\nbackend-path = \"src\"\n")
                .unwrap_err();
        assert!(err.to_string().contains("backend-path must be an array"));
    }

    #[test]
    fn backend_path_with_non_string_rejected() {
        let err = parse_build_system("[build-system]\nrequires = [\"x\"]\nbackend-path = [1, 2]\n")
            .unwrap_err();
        assert!(err.to_string().contains("backend-path[0]"));
    }

    #[test]
    fn legacy_setuptools_helper_is_stable() {
        // Two independent constructions must compare equal — protects
        // against accidental drift in the fallback constant.
        assert_eq!(
            BuildSystem::legacy_setuptools(),
            BuildSystem::legacy_setuptools()
        );
    }

    #[test]
    fn preserves_requires_order() {
        let src = "[build-system]\nrequires = [\"z\", \"a\", \"m\"]\n";
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.requires, vec!["z", "a", "m"]);
    }

    #[test]
    fn preserves_backend_path_order() {
        let src = "[build-system]\nrequires = [\"x\"]\nbackend-path = [\"b\", \"a\"]\n";
        let bs = parse_build_system(src).unwrap();
        assert_eq!(bs.backend_path.unwrap(), vec!["b", "a"]);
    }
}
