// pep660.rs — classify an unpacked wheel as a PEP 660 editable install.
//
// PEP 660 was deliberately minimal: it does not mandate a single shape
// for the redirection mechanism. In practice three patterns dominate
// across the major build backends (setuptools, hatchling, pdm-backend,
// flit-core, poetry-core):
//
//   1. Strict / `.pth` redirect — the wheel ships one or more `.pth`
//      files at the install root containing absolute paths to the
//      source tree. site.py processes these on interpreter start; no
//      import hook is needed.
//
//   2. Finder redirect — the wheel ships `__editable__*.py` modules
//      plus a `__editable__*.pth` file that imports the finder at
//      site-processing time. The finder is a MetaPathFinder that
//      resolves imports lazily to the source tree.
//
//   3. NotEditable — a regular wheel.
//
// This classifier is purely structural: given the list of paths inside
// the unpacked wheel, decide which shape applies. The result drives
// downstream behaviour in `installer.rs` (which must avoid byte-
// compiling editable shims) and `tree.rs` (which must skip them when
// summarizing dependencies).
//
// We deliberately accept `&[&str]` rather than reading from disk so
// the classifier can be exercised against in-memory wheel listings
// (used during PEP 658 metadata-only fetches and in unit tests).

/// Result of classifying an unpacked wheel against PEP 660.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditableKind {
    /// Standard (non-editable) wheel.
    NotEditable,
    /// PEP 660 `.pth` redirect only — site.py handles the redirection.
    PthRedirect {
        /// `.pth` files at the install root that drive the redirect.
        pth_files: Vec<String>,
    },
    /// PEP 660 finder-based redirect — `__editable__*.py` modules
    /// install a MetaPathFinder at site-processing time.
    FinderRedirect {
        /// `__editable__*.py` modules that contribute the finder.
        finder_modules: Vec<String>,
        /// Accompanying `.pth` files that trigger the finder.
        pth_files: Vec<String>,
    },
}

impl EditableKind {
    /// Convenience: is this wheel editable (either flavour)?
    pub fn is_editable(&self) -> bool {
        !matches!(self, EditableKind::NotEditable)
    }
}

/// Classify the wheel from its path listing. Paths are relative to
/// the install root (e.g. `mypkg/__init__.py`, `mypkg-1.0.dist-info/RECORD`).
pub fn classify_editable(paths: &[&str]) -> EditableKind {
    let mut pth_files = Vec::new();
    let mut finder_modules = Vec::new();

    for raw in paths {
        // Only consider files at the install root — `.pth` files inside
        // package directories are package data, not site-config.
        let path = *raw;
        if path.contains('/') || path.contains('\\') {
            continue;
        }
        if is_finder_module(path) {
            finder_modules.push(path.to_string());
        } else if path.ends_with(".pth") {
            pth_files.push(path.to_string());
        }
    }

    if !finder_modules.is_empty() {
        finder_modules.sort();
        pth_files.sort();
        EditableKind::FinderRedirect {
            finder_modules,
            pth_files,
        }
    } else if !pth_files.is_empty() {
        pth_files.sort();
        EditableKind::PthRedirect { pth_files }
    } else {
        EditableKind::NotEditable
    }
}

fn is_finder_module(name: &str) -> bool {
    // setuptools emits `__editable___<dist>_<ver>_finder.py`.
    // hatchling emits `__editable__<dist>.py`.
    // Both share the `__editable__` prefix and `.py` suffix.
    name.starts_with("__editable__") && name.ends_with(".py")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(paths: &[&str]) -> EditableKind {
        classify_editable(paths)
    }

    #[test]
    fn empty_listing_is_not_editable() {
        assert_eq!(classify(&[]), EditableKind::NotEditable);
    }

    #[test]
    fn regular_wheel_is_not_editable() {
        let paths = [
            "mypkg/__init__.py",
            "mypkg/core.py",
            "mypkg-1.0.dist-info/METADATA",
            "mypkg-1.0.dist-info/WHEEL",
            "mypkg-1.0.dist-info/RECORD",
        ];
        assert_eq!(classify(&paths), EditableKind::NotEditable);
    }

    #[test]
    fn pth_redirect_detected() {
        let paths = [
            "_mypkg.pth",
            "mypkg-1.0.dist-info/METADATA",
            "mypkg-1.0.dist-info/WHEEL",
            "mypkg-1.0.dist-info/RECORD",
        ];
        assert_eq!(
            classify(&paths),
            EditableKind::PthRedirect {
                pth_files: vec!["_mypkg.pth".to_string()],
            }
        );
    }

    #[test]
    fn multiple_pth_files_collected_sorted() {
        let paths = ["z.pth", "a.pth", "m.pth"];
        let kind = classify(&paths);
        match kind {
            EditableKind::PthRedirect { pth_files } => {
                assert_eq!(pth_files, vec!["a.pth", "m.pth", "z.pth"]);
            }
            other => panic!("expected PthRedirect, got {other:?}"),
        }
    }

    #[test]
    fn hatchling_style_finder_detected() {
        // hatchling: __editable__<dist>.py + __editable__<dist>.pth
        let paths = [
            "__editable__mypkg.py",
            "__editable__mypkg.pth",
            "mypkg-1.0.dist-info/METADATA",
            "mypkg-1.0.dist-info/WHEEL",
            "mypkg-1.0.dist-info/RECORD",
        ];
        assert_eq!(
            classify(&paths),
            EditableKind::FinderRedirect {
                finder_modules: vec!["__editable__mypkg.py".to_string()],
                pth_files: vec!["__editable__mypkg.pth".to_string()],
            }
        );
    }

    #[test]
    fn setuptools_style_finder_detected() {
        // setuptools: __editable___<dist>_<ver>_finder.py + __editable__.<dist>-<ver>.pth
        let paths = [
            "__editable___mypkg_1_0_finder.py",
            "__editable__.mypkg-1.0.pth",
            "mypkg-1.0.dist-info/METADATA",
        ];
        let kind = classify(&paths);
        assert!(matches!(kind, EditableKind::FinderRedirect { .. }));
        match kind {
            EditableKind::FinderRedirect { finder_modules, pth_files } => {
                assert_eq!(finder_modules, vec!["__editable___mypkg_1_0_finder.py"]);
                assert_eq!(pth_files, vec!["__editable__.mypkg-1.0.pth"]);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn finder_wins_when_both_present() {
        // Some backends ship a finder module without an accompanying
        // `.pth` (e.g. when site.py is bypassed). The classifier still
        // reports FinderRedirect — finder modules are the strong signal.
        let paths = ["__editable__mypkg.py", "regular.pth"];
        match classify(&paths) {
            EditableKind::FinderRedirect { finder_modules, pth_files } => {
                assert_eq!(finder_modules, vec!["__editable__mypkg.py"]);
                assert_eq!(pth_files, vec!["regular.pth"]);
            }
            other => panic!("expected FinderRedirect, got {other:?}"),
        }
    }

    #[test]
    fn finder_redirect_with_no_pth() {
        let paths = ["__editable__mypkg.py"];
        match classify(&paths) {
            EditableKind::FinderRedirect { finder_modules, pth_files } => {
                assert_eq!(finder_modules, vec!["__editable__mypkg.py"]);
                assert!(pth_files.is_empty());
            }
            other => panic!("expected FinderRedirect, got {other:?}"),
        }
    }

    #[test]
    fn pth_inside_package_dir_is_ignored() {
        // Some packages ship `.pth` files as PACKAGE DATA. Those live
        // under `pkgname/...` and are NOT site-config files.
        let paths = [
            "mypkg/__init__.py",
            "mypkg/data/special.pth",
            "mypkg-1.0.dist-info/RECORD",
        ];
        assert_eq!(classify(&paths), EditableKind::NotEditable);
    }

    #[test]
    fn finder_inside_package_dir_is_ignored() {
        // A module named `__editable__foo.py` nested inside a package
        // is a regular module name collision, not a PEP 660 finder.
        let paths = ["mypkg/__editable__foo.py"];
        assert_eq!(classify(&paths), EditableKind::NotEditable);
    }

    #[test]
    fn windows_style_backslash_paths_treated_as_nested() {
        // The wheel ZIP format mandates forward slashes, but if a
        // platform-specific bug feeds us backslashes, we treat them
        // as nested too (defense in depth).
        let paths = ["mypkg\\__editable__foo.py", "mypkg\\sub.pth"];
        assert_eq!(classify(&paths), EditableKind::NotEditable);
    }

    #[test]
    fn is_editable_helper() {
        assert!(!EditableKind::NotEditable.is_editable());
        assert!(EditableKind::PthRedirect { pth_files: vec!["a.pth".into()] }.is_editable());
        assert!(
            EditableKind::FinderRedirect {
                finder_modules: vec!["__editable__x.py".into()],
                pth_files: vec![],
            }
            .is_editable()
        );
    }

    #[test]
    fn dot_pth_only_not_a_pth_file() {
        // A file literally named `.pth` doesn't satisfy
        // `name.ends_with(".pth")` if we expect a stem. But our rule
        // is purely `.pth` suffix, so it WOULD be picked up. Document
        // that here so future readers know it's intentional.
        let paths = [".pth"];
        match classify(&paths) {
            EditableKind::PthRedirect { pth_files } => {
                assert_eq!(pth_files, vec![".pth"]);
            }
            other => panic!("expected PthRedirect, got {other:?}"),
        }
    }

    #[test]
    fn editable_prefix_without_py_suffix_not_finder() {
        // `__editable__notes.txt` is data, not a finder.
        let paths = ["__editable__notes.txt"];
        assert_eq!(classify(&paths), EditableKind::NotEditable);
    }

    #[test]
    fn realistic_setuptools_editable_wheel() {
        let paths = [
            "__editable___mypkg_1_2_3_finder.py",
            "__editable__.mypkg-1.2.3.pth",
            "mypkg-1.2.3.dist-info/METADATA",
            "mypkg-1.2.3.dist-info/WHEEL",
            "mypkg-1.2.3.dist-info/RECORD",
            "mypkg-1.2.3.dist-info/direct_url.json",
        ];
        let kind = classify(&paths);
        match kind {
            EditableKind::FinderRedirect { finder_modules, pth_files } => {
                assert_eq!(finder_modules.len(), 1);
                assert_eq!(pth_files.len(), 1);
                assert_eq!(
                    finder_modules[0],
                    "__editable___mypkg_1_2_3_finder.py"
                );
                assert_eq!(pth_files[0], "__editable__.mypkg-1.2.3.pth");
            }
            other => panic!("expected FinderRedirect, got {other:?}"),
        }
    }

    #[test]
    fn realistic_hatchling_editable_wheel() {
        let paths = [
            "__editable__mypkg.py",
            "__editable__mypkg.pth",
            "mypkg-1.2.3.dist-info/METADATA",
            "mypkg-1.2.3.dist-info/WHEEL",
            "mypkg-1.2.3.dist-info/RECORD",
        ];
        assert_eq!(
            classify(&paths),
            EditableKind::FinderRedirect {
                finder_modules: vec!["__editable__mypkg.py".into()],
                pth_files: vec!["__editable__mypkg.pth".into()],
            }
        );
    }
}
