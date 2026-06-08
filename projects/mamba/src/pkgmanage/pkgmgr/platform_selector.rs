// Cross-platform wheel-tag selector (Tick 119).
//
// `tags::TagSelector::current_host` builds a selector from the running
// process's `cfg!(target_os, target_arch)` + a `MAMBA_PYTHON_TAG` env
// var. That's fine for `mamba sync` invocations from a developer's
// laptop, but it's wrong for cross-platform lockfile resolution:
// `mamba lock --platform linux_x86_64` running on a macOS dev box must
// pin Linux wheels, not macOS wheels.
//
// This module builds a `TagSelector` from an explicitly-named
// `SupportedEnvironment` (already typed in `platforms`), independent of
// the host process. The resulting selector feeds straight into
// `wheel_picker::pick_best_wheel` for cross-platform resolution.
//
// Coverage matches what uv emits for the four production platforms:
//   * Linux x86_64 / aarch64 / armv7l (glibc, musl)
//   * macOS x86_64 / arm64 (with universal2 fallback)
//   * Windows AMD64 / arm64
//
// PEP 600 / 656 / 425 tag descents (newest manylinux version listed
// first) match what pip / uv generate so wheels published with the same
// tag set sort identically here.

use crate::pkgmanage::pkgmgr::platforms::{LibcFamily, SupportedEnvironment};
use crate::pkgmanage::pkgmgr::tags::TagSelector;
use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;

/// Build a `TagSelector` from an explicitly-named target environment.
///
/// Caller is expected to have constructed `env` from CLI input or a
/// pyproject `[tool.mamba.lock] targets` table; this function does
/// no host probing whatsoever.
pub fn selector_for(env: &SupportedEnvironment) -> TagSelector {
    let python = python_tag_ladder(&env.python_version);
    let abi = abi_tag_ladder(&env.python_version);
    let platform = platform_tag_ladder(env);
    TagSelector {
        python,
        abi,
        platform,
    }
}

/// Convenience: build a selector from a "platform key" string of the
/// form `cpython-3.12-linux-x86_64-glibc`. The format matches
/// `SupportedEnvironment::label` so labels round-trip.
pub fn selector_for_label(label: &str) -> Option<TagSelector> {
    let env = parse_platform_label(label)?;
    Some(selector_for(&env))
}

/// Parse a `SupportedEnvironment::label` back into a structured form.
/// Returns `None` for malformed inputs. The format is intentionally
/// strict so typo'd labels surface as errors at lock time rather than
/// silently producing the wrong wheel set.
pub fn parse_platform_label(label: &str) -> Option<SupportedEnvironment> {
    // `cpython-3.12-linux-x86_64-glibc` → 5 segments
    // `cpython-3.12-darwin-arm64`        → 4 segments (no libc)
    let parts: Vec<&str> = label.split('-').collect();
    if !matches!(parts.first(), Some(&"cpython")) {
        return None;
    }
    if parts.len() < 4 {
        return None;
    }
    let pyver = parts[1];
    let sys_platform = parts[2].to_string();
    let platform_machine = parts[3].to_string();
    let libc = match (sys_platform.as_str(), parts.get(4).copied()) {
        ("linux", Some("glibc")) => Some(LibcFamily::Glibc),
        ("linux", Some("musl")) => Some(LibcFamily::Musl),
        ("linux", _) => return None, // libc segment required on linux
        (_, None) => None,
        (_, Some(_)) => return None, // libc tag only legal on linux
    };
    let python_version = parse_python_version(pyver)?;
    Some(SupportedEnvironment {
        python_version,
        sys_platform,
        platform_machine,
        libc,
    })
}

fn parse_python_version(s: &str) -> Option<PythonVersion> {
    let parts: Vec<&str> = s.split('.').collect();
    let major: u32 = parts.first()?.parse().ok()?;
    let minor: u32 = parts.get(1)?.parse().ok()?;
    let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
    Some(PythonVersion {
        major,
        minor,
        patch,
    })
}

fn python_tag_ladder(v: &PythonVersion) -> Vec<String> {
    let py_full = format!("cp{}{}", v.major, v.minor);
    let py_minor = format!("py{}{}", v.major, v.minor);
    let py_major = format!("py{}", v.major);
    vec![
        py_full.clone(),
        format!("cp{}", v.major),
        py_minor,
        py_major.clone(),
        "py2.py3".to_string(),
        "py".to_string(),
    ]
}

fn abi_tag_ladder(v: &PythonVersion) -> Vec<String> {
    vec![
        format!("cp{}{}", v.major, v.minor),
        "abi3".to_string(),
        "none".to_string(),
    ]
}

fn platform_tag_ladder(env: &SupportedEnvironment) -> Vec<String> {
    match env.sys_platform.as_str() {
        "linux" => linux_platform_ladder(&env.platform_machine, env.libc),
        "darwin" => macos_platform_ladder(&env.platform_machine),
        "win32" => windows_platform_ladder(&env.platform_machine),
        _ => vec!["any".to_string()],
    }
}

fn linux_platform_ladder(arch: &str, libc: Option<LibcFamily>) -> Vec<String> {
    let mut out = Vec::new();
    match libc {
        Some(LibcFamily::Glibc) => {
            // PEP 600 manylinux_2_X_<arch> descending. We bracket between
            // 2_5 (manylinux1 era) and 2_38 — covers every released
            // CPython 3.x wheel on PyPI as of this writing.
            for minor in (5..=38).rev() {
                out.push(format!("manylinux_2_{minor}_{arch}"));
            }
            // PEP 513 / 599 legacy aliases.
            match arch {
                "x86_64" => {
                    out.push("manylinux2014_x86_64".to_string());
                    out.push("manylinux2010_x86_64".to_string());
                    out.push("manylinux1_x86_64".to_string());
                }
                "i686" => {
                    out.push("manylinux2014_i686".to_string());
                    out.push("manylinux2010_i686".to_string());
                    out.push("manylinux1_i686".to_string());
                }
                "aarch64" => {
                    out.push("manylinux2014_aarch64".to_string());
                }
                "armv7l" => {
                    out.push("manylinux2014_armv7l".to_string());
                }
                "ppc64le" => {
                    out.push("manylinux2014_ppc64le".to_string());
                }
                "s390x" => {
                    out.push("manylinux2014_s390x".to_string());
                }
                _ => {}
            }
        }
        Some(LibcFamily::Musl) => {
            // PEP 656 musllinux_1_X_<arch>, descending from 1_2 (newest
            // released) down to 1_0.
            for minor in (0..=3).rev() {
                out.push(format!("musllinux_1_{minor}_{arch}"));
            }
        }
        None => {}
    }
    out.push(format!("linux_{arch}"));
    out.push("any".to_string());
    out
}

fn macos_platform_ladder(arch: &str) -> Vec<String> {
    let mut out = Vec::new();
    match arch {
        "arm64" => {
            // macOS arm64 — 11.0 is the floor (Apple Silicon launch).
            // Descend from 14_x down to 11_0; also accept universal2.
            for major in (11..=15).rev() {
                for minor in (0..=15).rev() {
                    out.push(format!("macosx_{major}_{minor}_arm64"));
                    out.push(format!("macosx_{major}_{minor}_universal2"));
                }
            }
        }
        "x86_64" => {
            // macOS Intel — back to 10.9 (the SDK floor pip/uv ship).
            for major in (10..=15).rev() {
                let max_minor = if major == 10 { 16 } else { 15 };
                for minor in (0..=max_minor).rev() {
                    if major == 10 && minor < 9 {
                        break;
                    }
                    out.push(format!("macosx_{major}_{minor}_x86_64"));
                    out.push(format!("macosx_{major}_{minor}_universal2"));
                    out.push(format!("macosx_{major}_{minor}_intel"));
                }
            }
        }
        _ => {}
    }
    out.push("any".to_string());
    out
}

fn windows_platform_ladder(arch: &str) -> Vec<String> {
    let mut out = Vec::new();
    match arch {
        "AMD64" | "x86_64" => out.push("win_amd64".to_string()),
        "x86" | "i686" => out.push("win32".to_string()),
        "ARM64" | "aarch64" => out.push("win_arm64".to_string()),
        _ => {}
    }
    out.push("any".to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::wheel_picker::pick_best_wheel;

    fn cpython(major: u32, minor: u32) -> PythonVersion {
        PythonVersion {
            major,
            minor,
            patch: 0,
        }
    }

    fn linux_x86_64_glibc_cpython(minor: u32) -> SupportedEnvironment {
        SupportedEnvironment {
            python_version: cpython(3, minor),
            sys_platform: "linux".to_string(),
            platform_machine: "x86_64".to_string(),
            libc: Some(LibcFamily::Glibc),
        }
    }

    fn macos_arm64_cpython(minor: u32) -> SupportedEnvironment {
        SupportedEnvironment {
            python_version: cpython(3, minor),
            sys_platform: "darwin".to_string(),
            platform_machine: "arm64".to_string(),
            libc: None,
        }
    }

    fn windows_amd64_cpython(minor: u32) -> SupportedEnvironment {
        SupportedEnvironment {
            python_version: cpython(3, minor),
            sys_platform: "win32".to_string(),
            platform_machine: "AMD64".to_string(),
            libc: None,
        }
    }

    #[test]
    fn linux_glibc_cp312_picks_linux_wheel() {
        let env = linux_x86_64_glibc_cpython(12);
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp312-cp312-manylinux_2_17_x86_64.whl".to_string(),
            "pkg-1.0-cp312-cp312-win_amd64.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-cp312-cp312-manylinux_2_17_x86_64.whl"
        );
    }

    #[test]
    fn macos_arm64_cp312_picks_arm64_wheel() {
        let env = macos_arm64_cpython(12);
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp312-cp312-macosx_11_0_x86_64.whl".to_string(),
            "pkg-1.0-cp312-cp312-manylinux_2_17_x86_64.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl");
    }

    #[test]
    fn windows_amd64_cp311_picks_win_amd64_wheel() {
        let env = windows_amd64_cpython(11);
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp311-cp311-win_amd64.whl".to_string(),
            "pkg-1.0-cp311-cp311-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp311-cp311-manylinux_2_17_x86_64.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp311-cp311-win_amd64.whl");
    }

    #[test]
    fn macos_arm64_accepts_universal2() {
        let env = macos_arm64_cpython(12);
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_universal2.whl".to_string(),
            // Pure-python fallback should NOT beat universal2.
            "pkg-1.0-py3-none-any.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-cp312-cp312-macosx_11_0_universal2.whl"
        );
    }

    #[test]
    fn linux_musl_picks_musllinux_wheel() {
        let env = SupportedEnvironment {
            python_version: cpython(3, 12),
            sys_platform: "linux".to_string(),
            platform_machine: "x86_64".to_string(),
            libc: Some(LibcFamily::Musl),
        };
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp312-cp312-musllinux_1_2_x86_64.whl".to_string(),
            "pkg-1.0-cp312-cp312-manylinux_2_17_x86_64.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-cp312-cp312-musllinux_1_2_x86_64.whl"
        );
    }

    #[test]
    fn linux_aarch64_picks_aarch64_wheel() {
        let env = SupportedEnvironment {
            python_version: cpython(3, 12),
            sys_platform: "linux".to_string(),
            platform_machine: "aarch64".to_string(),
            libc: Some(LibcFamily::Glibc),
        };
        let s = selector_for(&env);
        let files = vec![
            "pkg-1.0-cp312-cp312-manylinux_2_17_aarch64.whl".to_string(),
            "pkg-1.0-cp312-cp312-manylinux_2_17_x86_64.whl".to_string(),
        ];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-cp312-cp312-manylinux_2_17_aarch64.whl"
        );
    }

    #[test]
    fn any_wheel_is_last_resort_fallback() {
        let env = linux_x86_64_glibc_cpython(12);
        let s = selector_for(&env);
        let files = vec!["pkg-1.0-py3-none-any.whl".to_string()];
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-py3-none-any.whl");
    }

    #[test]
    fn label_round_trips_linux_glibc() {
        let env = linux_x86_64_glibc_cpython(12);
        let label = env.label();
        let parsed = parse_platform_label(&label).unwrap();
        assert_eq!(parsed, env);
    }

    #[test]
    fn label_round_trips_macos_arm64() {
        let env = macos_arm64_cpython(12);
        let label = env.label();
        let parsed = parse_platform_label(&label).unwrap();
        assert_eq!(parsed, env);
    }

    #[test]
    fn label_round_trips_windows_amd64() {
        let env = windows_amd64_cpython(11);
        let label = env.label();
        let parsed = parse_platform_label(&label).unwrap();
        assert_eq!(parsed, env);
    }

    #[test]
    fn label_round_trips_linux_musl() {
        let env = SupportedEnvironment {
            python_version: cpython(3, 12),
            sys_platform: "linux".to_string(),
            platform_machine: "x86_64".to_string(),
            libc: Some(LibcFamily::Musl),
        };
        let parsed = parse_platform_label(&env.label()).unwrap();
        assert_eq!(parsed, env);
    }

    #[test]
    fn label_parse_rejects_non_cpython_prefix() {
        assert!(parse_platform_label("pypy-3.12-linux-x86_64-glibc").is_none());
    }

    #[test]
    fn label_parse_rejects_linux_without_libc() {
        assert!(parse_platform_label("cpython-3.12-linux-x86_64").is_none());
    }

    #[test]
    fn label_parse_rejects_libc_on_non_linux() {
        assert!(parse_platform_label("cpython-3.12-darwin-arm64-glibc").is_none());
    }

    #[test]
    fn selector_for_label_convenience() {
        let s = selector_for_label("cpython-3.12-linux-x86_64-glibc").unwrap();
        // The python ladder should start at cp312.
        assert_eq!(s.python[0], "cp312");
        // The platform ladder should contain a manylinux_2_x_x86_64 prefix.
        assert!(s.platform.iter().any(|p| p == "manylinux_2_17_x86_64"));
    }

    #[test]
    fn selector_for_label_rejects_garbage() {
        assert!(selector_for_label("complete-garbage").is_none());
    }

    #[test]
    fn realistic_numpy_release_resolves_per_platform() {
        // Same numpy 1.26 file list as wheel_picker.rs uses, but now
        // picked through the explicit selector — each platform picks
        // the wheel that matches its arch + python.
        let files = vec![
            "numpy-1.26.0-cp310-cp310-macosx_11_0_arm64.whl".to_string(),
            "numpy-1.26.0-cp310-cp310-manylinux_2_17_x86_64.manylinux2014_x86_64.whl".to_string(),
            "numpy-1.26.0-cp310-cp310-win_amd64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-win_amd64.whl".to_string(),
        ];

        let linux = selector_for(&linux_x86_64_glibc_cpython(12));
        let macos = selector_for(&macos_arm64_cpython(12));
        let win = selector_for(&windows_amd64_cpython(12));

        let linux_pick = pick_best_wheel(&files, &linux).unwrap();
        let macos_pick = pick_best_wheel(&files, &macos).unwrap();
        let win_pick = pick_best_wheel(&files, &win).unwrap();

        assert!(linux_pick.filename.contains("manylinux"));
        assert!(linux_pick.filename.contains("cp312"));
        assert!(macos_pick.filename.contains("macosx"));
        assert!(macos_pick.filename.contains("arm64"));
        assert!(macos_pick.filename.contains("cp312"));
        assert_eq!(win_pick.filename, "numpy-1.26.0-cp312-cp312-win_amd64.whl");
    }
}
