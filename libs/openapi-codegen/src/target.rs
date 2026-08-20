// <HANDWRITE gap="missing-generator:target-profile-contract" tracker="#1569" reason="versioned language target profiles and toolchain requirements are hand-written">
//! Versioned language targets and their generated-artifact requirements.
//!
//! A target is intentionally separate from [`crate::Lang`]: `Lang` selects an
//! emitter, while a target selects the minimum language/toolchain contract and
//! any syntax that is safe to use for that contract.

use std::str::FromStr;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::Lang;

/// Python language level for generated clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonTarget {
    /// Python 3.11: native `Self` and PEP 604 union annotations.
    Py311,
    /// Python 3.12: Python 3.11 features plus PEP 695 `type` aliases.
    Py312,
    /// Python 3.13-compatible output.
    Py313,
    /// Python 3.14-compatible output.
    Py314,
}

impl PythonTarget {
    /// The minimum interpreter version accepted by the generated client.
    pub const fn minimum_version(self) -> &'static str {
        match self {
            Self::Py311 => "3.11",
            Self::Py312 => "3.12",
            Self::Py313 => "3.13",
            Self::Py314 => "3.14",
        }
    }

    /// PEP 695's `type` statement first shipped in Python 3.12.
    pub const fn uses_pep695_type_aliases(self) -> bool {
        !matches!(self, Self::Py311)
    }
}

/// TypeScript compiler level for generated clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeScriptTarget {
    /// TypeScript 5.0+ with modern ESM/type-only export support.
    Ts50,
}

impl TypeScriptTarget {
    /// The minimum TypeScript compiler version accepted by the generated client.
    pub const fn minimum_version(self) -> &'static str {
        match self {
            Self::Ts50 => "5.0",
        }
    }
}

/// Rust edition for generated clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustTarget {
    /// Rust 2021 edition output.
    Rust2021,
    /// Rust 2024 edition output, including its additional reserved keywords.
    Rust2024,
}

impl RustTarget {
    /// The minimum Rust toolchain version for this edition.
    pub const fn minimum_version(self) -> &'static str {
        match self {
            Self::Rust2021 => "1.56",
            Self::Rust2024 => "1.85",
        }
    }

    /// Rust 2024 reserves `gen` for future generator blocks.
    pub const fn reserves_gen(self) -> bool {
        matches!(self, Self::Rust2024)
    }
}

/// A language-valid target profile for one generation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProfile {
    Python(PythonTarget),
    TypeScript(TypeScriptTarget),
    Rust(RustTarget),
}

impl TargetProfile {
    /// The language this profile may be used with.
    pub const fn lang(self) -> Lang {
        match self {
            Self::Python(_) => Lang::Py,
            Self::TypeScript(_) => Lang::Ts,
            Self::Rust(_) => Lang::Rust,
        }
    }

    /// The conservative default keeps existing callers source-compatible while
    /// making the default Python client a modern Python 3.11 artifact.
    pub const fn default_for(lang: Lang) -> Self {
        match lang {
            Lang::Py => Self::Python(PythonTarget::Py311),
            Lang::Ts => Self::TypeScript(TypeScriptTarget::Ts50),
            Lang::Rust => Self::Rust(RustTarget::Rust2021),
        }
    }

    /// Stable target identifier used in generated-output metadata.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Python(PythonTarget::Py311) => "python-3.11",
            Self::Python(PythonTarget::Py312) => "python-3.12",
            Self::Python(PythonTarget::Py313) => "python-3.13",
            Self::Python(PythonTarget::Py314) => "python-3.14",
            Self::TypeScript(TypeScriptTarget::Ts50) => "typescript-5.0",
            Self::Rust(RustTarget::Rust2021) => "rust-2021",
            Self::Rust(RustTarget::Rust2024) => "rust-2024",
        }
    }

    /// Deterministic requirements carried with every generated output.
    pub const fn requirements(self) -> TargetRequirements {
        match self {
            Self::Python(target) => TargetRequirements {
                target: self.id(),
                language: Lang::Py,
                compiler: "python",
                minimum_version: target.minimum_version(),
                language_standard: target.minimum_version(),
                module_system: None,
                module_resolution: None,
                strict: None,
                transport: Some("generated-h2c-and-tls-alpn-h2"),
                runtime_dependencies: &["pydantic>=2"],
            },
            Self::TypeScript(target) => TargetRequirements {
                target: self.id(),
                language: Lang::Ts,
                compiler: "typescript",
                minimum_version: target.minimum_version(),
                language_standard: "ES2022",
                module_system: Some("ESNext"),
                module_resolution: Some("Bundler"),
                strict: Some(true),
                transport: Some("fetch-or-axios"),
                runtime_dependencies: &[],
            },
            Self::Rust(target) => TargetRequirements {
                target: self.id(),
                language: Lang::Rust,
                compiler: "rustc",
                minimum_version: target.minimum_version(),
                language_standard: match target {
                    RustTarget::Rust2021 => "2021",
                    RustTarget::Rust2024 => "2024",
                },
                module_system: None,
                module_resolution: None,
                strict: None,
                transport: Some("reqwest-blocking"),
                runtime_dependencies: &["reqwest", "serde", "serde_json"],
            },
        }
    }

    /// Parse the stable identifier exposed by the CLI and generation manifest.
    pub fn from_id(id: &str) -> Result<Self> {
        id.parse()
    }
}

impl FromStr for TargetProfile {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "python-3.11" => Ok(Self::Python(PythonTarget::Py311)),
            "python-3.12" => Ok(Self::Python(PythonTarget::Py312)),
            "python-3.13" => Ok(Self::Python(PythonTarget::Py313)),
            "python-3.14" => Ok(Self::Python(PythonTarget::Py314)),
            "typescript-5.0" => Ok(Self::TypeScript(TypeScriptTarget::Ts50)),
            "rust-2021" => Ok(Self::Rust(RustTarget::Rust2021)),
            "rust-2024" => Ok(Self::Rust(RustTarget::Rust2024)),
            _ => bail!(
                "unknown target profile {value:?}; expected one of: python-3.11, python-3.12, \
                 python-3.13, python-3.14, typescript-5.0, rust-2021, rust-2024"
            ),
        }
    }
}

/// A project-owned, pinned policy for generated client targets.
///
/// The policy deliberately preserves the exact public version contract. It
/// resolves that contract once before generation, while emitters consume only
/// the syntax capabilities implied by the resulting [`TargetProfile`]. This
/// keeps a future target bump to a policy-file edit when no new syntax is
/// introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetPolicy {
    typescript: TargetProfile,
    python: TargetProfile,
    rust: TargetProfile,
}

impl TargetPolicy {
    /// Parse a project `codegen.toml` target policy.
    ///
    /// ```toml
    /// [targets]
    /// typescript = "typescript-5.0"
    /// python = "python-3.14"
    /// rust = "rust-2024"
    /// ```
    pub fn from_toml(source: &str) -> Result<Self> {
        let raw: RawTargetPolicy = toml::from_str(source).context("parse codegen target policy")?;
        Ok(Self {
            typescript: parse_policy_target(
                raw.targets.typescript,
                Lang::Ts,
                "targets.typescript",
            )?,
            python: parse_policy_target(raw.targets.python, Lang::Py, "targets.python")?,
            rust: parse_policy_target(raw.targets.rust, Lang::Rust, "targets.rust")?,
        })
    }

    /// Resolve a language target, using the policy by default or an explicit
    /// CLI override when supplied. A cross-language override is rejected.
    pub fn resolve(self, lang: Lang, explicit: Option<&str>) -> Result<TargetProfile> {
        let target = match explicit {
            Some(value) => TargetProfile::from_id(value)
                .with_context(|| format!("parse explicit target profile {value:?}"))?,
            None => self.for_lang(lang),
        };
        if target.lang() != lang {
            bail!(
                "target profile {} is for {:?}, not requested language {:?}",
                target.id(),
                target.lang(),
                lang
            );
        }
        Ok(target)
    }

    /// The policy default for one target language.
    pub const fn for_lang(self, lang: Lang) -> TargetProfile {
        match lang {
            Lang::Ts => self.typescript,
            Lang::Py => self.python,
            Lang::Rust => self.rust,
        }
    }
}

#[derive(Deserialize)]
struct RawTargetPolicy {
    targets: RawTargets,
}

#[derive(Deserialize)]
struct RawTargets {
    typescript: String,
    python: String,
    rust: String,
}

fn parse_policy_target(value: String, lang: Lang, key: &str) -> Result<TargetProfile> {
    let target = TargetProfile::from_id(&value).with_context(|| format!("parse {key}"))?;
    if target.lang() != lang {
        bail!("{key} must select {:?}, got {}", lang, target.id());
    }
    Ok(target)
}

/// Minimum toolchain and runtime dependencies required by generated artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetRequirements {
    pub target: &'static str,
    pub language: Lang,
    pub compiler: &'static str,
    pub minimum_version: &'static str,
    pub language_standard: &'static str,
    pub module_system: Option<&'static str>,
    pub module_resolution: Option<&'static str>,
    pub strict: Option<bool>,
    pub transport: Option<&'static str>,
    pub runtime_dependencies: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_are_language_valid_and_have_stable_requirements() {
        let profile = TargetProfile::Python(PythonTarget::Py312);
        assert_eq!(profile.lang(), Lang::Py);
        assert_eq!(profile.id(), "python-3.12");
        assert_eq!(profile.requirements().minimum_version, "3.12");
        assert!(profile
            .requirements()
            .runtime_dependencies
            .contains(&"pydantic>=2"));

        assert!(RustTarget::Rust2024.reserves_gen());
        assert!(!RustTarget::Rust2021.reserves_gen());

        let typescript = TargetProfile::TypeScript(TypeScriptTarget::Ts50).requirements();
        assert_eq!(typescript.language_standard, "ES2022");
        assert_eq!(typescript.module_system, Some("ESNext"));
        assert_eq!(typescript.module_resolution, Some("Bundler"));
        assert_eq!(typescript.strict, Some(true));

        let rust = TargetProfile::Rust(RustTarget::Rust2024).requirements();
        assert_eq!(rust.language_standard, "2024");
        assert_eq!(rust.transport, Some("reqwest-blocking"));
    }

    #[test]
    fn target_policy_keeps_version_contracts_and_rejects_cross_language_overrides() {
        let policy = TargetPolicy::from_toml(
            r#"
                [targets]
                typescript = "typescript-5.0"
                python = "python-3.14"
                rust = "rust-2024"
            "#,
        )
        .unwrap();

        assert_eq!(policy.resolve(Lang::Py, None).unwrap().id(), "python-3.14");
        assert_eq!(
            policy.resolve(Lang::Py, Some("python-3.11")).unwrap().id(),
            "python-3.11"
        );
        assert!(policy.resolve(Lang::Py, Some("rust-2024")).is_err());
        assert!(TargetProfile::from_id("python-3.15").is_err());
    }
}
// </HANDWRITE>
