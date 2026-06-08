// python-build-standalone download URL templater (Tick 59).
//
// uv ships managed Python builds by pulling pre-built tarballs from
// the `astral-sh/python-build-standalone` GitHub release archive. The
// download URL follows a fixed pattern:
//
//   https://github.com/astral-sh/python-build-standalone/releases/
//     download/<release_tag>/
//     cpython-<py_version>+<release_tag>-<target_triple>-<variant>-full.tar.zst
//
// This module turns a `(version, target, release_tag)` tuple into that
// URL. Pure-data: no I/O, no fallback to system Python, no download.
//
// Coverage:
//   * Archs:    x86_64, aarch64, i686
//   * OSes:     Linux (gnu | musl), macOS, Windows (msvc)
//   * Variants: pgo+lto, pgo, lto, noopt, debug
//
// `PbsTarget::default_variant_for(arch, os)` returns the variant uv
// picks by default for a given target — for example pgo+lto on Linux
// glibc / macOS, pgo on Windows, noopt on musl (the pgo+lto build is
// not produced for musl).

use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;

const PBS_BASE: &str =
    "https://github.com/astral-sh/python-build-standalone/releases/download";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbsArch {
    X86_64,
    Aarch64,
    I686,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbsOs {
    Linux,
    Darwin,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbsLibc {
    Gnu,
    Musl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbsVariant {
    PgoLto,
    Pgo,
    Lto,
    Noopt,
    Debug,
}

/// One PBS target: arch + OS + Linux libc + build variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PbsTarget {
    pub arch: PbsArch,
    pub os: PbsOs,
    /// `Some` only on Linux. macOS / Windows ignore this.
    pub libc: Option<PbsLibc>,
    pub variant: PbsVariant,
}

/// Errors raised when a target is internally inconsistent (e.g. macOS
/// with a libc set, or musl with the pgo+lto variant — that build is
/// not produced).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PbsUrlError {
    LibcOnNonLinux,
    MissingLinuxLibc,
    UnsupportedVariantForTarget {
        detail: String,
    },
}

impl std::fmt::Display for PbsUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PbsUrlError::LibcOnNonLinux => {
                write!(f, "libc set on non-Linux target")
            }
            PbsUrlError::MissingLinuxLibc => {
                write!(f, "Linux target missing libc selection")
            }
            PbsUrlError::UnsupportedVariantForTarget { detail } => {
                write!(f, "unsupported variant: {detail}")
            }
        }
    }
}

impl std::error::Error for PbsUrlError {}

impl PbsTarget {
    /// Compose a target. Returns an error when the libc / OS / variant
    /// combination cannot exist as a real PBS artifact.
    pub fn new(
        arch: PbsArch,
        os: PbsOs,
        libc: Option<PbsLibc>,
        variant: PbsVariant,
    ) -> Result<Self, PbsUrlError> {
        match (os, libc) {
            (PbsOs::Linux, None) => return Err(PbsUrlError::MissingLinuxLibc),
            (PbsOs::Linux, Some(_)) => {}
            (_, Some(_)) => return Err(PbsUrlError::LibcOnNonLinux),
            (_, None) => {}
        }
        if let (PbsOs::Linux, Some(PbsLibc::Musl), PbsVariant::PgoLto) = (os, libc, variant) {
            return Err(PbsUrlError::UnsupportedVariantForTarget {
                detail: "musl does not ship a pgo+lto build".into(),
            });
        }
        Ok(Self {
            arch,
            os,
            libc,
            variant,
        })
    }

    /// uv's default variant for an `(arch, os, libc)` triple. Used when
    /// the caller has not pinned a specific build.
    pub fn default_variant_for(os: PbsOs, libc: Option<PbsLibc>) -> PbsVariant {
        match (os, libc) {
            (PbsOs::Linux, Some(PbsLibc::Musl)) => PbsVariant::Noopt,
            (PbsOs::Windows, _) => PbsVariant::Pgo,
            _ => PbsVariant::PgoLto,
        }
    }

    /// Render the Rust-style target triple used in PBS filenames.
    pub fn target_triple(&self) -> String {
        let arch = match self.arch {
            PbsArch::X86_64 => "x86_64",
            PbsArch::Aarch64 => "aarch64",
            PbsArch::I686 => "i686",
        };
        let os_part = match (self.os, self.libc) {
            (PbsOs::Darwin, _) => "apple-darwin".to_string(),
            (PbsOs::Windows, _) => "pc-windows-msvc".to_string(),
            (PbsOs::Linux, Some(PbsLibc::Gnu)) => "unknown-linux-gnu".to_string(),
            (PbsOs::Linux, Some(PbsLibc::Musl)) => "unknown-linux-musl".to_string(),
            (PbsOs::Linux, None) => "unknown-linux".to_string(),
        };
        format!("{arch}-{os_part}")
    }

    /// PBS variant tag as it appears in the artifact filename.
    pub fn variant_tag(&self) -> &'static str {
        match self.variant {
            PbsVariant::PgoLto => "pgo+lto",
            PbsVariant::Pgo => "pgo",
            PbsVariant::Lto => "lto",
            PbsVariant::Noopt => "noopt",
            PbsVariant::Debug => "debug",
        }
    }
}

/// One downloadable PBS artifact: version + target + release tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PbsArtifact {
    /// Release tag from the upstream release archive, e.g. `20240726`.
    pub release_tag: String,
    pub version: PythonVersion,
    pub target: PbsTarget,
}

impl PbsArtifact {
    /// Filename component without the directory path or scheme.
    pub fn filename(&self) -> String {
        format!(
            "cpython-{}+{}-{}-{}-full.tar.zst",
            self.version,
            self.release_tag,
            self.target.target_triple(),
            self.target.variant_tag(),
        )
    }

    /// Full GitHub download URL.
    pub fn url(&self) -> String {
        format!("{}/{}/{}", PBS_BASE, self.release_tag, self.filename())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pv(major: u32, minor: u32, patch: u32) -> PythonVersion {
        PythonVersion::new(major, minor, patch)
    }

    #[test]
    fn linux_glibc_x86_64_pgo_lto_url() {
        let target =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Linux, Some(PbsLibc::Gnu), PbsVariant::PgoLto)
                .unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert_eq!(
            art.filename(),
            "cpython-3.12.4+20240726-x86_64-unknown-linux-gnu-pgo+lto-full.tar.zst"
        );
        assert_eq!(
            art.url(),
            "https://github.com/astral-sh/python-build-standalone/releases/download/20240726/cpython-3.12.4+20240726-x86_64-unknown-linux-gnu-pgo+lto-full.tar.zst"
        );
    }

    #[test]
    fn macos_aarch64_pgo_lto_filename() {
        let target =
            PbsTarget::new(PbsArch::Aarch64, PbsOs::Darwin, None, PbsVariant::PgoLto).unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert_eq!(
            art.filename(),
            "cpython-3.12.4+20240726-aarch64-apple-darwin-pgo+lto-full.tar.zst"
        );
    }

    #[test]
    fn windows_x86_64_pgo_filename() {
        let target = PbsTarget::new(PbsArch::X86_64, PbsOs::Windows, None, PbsVariant::Pgo)
            .unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert_eq!(
            art.filename(),
            "cpython-3.12.4+20240726-x86_64-pc-windows-msvc-pgo-full.tar.zst"
        );
    }

    #[test]
    fn linux_musl_noopt_filename() {
        let target =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Linux, Some(PbsLibc::Musl), PbsVariant::Noopt)
                .unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert_eq!(
            art.filename(),
            "cpython-3.12.4+20240726-x86_64-unknown-linux-musl-noopt-full.tar.zst"
        );
    }

    #[test]
    fn linux_i686_glibc_lto_filename() {
        let target =
            PbsTarget::new(PbsArch::I686, PbsOs::Linux, Some(PbsLibc::Gnu), PbsVariant::Lto)
                .unwrap();
        let art = PbsArtifact {
            release_tag: "20240909".into(),
            version: pv(3, 11, 9),
            target,
        };
        assert_eq!(
            art.filename(),
            "cpython-3.11.9+20240909-i686-unknown-linux-gnu-lto-full.tar.zst"
        );
    }

    #[test]
    fn debug_variant_tag_in_filename() {
        let target =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Linux, Some(PbsLibc::Gnu), PbsVariant::Debug)
                .unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert!(art.filename().contains("-debug-"));
    }

    #[test]
    fn rejects_libc_on_macos() {
        let err =
            PbsTarget::new(PbsArch::Aarch64, PbsOs::Darwin, Some(PbsLibc::Gnu), PbsVariant::PgoLto)
                .unwrap_err();
        assert_eq!(err, PbsUrlError::LibcOnNonLinux);
    }

    #[test]
    fn rejects_libc_on_windows() {
        let err =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Windows, Some(PbsLibc::Gnu), PbsVariant::Pgo)
                .unwrap_err();
        assert_eq!(err, PbsUrlError::LibcOnNonLinux);
    }

    #[test]
    fn rejects_linux_without_libc() {
        let err =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Linux, None, PbsVariant::PgoLto).unwrap_err();
        assert_eq!(err, PbsUrlError::MissingLinuxLibc);
    }

    #[test]
    fn rejects_musl_pgo_lto_combination() {
        let err = PbsTarget::new(
            PbsArch::X86_64,
            PbsOs::Linux,
            Some(PbsLibc::Musl),
            PbsVariant::PgoLto,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            PbsUrlError::UnsupportedVariantForTarget { .. }
        ));
    }

    #[test]
    fn default_variant_is_pgo_lto_on_linux_glibc() {
        assert_eq!(
            PbsTarget::default_variant_for(PbsOs::Linux, Some(PbsLibc::Gnu)),
            PbsVariant::PgoLto
        );
    }

    #[test]
    fn default_variant_is_pgo_lto_on_macos() {
        assert_eq!(
            PbsTarget::default_variant_for(PbsOs::Darwin, None),
            PbsVariant::PgoLto
        );
    }

    #[test]
    fn default_variant_is_pgo_on_windows() {
        assert_eq!(
            PbsTarget::default_variant_for(PbsOs::Windows, None),
            PbsVariant::Pgo
        );
    }

    #[test]
    fn default_variant_is_noopt_on_linux_musl() {
        assert_eq!(
            PbsTarget::default_variant_for(PbsOs::Linux, Some(PbsLibc::Musl)),
            PbsVariant::Noopt
        );
    }

    #[test]
    fn target_triples_round_trip_through_constants() {
        let cases = [
            (
                PbsTarget::new(
                    PbsArch::X86_64,
                    PbsOs::Linux,
                    Some(PbsLibc::Gnu),
                    PbsVariant::PgoLto,
                )
                .unwrap(),
                "x86_64-unknown-linux-gnu",
            ),
            (
                PbsTarget::new(
                    PbsArch::Aarch64,
                    PbsOs::Linux,
                    Some(PbsLibc::Gnu),
                    PbsVariant::PgoLto,
                )
                .unwrap(),
                "aarch64-unknown-linux-gnu",
            ),
            (
                PbsTarget::new(
                    PbsArch::Aarch64,
                    PbsOs::Linux,
                    Some(PbsLibc::Musl),
                    PbsVariant::Noopt,
                )
                .unwrap(),
                "aarch64-unknown-linux-musl",
            ),
            (
                PbsTarget::new(PbsArch::Aarch64, PbsOs::Darwin, None, PbsVariant::PgoLto).unwrap(),
                "aarch64-apple-darwin",
            ),
            (
                PbsTarget::new(PbsArch::X86_64, PbsOs::Darwin, None, PbsVariant::PgoLto).unwrap(),
                "x86_64-apple-darwin",
            ),
            (
                PbsTarget::new(PbsArch::I686, PbsOs::Windows, None, PbsVariant::Pgo).unwrap(),
                "i686-pc-windows-msvc",
            ),
        ];
        for (target, expected) in cases {
            assert_eq!(target.target_triple(), expected);
        }
    }

    #[test]
    fn variant_tags_match_pbs_naming() {
        let cases = [
            (PbsVariant::PgoLto, "pgo+lto"),
            (PbsVariant::Pgo, "pgo"),
            (PbsVariant::Lto, "lto"),
            (PbsVariant::Noopt, "noopt"),
            (PbsVariant::Debug, "debug"),
        ];
        for (variant, expected) in cases {
            let target = PbsTarget {
                arch: PbsArch::X86_64,
                os: PbsOs::Linux,
                libc: Some(PbsLibc::Gnu),
                variant,
            };
            assert_eq!(target.variant_tag(), expected);
        }
    }

    #[test]
    fn url_starts_with_pbs_base_and_release_tag() {
        let target =
            PbsTarget::new(PbsArch::X86_64, PbsOs::Linux, Some(PbsLibc::Gnu), PbsVariant::PgoLto)
                .unwrap();
        let art = PbsArtifact {
            release_tag: "20240726".into(),
            version: pv(3, 12, 4),
            target,
        };
        assert!(art.url().starts_with(
            "https://github.com/astral-sh/python-build-standalone/releases/download/20240726/"
        ));
    }

    #[test]
    fn three_zero_patch_version_renders_explicit_zero() {
        let target =
            PbsTarget::new(PbsArch::Aarch64, PbsOs::Darwin, None, PbsVariant::PgoLto).unwrap();
        let art = PbsArtifact {
            release_tag: "20240909".into(),
            version: pv(3, 13, 0),
            target,
        };
        assert!(art.filename().contains("cpython-3.13.0+20240909-"));
    }
}
