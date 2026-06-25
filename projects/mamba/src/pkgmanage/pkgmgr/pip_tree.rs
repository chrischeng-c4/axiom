// `uv pip tree` — dependency-tree rendering for installed environments.
//
// This bridges the installed `.dist-info/METADATA` inventory reader to the
// same deterministic tree renderer used by `mamba tree`, so project lockfile
// trees and pip-compatible environment trees share traversal semantics.

use crate::pkgmanage::pkgmgr::lockfile::{Lockfile, Package};
use crate::pkgmanage::pkgmgr::pip_inventory::InstalledDist;
use crate::pkgmanage::pkgmgr::tree::{TreeOptions, render_lockfile_tree};

pub fn render_installed_tree(dists: &[InstalledDist], opts: &TreeOptions) -> String {
    let lockfile = installed_to_lockfile(dists);
    render_lockfile_tree(&lockfile, opts)
}

fn installed_to_lockfile(dists: &[InstalledDist]) -> Lockfile {
    let mut packages: Vec<Package> = dists
        .iter()
        .map(|dist| Package {
            name: dist.name.clone(),
            version: dist.version.clone(),
            sha256: String::new(),
            source: dist.dist_info.display().to_string(),
            dependencies: dist
                .requires
                .iter()
                .filter(|req| !is_extra_marker(req))
                .cloned()
                .collect(),
            markers: None,
            source_ref: None,
        })
        .collect();
    packages.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    Lockfile {
        format_version: 1,
        input_hash: "installed-site-packages".into(),
        packages,
    }
}

fn is_extra_marker(req: &str) -> bool {
    let Some((_, marker)) = req.split_once(';') else {
        return false;
    };
    let marker = marker.trim();
    marker.contains("extra ==") || marker.contains("extra==")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
    use std::path::PathBuf;

    fn dist(name: &str, version: &str, requires: &[&str]) -> InstalledDist {
        InstalledDist {
            canonical_name: pep503_normalize(name),
            name: name.into(),
            version: version.into(),
            dist_info: PathBuf::from(format!("/fake/{name}-{version}.dist-info")),
            summary: None,
            requires: requires.iter().map(|r| r.to_string()).collect(),
            home_page: None,
            author: None,
            license: None,
        }
    }

    #[test]
    fn installed_tree_uses_shared_renderer() {
        let dists = vec![
            dist("Requests", "2.31.0", &["urllib3>=2"]),
            dist("urllib3", "2.1.0", &[]),
        ];
        let out = render_installed_tree(&dists, &TreeOptions::default());
        assert_eq!(out, "Requests v2.31.0\n└── urllib3 v2.1.0\n");
    }

    #[test]
    fn installed_tree_skips_extra_only_edges() {
        let dists = vec![
            dist("demo", "1.0.0", &["pytest ; extra == \"test\"", "idna>=3"]),
            dist("idna", "3.6", &[]),
            dist("pytest", "8.0.0", &[]),
        ];
        let out = render_installed_tree(&dists, &TreeOptions::default());
        assert!(out.contains("idna v3.6"), "{out}");
        assert!(out.contains("pytest v8.0.0"), "{out}");
        assert!(!out.contains("demo v1.0.0\n├── pytest"), "{out}");
    }
}
