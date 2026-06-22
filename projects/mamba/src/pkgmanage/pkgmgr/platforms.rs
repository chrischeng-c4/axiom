// Cross-platform resolution support (Tick 25).
//
// uv's "universal" lockfile is one file that's valid across multiple
// `(python_version, sys_platform, platform_machine, libc)` tuples. Each
// requirement is included if its PEP 508 marker evaluates true under *at
// least one* supported environment; entries that only apply on a subset
// of envs carry that subset as metadata so `mamba sync` knows which
// rows to install on the current host.
//
// This module ships:
//   - `SupportedEnvironment` — one universal-lock target. Includes Python
//     version + the platform tuple needed to evaluate platform markers.
//   - `EnvironmentSet` — a deduplicated collection of `SupportedEnvironment`s,
//     plus presets for the common "well-known platforms × stable pythons"
//     matrices.
//   - `evaluate_marker_across_envs` — run a marker against every env in
//     the set; return the subset of envs where it's true.
//   - `applies_anywhere` / `applies_everywhere` — the two questions the
//     resolver actually asks ("include this at all?", "is it conditional?").
//
// We piggyback on the existing `markers::evaluate` + `markers::MarkerEnv`
// from Tick 12 instead of re-implementing PEP 508 parsing.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::markers::{self, MarkerEnv};
use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// One target environment for the universal lockfile. Drives marker
/// evaluation + wheel-tag selection (`platform_machine` and the libc tag
/// are what differentiate `manylinux_2_28_x86_64` from `_aarch64`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupportedEnvironment {
    pub python_version: PythonVersion,
    /// `sys.platform` value, e.g. `"linux"`, `"darwin"`, `"win32"`.
    pub sys_platform: String,
    /// `platform.machine()` value, e.g. `"x86_64"`, `"aarch64"`,
    /// `"arm64"`, `"AMD64"`.
    pub platform_machine: String,
    /// libc family for Linux targets, `None` on macOS / Windows. Used by
    /// PEP 600 manylinux tag matching downstream.
    pub libc: Option<LibcFamily>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LibcFamily {
    Glibc,
    Musl,
}

impl SupportedEnvironment {
    /// Build the `MarkerEnv` exposed to PEP 508 markers for *this* target.
    /// `python_full_version` is `python_version.major.minor.patch`.
    /// `os_name` and `platform_system` are derived from `sys_platform`.
    pub fn to_marker_env(&self) -> MarkerEnv {
        let python_version_short = format!(
            "{}.{}",
            self.python_version.major, self.python_version.minor
        );
        let python_full = self.python_version.to_string();

        let (os_name, platform_system) = match self.sys_platform.as_str() {
            "linux" => ("posix", "Linux"),
            "darwin" => ("posix", "Darwin"),
            "win32" => ("nt", "Windows"),
            _ => ("posix", "Unknown"),
        };

        MarkerEnv {
            python_version: python_version_short,
            python_full_version: python_full,
            implementation_name: "cpython".to_string(),
            implementation_version: format!("{}.0", self.python_version),
            os_name: os_name.to_string(),
            platform_machine: self.platform_machine.clone(),
            platform_release: String::new(),
            platform_system: platform_system.to_string(),
            platform_version: String::new(),
            platform_python_implementation: "CPython".to_string(),
            sys_platform: self.sys_platform.clone(),
            extras: BTreeSet::new(),
        }
    }

    /// Short stable label, e.g. `cpython-3.12-linux-x86_64-glibc`. Stable
    /// across runs so lockfile diffs are reviewable.
    pub fn label(&self) -> String {
        let libc = match self.libc {
            Some(LibcFamily::Glibc) => "-glibc",
            Some(LibcFamily::Musl) => "-musl",
            None => "",
        };
        format!(
            "cpython-{}.{}-{}-{}{}",
            self.python_version.major,
            self.python_version.minor,
            self.sys_platform,
            self.platform_machine,
            libc
        )
    }
}

/// A deduplicated collection of `SupportedEnvironment`s in stable order.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentSet {
    envs: Vec<SupportedEnvironment>,
}

impl EnvironmentSet {
    pub fn new() -> Self {
        Self { envs: Vec::new() }
    }

    pub fn from_iter_dedup<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SupportedEnvironment>,
    {
        let mut set = Self::new();
        for env in iter {
            set.insert(env);
        }
        set
    }

    pub fn insert(&mut self, env: SupportedEnvironment) {
        if !self.envs.contains(&env) {
            self.envs.push(env);
            self.envs.sort();
        }
    }

    pub fn len(&self) -> usize {
        self.envs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.envs.is_empty()
    }

    pub fn as_slice(&self) -> &[SupportedEnvironment] {
        &self.envs
    }

    /// Cartesian-product preset: every Python version × every (sys_platform,
    /// platform_machine, libc) tuple. Useful for declaring "lock for the
    /// full standard matrix" from one knob.
    pub fn cross_product(
        python_versions: &[PythonVersion],
        platforms: &[(String, String, Option<LibcFamily>)],
    ) -> Self {
        let mut set = Self::new();
        for py in python_versions {
            for (sys_platform, platform_machine, libc) in platforms {
                set.insert(SupportedEnvironment {
                    python_version: *py,
                    sys_platform: sys_platform.clone(),
                    platform_machine: platform_machine.clone(),
                    libc: *libc,
                });
            }
        }
        set
    }

    /// Preset for "the standard matrix" — Python 3.10 / 3.11 / 3.12 / 3.13
    /// times Linux x86_64 + aarch64 (glibc), macOS x86_64 + arm64, Windows
    /// x86_64. 4 × 5 = 20 environments. Stable list so lockfile output is
    /// deterministic.
    pub fn standard_matrix() -> Self {
        let pys = [
            PythonVersion::new(3, 10, 0),
            PythonVersion::new(3, 11, 0),
            PythonVersion::new(3, 12, 0),
            PythonVersion::new(3, 13, 0),
        ];
        let plats: Vec<(String, String, Option<LibcFamily>)> = vec![
            ("linux".into(), "x86_64".into(), Some(LibcFamily::Glibc)),
            ("linux".into(), "aarch64".into(), Some(LibcFamily::Glibc)),
            ("darwin".into(), "x86_64".into(), None),
            ("darwin".into(), "arm64".into(), None),
            ("win32".into(), "AMD64".into(), None),
        ];
        Self::cross_product(&pys, &plats)
    }
}

/// Run `marker` against every env and return the subset where it's true.
/// An empty marker (i.e. no marker on the requirement) is treated as
/// "always true" and returns every env unchanged.
pub fn evaluate_marker_across_envs<'a>(
    marker: Option<&str>,
    envs: &'a EnvironmentSet,
) -> Result<Vec<&'a SupportedEnvironment>, IndexError> {
    let Some(marker) = marker else {
        return Ok(envs.as_slice().iter().collect());
    };
    let mut matched = Vec::with_capacity(envs.len());
    for env in envs.as_slice() {
        let marker_env = env.to_marker_env();
        let truth =
            markers::evaluate(marker, &marker_env).map_err(|err| IndexError::ParseError {
                url: "<universal lock marker>".into(),
                detail: format!(
                    "evaluating marker {marker:?} against {}: {err}",
                    env.label()
                ),
            })?;
        if truth {
            matched.push(env);
        }
    }
    Ok(matched)
}

/// True iff the marker is true for at least one supported environment. This
/// is the "include this requirement at all?" gate.
pub fn applies_anywhere(marker: Option<&str>, envs: &EnvironmentSet) -> Result<bool, IndexError> {
    Ok(!evaluate_marker_across_envs(marker, envs)?.is_empty())
}

/// True iff the marker is true for *every* supported environment. When
/// true, the lockfile entry is unconditional; when false, the entry needs
/// the `markers` field populated so sync can filter by current host.
pub fn applies_everywhere(marker: Option<&str>, envs: &EnvironmentSet) -> Result<bool, IndexError> {
    let matches = evaluate_marker_across_envs(marker, envs)?;
    Ok(matches.len() == envs.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(
        py: (u32, u32),
        sys: &str,
        mach: &str,
        libc: Option<LibcFamily>,
    ) -> SupportedEnvironment {
        SupportedEnvironment {
            python_version: PythonVersion::new(py.0, py.1, 0),
            sys_platform: sys.into(),
            platform_machine: mach.into(),
            libc,
        }
    }

    #[test]
    fn label_renders_libc_for_linux_and_none_elsewhere() {
        let linux = env((3, 12), "linux", "x86_64", Some(LibcFamily::Glibc));
        assert_eq!(linux.label(), "cpython-3.12-linux-x86_64-glibc");

        let mac = env((3, 12), "darwin", "arm64", None);
        assert_eq!(mac.label(), "cpython-3.12-darwin-arm64");

        let win = env((3, 11), "win32", "AMD64", None);
        assert_eq!(win.label(), "cpython-3.11-win32-AMD64");
    }

    #[test]
    fn to_marker_env_populates_derived_fields() {
        let e = env((3, 12), "linux", "x86_64", Some(LibcFamily::Glibc));
        let me = e.to_marker_env();
        assert_eq!(me.python_version, "3.12");
        assert_eq!(me.python_full_version, "3.12.0");
        assert_eq!(me.sys_platform, "linux");
        assert_eq!(me.platform_machine, "x86_64");
        assert_eq!(me.platform_system, "Linux");
        assert_eq!(me.os_name, "posix");

        let me_win = env((3, 12), "win32", "AMD64", None).to_marker_env();
        assert_eq!(me_win.os_name, "nt");
        assert_eq!(me_win.platform_system, "Windows");

        let me_mac = env((3, 12), "darwin", "arm64", None).to_marker_env();
        assert_eq!(me_mac.platform_system, "Darwin");
        assert_eq!(me_mac.os_name, "posix");
    }

    #[test]
    fn environment_set_dedupes_inserts() {
        let mut set = EnvironmentSet::new();
        let e1 = env((3, 12), "linux", "x86_64", Some(LibcFamily::Glibc));
        let e2 = env((3, 12), "linux", "x86_64", Some(LibcFamily::Glibc));
        set.insert(e1.clone());
        set.insert(e2);
        assert_eq!(set.len(), 1);
        assert_eq!(set.as_slice()[0], e1);
    }

    #[test]
    fn cross_product_yields_full_matrix() {
        let pys = [PythonVersion::new(3, 11, 0), PythonVersion::new(3, 12, 0)];
        let plats: Vec<(String, String, Option<LibcFamily>)> = vec![
            ("linux".into(), "x86_64".into(), Some(LibcFamily::Glibc)),
            ("darwin".into(), "arm64".into(), None),
        ];
        let set = EnvironmentSet::cross_product(&pys, &plats);
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn standard_matrix_has_twenty_entries() {
        let set = EnvironmentSet::standard_matrix();
        assert_eq!(set.len(), 20);
    }

    #[test]
    fn empty_marker_applies_to_every_env() {
        let set = EnvironmentSet::standard_matrix();
        let matches = evaluate_marker_across_envs(None, &set).unwrap();
        assert_eq!(matches.len(), set.len());
        assert!(applies_anywhere(None, &set).unwrap());
        assert!(applies_everywhere(None, &set).unwrap());
    }

    #[test]
    fn sys_platform_filter_subsets_envs() {
        let set = EnvironmentSet::standard_matrix();
        let matches = evaluate_marker_across_envs(Some("sys_platform == 'linux'"), &set).unwrap();
        // 4 python versions × 2 linux platforms = 8.
        assert_eq!(matches.len(), 8);
        assert!(matches.iter().all(|e| e.sys_platform == "linux"));

        assert!(applies_anywhere(Some("sys_platform == 'linux'"), &set).unwrap());
        assert!(!applies_everywhere(Some("sys_platform == 'linux'"), &set).unwrap());
    }

    #[test]
    fn python_version_floor_excludes_old_pythons() {
        let set = EnvironmentSet::standard_matrix();
        let matches = evaluate_marker_across_envs(Some("python_version >= '3.12'"), &set).unwrap();
        // 2 python versions (3.12, 3.13) × 5 platforms = 10.
        assert_eq!(matches.len(), 10);
        assert!(matches.iter().all(|e| e.python_version.minor >= 12));
    }

    #[test]
    fn combined_marker_intersection() {
        let set = EnvironmentSet::standard_matrix();
        let matches = evaluate_marker_across_envs(
            Some("sys_platform == 'darwin' and platform_machine == 'arm64'"),
            &set,
        )
        .unwrap();
        // 4 python versions, single platform combo = 4.
        assert_eq!(matches.len(), 4);
        for e in matches {
            assert_eq!(e.sys_platform, "darwin");
            assert_eq!(e.platform_machine, "arm64");
        }
    }

    #[test]
    fn never_true_marker_drops_all_envs() {
        let set = EnvironmentSet::standard_matrix();
        let matches = evaluate_marker_across_envs(Some("sys_platform == 'haiku'"), &set).unwrap();
        assert!(matches.is_empty());
        assert!(!applies_anywhere(Some("sys_platform == 'haiku'"), &set).unwrap());
    }

    #[test]
    fn malformed_marker_yields_parse_error() {
        let set = EnvironmentSet::standard_matrix();
        let err = evaluate_marker_across_envs(Some("python_version >= "), &set).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("evaluating marker"), "got: {msg}");
    }
}
