// `mamba audit` — offline lockfile vulnerability audit.
//
// The command intentionally starts with a local advisory database instead of a
// live network service: package-management validation is offline-first, and CI
// needs deterministic evidence. The JSON shape is deliberately small and maps
// onto common advisory exports:
//
// {
//   "advisories": [{
//     "id": "GHSA-...",
//     "package": "demo-pkg",
//     "affected": ["<1.2.0", ">=2.0,<2.1"],
//     "severity": "high",
//     "summary": "...",
//     "url": "https://..."
//   }]
// }

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::pep440;
use crate::pkgmanage::sync::{LockedPkg, parse_locked_packages};

const LOCKFILE_FILE: &str = "mamba.lock";
const ADVISORY_DB_ENV: &str = "MAMBA_ADVISORY_DB";

pub fn cmd_audit(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let lock_path = project_dir.join(LOCKFILE_FILE);
    let lock_src =
        fs::read_to_string(&lock_path).with_context(|| format!("read {}", lock_path.display()))?;
    let packages = parse_locked_packages(&lock_src)?;
    let db_path = advisory_db_path(sub)?;
    let db = load_advisory_db(&db_path)?;
    let findings = audit_packages(&packages, &db)?;

    if sub.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&findings)?);
    } else if findings.is_empty() {
        println!("No vulnerabilities found");
    } else {
        for finding in &findings {
            println!(
                "{}\t{}\t{}=={}\t{}",
                finding.id, finding.severity, finding.package, finding.version, finding.affected
            );
            if !finding.summary.is_empty() {
                println!("  {}", finding.summary);
            }
            if !finding.url.is_empty() {
                println!("  {}", finding.url);
            }
        }
    }

    if !findings.is_empty() {
        bail!("{} vulnerable package(s) found", findings.len());
    }
    Ok(())
}

fn advisory_db_path(sub: &ArgMatches) -> Result<PathBuf> {
    sub.get_one::<String>("advisory-db")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(ADVISORY_DB_ENV).map(PathBuf::from))
        .context(
            "no advisory database configured; pass --advisory-db PATH or set MAMBA_ADVISORY_DB",
        )
}

#[derive(Debug, Deserialize)]
struct AdvisoryDb {
    #[serde(default)]
    advisories: Vec<Advisory>,
}

#[derive(Debug, Deserialize)]
struct Advisory {
    id: String,
    package: String,
    #[serde(default)]
    affected: Vec<String>,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditFinding {
    pub id: String,
    pub package: String,
    pub version: String,
    pub affected: String,
    pub severity: String,
    pub summary: String,
    pub url: String,
}

fn load_advisory_db(path: &Path) -> Result<AdvisoryDb> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn audit_packages(packages: &[LockedPkg], db: &AdvisoryDb) -> Result<Vec<AuditFinding>> {
    let mut findings = Vec::new();
    for pkg in packages {
        let pkg_name = normalize_name(&pkg.name);
        for advisory in &db.advisories {
            if normalize_name(&advisory.package) != pkg_name {
                continue;
            }
            for affected in &advisory.affected {
                if version_matches_spec_set(&pkg.version, affected)? {
                    findings.push(AuditFinding {
                        id: advisory.id.clone(),
                        package: pkg.name.clone(),
                        version: pkg.version.clone(),
                        affected: affected.clone(),
                        severity: advisory.severity.clone(),
                        summary: advisory.summary.clone(),
                        url: advisory.url.clone(),
                    });
                    break;
                }
            }
        }
    }
    findings.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then(a.version.cmp(&b.version))
            .then(a.id.cmp(&b.id))
    });
    Ok(findings)
}

fn version_matches_spec_set(version: &str, spec_set: &str) -> Result<bool> {
    let installed = pep440::parse(version)
        .with_context(|| format!("parse installed version `{version}` for audit"))?;
    for raw_clause in spec_set.split(',') {
        let clause = raw_clause.trim();
        if clause.is_empty() {
            continue;
        }
        if !version_matches_clause(&installed, clause)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn version_matches_clause(installed: &pep440::Pep440Version, clause: &str) -> Result<bool> {
    for op in ["<=", ">=", "==", "!=", "<", ">", "="] {
        if let Some(raw) = clause.strip_prefix(op) {
            let want_raw = raw.trim();
            let want = pep440::parse(want_raw)
                .with_context(|| format!("parse audit version `{want_raw}` in `{clause}`"))?;
            let ord = installed.cmp(&want);
            return Ok(match op {
                "<" => ord == Ordering::Less,
                "<=" => ord != Ordering::Greater,
                ">" => ord == Ordering::Greater,
                ">=" => ord != Ordering::Less,
                "==" | "=" => ord == Ordering::Equal,
                "!=" => ord != Ordering::Equal,
                _ => unreachable!(),
            });
        }
    }
    bail!("unsupported audit version clause `{clause}`")
}

/// PEP 503 normalize: lowercase + collapse `-`/`_`/`.` to single `-`.
fn normalize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_spec_supports_comparator_sets() {
        assert!(version_matches_spec_set("1.2.0", ">=1.0,<2.0").unwrap());
        assert!(!version_matches_spec_set("2.0.0", ">=1.0,<2.0").unwrap());
        assert!(version_matches_spec_set("1.0.0", "==1.0").unwrap());
        assert!(!version_matches_spec_set("1.0.0", "!=1.0").unwrap());
    }

    #[test]
    fn audit_matches_normalized_names() {
        let packages = vec![LockedPkg {
            name: "demo_pkg".into(),
            version: "1.0.0".into(),
            url: String::new(),
            sha256: String::new(),
            source_kind: String::new(),
            path: String::new(),
            provider: String::new(),
            provides: Vec::new(),
            compatibility: String::new(),
            maturity: String::new(),
        }];
        let db = AdvisoryDb {
            advisories: vec![Advisory {
                id: "GHSA-demo".into(),
                package: "Demo-Pkg".into(),
                affected: vec!["<2.0".into()],
                severity: "high".into(),
                summary: "demo".into(),
                url: "https://example.invalid/GHSA-demo".into(),
            }],
        };
        let findings = audit_packages(&packages, &db).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].id, "GHSA-demo");
    }
}
