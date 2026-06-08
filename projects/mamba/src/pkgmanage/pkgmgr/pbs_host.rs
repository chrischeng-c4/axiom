// Host detection for PBS downloads (Tick 61).
//
// Tick 59's `pbs_url` turns a `(version, target, release_tag)` tuple
// into a download URL but takes the target as input. This module is
// the missing half: detect the current host's `(arch, os, libc)` and
// pair it with uv's default variant so callers can resolve "give me
// PBS for *this* machine" without hand-rolling the cfg matrix.
//
// Coverage:
//   * Arch       — cfg!(target_arch = "x86_64" | "aarch64" | "x86")
//   * OS         — cfg!(target_os  = "linux" | "macos" | "windows")
//   * Linux libc — cfg!(target_env = "gnu" | "musl") — fast, no IO.
//
// Anything outside that grid (FreeBSD, riscv, mips, etc.) returns
// `PbsHostError::UnsupportedHost` so callers can fall back to a system
// interpreter discovery path. Pure-data: no subprocess, no /proc reads.

use crate::pkgmanage::pkgmgr::pbs_url::{PbsArch, PbsLibc, PbsOs, PbsTarget};

/// Reasons host detection can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PbsHostError {
    UnsupportedHost { detail: String },
}

impl std::fmt::Display for PbsHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PbsHostError::UnsupportedHost { detail } => {
                write!(f, "unsupported host: {detail}")
            }
        }
    }
}

impl std::error::Error for PbsHostError {}

/// Detect the host architecture.
pub fn host_arch() -> Result<PbsArch, PbsHostError> {
    if cfg!(target_arch = "x86_64") {
        Ok(PbsArch::X86_64)
    } else if cfg!(target_arch = "aarch64") {
        Ok(PbsArch::Aarch64)
    } else if cfg!(target_arch = "x86") {
        Ok(PbsArch::I686)
    } else {
        Err(PbsHostError::UnsupportedHost {
            detail: format!("unknown target_arch: {}", std::env::consts::ARCH),
        })
    }
}

/// Detect the host operating system.
pub fn host_os() -> Result<PbsOs, PbsHostError> {
    if cfg!(target_os = "linux") {
        Ok(PbsOs::Linux)
    } else if cfg!(target_os = "macos") {
        Ok(PbsOs::Darwin)
    } else if cfg!(target_os = "windows") {
        Ok(PbsOs::Windows)
    } else {
        Err(PbsHostError::UnsupportedHost {
            detail: format!("unknown target_os: {}", std::env::consts::OS),
        })
    }
}

/// Detect the libc family. Returns `Some` only on Linux; `None` on
/// macOS / Windows where the question is meaningless.
pub fn host_libc(os: PbsOs) -> Result<Option<PbsLibc>, PbsHostError> {
    if os != PbsOs::Linux {
        return Ok(None);
    }
    if cfg!(target_env = "gnu") {
        Ok(Some(PbsLibc::Gnu))
    } else if cfg!(target_env = "musl") {
        Ok(Some(PbsLibc::Musl))
    } else {
        Err(PbsHostError::UnsupportedHost {
            detail: format!(
                "unknown target_env on Linux: {:?}",
                option_env!("CARGO_CFG_TARGET_ENV").unwrap_or("?")
            ),
        })
    }
}

/// Detect the host and return a default `PbsTarget` (uv's chosen
/// variant per platform). Callers that want a non-default variant
/// should compose `PbsTarget::new` directly.
pub fn detect_host_target() -> Result<PbsTarget, PbsHostError> {
    let arch = host_arch()?;
    let os = host_os()?;
    let libc = host_libc(os)?;
    let variant = PbsTarget::default_variant_for(os, libc);
    PbsTarget::new(arch, os, libc, variant).map_err(|e| PbsHostError::UnsupportedHost {
        detail: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_arch_is_one_of_supported() {
        let got = host_arch();
        if cfg!(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "x86"
        )) {
            assert!(got.is_ok(), "expected supported arch, got {got:?}");
        }
    }

    #[test]
    fn host_os_is_one_of_supported() {
        let got = host_os();
        if cfg!(any(target_os = "linux", target_os = "macos", target_os = "windows")) {
            assert!(got.is_ok(), "expected supported os, got {got:?}");
        }
    }

    #[test]
    fn libc_is_none_on_macos() {
        let got = host_libc(PbsOs::Darwin).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn libc_is_none_on_windows() {
        let got = host_libc(PbsOs::Windows).unwrap();
        assert!(got.is_none());
    }

    #[test]
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    fn libc_is_gnu_on_glibc_linux() {
        assert_eq!(host_libc(PbsOs::Linux).unwrap(), Some(PbsLibc::Gnu));
    }

    #[test]
    #[cfg(all(target_os = "linux", target_env = "musl"))]
    fn libc_is_musl_on_musl_linux() {
        assert_eq!(host_libc(PbsOs::Linux).unwrap(), Some(PbsLibc::Musl));
    }

    #[test]
    fn detect_host_target_returns_consistent_libc() {
        if let Ok(target) = detect_host_target() {
            match target.os {
                PbsOs::Linux => assert!(target.libc.is_some()),
                _ => assert!(target.libc.is_none()),
            }
        }
    }

    #[test]
    fn detect_host_target_default_variant_matches_table() {
        if let Ok(target) = detect_host_target() {
            assert_eq!(
                target.variant,
                PbsTarget::default_variant_for(target.os, target.libc)
            );
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn arch_is_x86_64_when_target_arch_says_so() {
        assert_eq!(host_arch().unwrap(), PbsArch::X86_64);
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn arch_is_aarch64_when_target_arch_says_so() {
        assert_eq!(host_arch().unwrap(), PbsArch::Aarch64);
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn arch_is_i686_when_target_arch_says_so() {
        assert_eq!(host_arch().unwrap(), PbsArch::I686);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn os_is_linux_when_target_os_says_so() {
        assert_eq!(host_os().unwrap(), PbsOs::Linux);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn os_is_darwin_when_target_os_says_so() {
        assert_eq!(host_os().unwrap(), PbsOs::Darwin);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn os_is_windows_when_target_os_says_so() {
        assert_eq!(host_os().unwrap(), PbsOs::Windows);
    }

    #[test]
    fn detect_host_target_round_trips_through_url_machinery() {
        use crate::pkgmanage::pkgmgr::pbs_url::PbsArtifact;
        use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;
        if let Ok(target) = detect_host_target() {
            let art = PbsArtifact {
                release_tag: "20240726".into(),
                version: PythonVersion::new(3, 12, 4),
                target,
            };
            let url = art.url();
            assert!(
                url.contains(&target.target_triple()),
                "url {url} must contain target triple {}",
                target.target_triple()
            );
            assert!(url.contains(target.variant_tag()));
            assert!(url.contains("cpython-3.12.4+20240726-"));
        }
    }

    #[test]
    fn unsupported_host_error_displays_with_detail() {
        let err = PbsHostError::UnsupportedHost {
            detail: "some bogus arch".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unsupported host"));
        assert!(msg.contains("some bogus arch"));
    }
}
