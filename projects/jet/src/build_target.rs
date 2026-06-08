// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Build target plumbing — `--target {web,desktop,tui}`.
//!
//! @spec .aw/tech-design/projects/jet/logic/multi-target/build-targets.md
//!
//! Slice 1 of #1239: the typed enum, parsing, and incompatibility
//! validation. Wiring into `wasm_build::build_with_profile` (cargo
//! features) and the `jet-target.json` manifest emission are
//! follow-up slices.

use std::fmt;

/// Canonical build target. Mirrors the `target:` field in
/// `target-profiles.yaml` and the `target-{web,desktop,tui}` cargo
/// features on `jet-multi-target`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTarget {
    Web,
    Desktop,
    Tui,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl BuildTarget {
    pub const ALL: &'static [&'static str] = &["web", "desktop", "tui"];

    pub fn as_str(self) -> &'static str {
        match self {
            BuildTarget::Web => "web",
            BuildTarget::Desktop => "desktop",
            BuildTarget::Tui => "tui",
        }
    }

    /// The cargo feature on `jet-multi-target` that selects this
    /// target's profile via `use_target()` (H3).
    pub fn target_feature(self) -> &'static str {
        match self {
            BuildTarget::Web => "jet-multi-target/target-web",
            BuildTarget::Desktop => "jet-multi-target/target-desktop",
            BuildTarget::Tui => "jet-multi-target/target-tui",
        }
    }

    /// True iff the target's primary artifact is a wasm module. TUI
    /// is the lone outlier; it builds a native binary.
    pub fn produces_wasm(self) -> bool {
        matches!(self, BuildTarget::Web | BuildTarget::Desktop)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl fmt::Display for BuildTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildTargetError {
    Unknown(String),
    /// `--target tui` cannot be combined with `--wasm` because TUI
    /// builds emit a native binary, not a wasm module.
    TuiWithWasm,
    /// JS-bundle-only flags rejected for non-bundler targets.
    JsBundleFlagOnNonBundler {
        target: BuildTarget,
        flag: &'static str,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl fmt::Display for BuildTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildTargetError::Unknown(value) => write!(
                f,
                "unknown --target value '{}': must be one of {}",
                value,
                BuildTarget::ALL.join(", "),
            ),
            BuildTargetError::TuiWithWasm => {
                f.write_str("--target tui builds a native binary; drop --wasm")
            }
            BuildTargetError::JsBundleFlagOnNonBundler { target, flag } => write!(
                f,
                "--{} is a JS-bundle flag; not valid for --target {}",
                flag, target,
            ),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl std::error::Error for BuildTargetError {}

/// Parse the raw CLI value. `None` means the flag was omitted; the
/// caller fills in the `web` default and prints the `info: target=...`
/// log line per the spec.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn parse(raw: Option<&str>) -> Result<Option<BuildTarget>, BuildTargetError> {
    let Some(s) = raw else { return Ok(None) };
    Ok(Some(match s {
        "web" => BuildTarget::Web,
        "desktop" => BuildTarget::Desktop,
        "tui" => BuildTarget::Tui,
        other => return Err(BuildTargetError::Unknown(other.to_string())),
    }))
}

/// Resolve the effective target (apply the web default).
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn resolve(raw: Option<&str>) -> Result<BuildTarget, BuildTargetError> {
    Ok(parse(raw)?.unwrap_or(BuildTarget::Web))
}

/// Snapshot of every bundler-flag the build subcommand exposes, used
/// by [`validate_combination`] to gate (target, flag) pairs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Copy, Default)]
pub struct FlagSnapshot {
    pub wasm: bool,
    pub minify: bool,
    pub sourcemap_set: bool,
    pub splitting: bool,
    pub drop_set: bool,
}

/// Apply the validation table from `build-targets.md`. First failure
/// wins; the caller surfaces it before any expensive build step runs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn validate_combination(
    target: BuildTarget,
    flags: FlagSnapshot,
) -> Result<(), BuildTargetError> {
    if matches!(target, BuildTarget::Tui) {
        if flags.wasm {
            return Err(BuildTargetError::TuiWithWasm);
        }
        // The bundler-only flags map 1:1 to the spec's table.
        for (raised, name) in [
            (flags.minify, "minify"),
            (flags.sourcemap_set, "sourcemap"),
            (flags.splitting, "splitting"),
            (flags.drop_set, "drop"),
        ] {
            if raised {
                return Err(BuildTargetError::JsBundleFlagOnNonBundler { target, flag: name });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_targets() {
        assert_eq!(parse(Some("web")).unwrap(), Some(BuildTarget::Web));
        assert_eq!(parse(Some("desktop")).unwrap(), Some(BuildTarget::Desktop));
        assert_eq!(parse(Some("tui")).unwrap(), Some(BuildTarget::Tui));
    }

    #[test]
    fn parse_none_is_ok() {
        assert_eq!(parse(None).unwrap(), None);
    }

    #[test]
    fn parse_unknown_lists_valid_options() {
        let err = parse(Some("ios")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ios"));
        assert!(msg.contains("web"));
        assert!(msg.contains("desktop"));
        assert!(msg.contains("tui"));
    }

    #[test]
    fn resolve_defaults_to_web() {
        assert_eq!(resolve(None).unwrap(), BuildTarget::Web);
    }

    #[test]
    fn target_features_are_unique() {
        let features: Vec<_> = [BuildTarget::Web, BuildTarget::Desktop, BuildTarget::Tui]
            .iter()
            .map(|t| t.target_feature())
            .collect();
        let mut sorted = features.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), features.len());
    }

    #[test]
    fn produces_wasm_matches_spec() {
        assert!(BuildTarget::Web.produces_wasm());
        assert!(BuildTarget::Desktop.produces_wasm());
        assert!(!BuildTarget::Tui.produces_wasm());
    }

    #[test]
    fn tui_rejects_wasm() {
        let flags = FlagSnapshot {
            wasm: true,
            ..Default::default()
        };
        let err = validate_combination(BuildTarget::Tui, flags).unwrap_err();
        assert_eq!(err, BuildTargetError::TuiWithWasm);
        assert!(err.to_string().contains("native binary"));
    }

    #[test]
    fn tui_rejects_each_bundler_flag() {
        for (set, name) in [
            (
                FlagSnapshot {
                    minify: true,
                    ..Default::default()
                },
                "minify",
            ),
            (
                FlagSnapshot {
                    sourcemap_set: true,
                    ..Default::default()
                },
                "sourcemap",
            ),
            (
                FlagSnapshot {
                    splitting: true,
                    ..Default::default()
                },
                "splitting",
            ),
            (
                FlagSnapshot {
                    drop_set: true,
                    ..Default::default()
                },
                "drop",
            ),
        ] {
            let err = validate_combination(BuildTarget::Tui, set).unwrap_err();
            match err {
                BuildTargetError::JsBundleFlagOnNonBundler { target, flag } => {
                    assert_eq!(target, BuildTarget::Tui);
                    assert_eq!(flag, name);
                }
                other => panic!(
                    "expected JsBundleFlagOnNonBundler({}), got {:?}",
                    name, other
                ),
            }
        }
    }

    #[test]
    fn web_accepts_full_bundler_flags() {
        let flags = FlagSnapshot {
            wasm: true,
            minify: true,
            sourcemap_set: true,
            splitting: true,
            drop_set: true,
        };
        validate_combination(BuildTarget::Web, flags).unwrap();
    }

    #[test]
    fn desktop_accepts_full_bundler_flags() {
        let flags = FlagSnapshot {
            wasm: true,
            minify: true,
            sourcemap_set: true,
            splitting: true,
            drop_set: true,
        };
        validate_combination(BuildTarget::Desktop, flags).unwrap();
    }
}
// CODEGEN-END
