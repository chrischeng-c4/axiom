// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/td_check_section_type.md#source
// CODEGEN-BEGIN

//! `aw td check --section-type-conformance [<path>]` verb body.
//!
//! Walks the requested path (default: project root) for TD specs under
//! `.aw/tech-design/**/*.md`, enumerates every `## ` H2 with its
//! `<!-- type: ... lang: ... -->` annotation, and reports per-spec which
//! sections fail to match the section-type registry's `section_types.const`
//! list. Three finding kinds: `unknown`, `deprecated`,
//! `missing-type-annotation`. Exit 0 on no findings, 1 on findings, 2 on
//! invocation error (registry missing / unparsable).
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-td-check-section-type-conformance.md#schema

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const REGISTRY_PROJECT: &str = "agentic-workflow";
const REGISTRY_PROJECT_REL_PATH: &str = "surface/specs/score-section-type-registry.md";

const SEED_DEPRECATED: &[&str] = &["overview", "requirements", "doc"];

// Args mirror the CLI shape (`--section-type-conformance` toggle plus
// optional positional `path` and `--json`). The toggle is checked at the
// dispatch site in `td.rs`; this module only runs once it's true.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-td-check-section-type-conformance.md#schema
#[derive(Debug, Clone)]
pub struct CheckArgs {
    pub path: Option<PathBuf>,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td_check_section_type.md#source
pub enum FindingKind {
    Unknown,
    Deprecated,
    MissingTypeAnnotation,
}

#[derive(Debug, Clone, Serialize)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td_check_section_type.md#source
pub struct Finding {
    pub spec_path: String,
    pub section_heading: String,
    pub kind: FindingKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td_check_section_type.md#source
pub struct Report {
    pub findings: Vec<Finding>,
    pub total_specs: usize,
    pub total_findings: usize,
}

struct Registry {
    section_types: HashSet<String>,
    deprecated: HashSet<String>,
}

// Entry point — runs the conformance check and exits the process via
// `std::process::exit` to surface the spec-mandated exit code (0/1/2).
// `td.rs` calls this and propagates any returned error from registry-load
// failures into exit code 2 via the existing translator.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-td-check-section-type-conformance.md#logic
pub fn run(args: CheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let registry =
        load_registry(&project_root).context("loading section-type registry (exit 2)")?;

    let scope_root = match &args.path {
        Some(p) if p.is_absolute() => p.clone(),
        Some(p) => std::env::current_dir()?.join(p),
        None => crate::shared::workspace::tech_design_path(&project_root),
    };

    let specs = collect_specs(&scope_root)?;
    let mut findings = Vec::new();
    let mut scanned = 0usize;
    for spec_abs in &specs {
        let sections = match scan_spec(spec_abs) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if sections.is_empty() {
            continue;
        }
        scanned += 1;
        let rel = spec_abs
            .strip_prefix(&project_root)
            .unwrap_or(spec_abs)
            .to_string_lossy()
            .into_owned();
        for sec in sections {
            classify_section(&registry, &rel, &sec, &mut findings);
        }
    }

    findings.sort_by(|a, b| {
        a.spec_path
            .cmp(&b.spec_path)
            .then_with(|| a.section_heading.cmp(&b.section_heading))
    });

    let report = Report {
        total_specs: scanned,
        total_findings: findings.len(),
        findings,
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).context("serialise report")?
        );
    } else {
        print_human(&report);
    }

    if report.total_findings == 0 {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn print_human(r: &Report) {
    if r.findings.is_empty() {
        println!(
            "section-type conformance: 0 findings across {} spec(s)",
            r.total_specs
        );
        return;
    }
    for f in &r.findings {
        let ann = f
            .annotation_type
            .as_deref()
            .map(|t| format!(" (type: {})", t))
            .unwrap_or_default();
        println!(
            "{}: {}: {}{}",
            f.spec_path,
            f.section_heading,
            kind_label(&f.kind),
            ann
        );
    }
    println!(
        "section-type conformance: {} finding(s) across {} spec(s)",
        r.total_findings, r.total_specs
    );
}

fn kind_label(k: &FindingKind) -> &'static str {
    match k {
        FindingKind::Unknown => "unknown",
        FindingKind::Deprecated => "deprecated",
        FindingKind::MissingTypeAnnotation => "missing-type-annotation",
    }
}

// Locate and parse the section-type registry's `## Schema` YAML block.
// The block is a fenced ```yaml block immediately after the
// `## Schema` heading + `<!-- type: schema lang: yaml -->` annotation.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-section-type-registry.md#schema
fn load_registry(project_root: &Path) -> Result<Registry> {
    let registry_abs = crate::services::project_registry::resolve_td_root_from_config(
        project_root,
        REGISTRY_PROJECT,
    )
    .map(|resolved| PathBuf::from(resolved.root).join(REGISTRY_PROJECT_REL_PATH))
    .unwrap_or_else(|_| {
        crate::shared::workspace::tech_design_path(project_root)
            .join("projects")
            .join(REGISTRY_PROJECT)
            .join(REGISTRY_PROJECT_REL_PATH)
    });
    let raw = std::fs::read_to_string(&registry_abs)
        .with_context(|| format!("read registry at {}", registry_abs.display()))?;
    let yaml_block = extract_yaml_block(&raw, "Schema")
        .ok_or_else(|| anyhow::anyhow!("registry missing `## Schema` yaml block"))?;
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&yaml_block).context("parse registry Schema block as YAML")?;

    let section_types = doc
        .get("properties")
        .and_then(|p| p.get("section_types"))
        .and_then(|s| s.get("const"))
        .and_then(|c| c.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| {
                    item.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    if section_types.is_empty() {
        anyhow::bail!("registry section_types.const list is empty or unparseable");
    }

    let mut deprecated: HashSet<String> = doc
        .get("properties")
        .and_then(|p| p.get("deprecated"))
        .and_then(|d| d.get("const"))
        .and_then(|c| c.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    for seed in SEED_DEPRECATED {
        deprecated.insert((*seed).into());
    }

    Ok(Registry {
        section_types,
        deprecated,
    })
}

// Pull out the YAML body of a fenced block under heading `## <name>`.
fn extract_yaml_block(raw: &str, heading: &str) -> Option<String> {
    let lines: Vec<&str> = raw.lines().collect();
    let head_marker = format!("## {}", heading);
    let head_idx = lines.iter().position(|l| l.trim_end() == head_marker)?;
    let fence_open = lines
        .iter()
        .enumerate()
        .skip(head_idx + 1)
        .find(|(_, l)| l.trim_start().starts_with("```yaml"))
        .map(|(i, _)| i)?;
    let fence_close = lines
        .iter()
        .enumerate()
        .skip(fence_open + 1)
        .find(|(_, l)| l.trim_start().starts_with("```"))
        .map(|(i, _)| i)?;
    Some(lines[fence_open + 1..fence_close].join("\n"))
}

#[derive(Debug)]
struct H2Section {
    heading: String,
    annotation_type: Option<String>,
}

// Walk a spec file and emit one H2Section per `## ` heading.
fn scan_spec(path: &Path) -> Result<Vec<H2Section>> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("read spec {}", path.display()))?;
    let lines: Vec<&str> = raw.lines().collect();

    // Strip frontmatter so we don't treat YAML keys starting with `## `
    // (none expected, but be defensive).
    let body_start = if lines.first() == Some(&"---") {
        lines
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, l)| l.trim() == "---")
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
    } else {
        0
    };

    // Skip code-fence regions so an `## ` line inside ```...``` doesn't
    // count as a heading. Also skip H2s under a `# Reviews` H1 — that's
    // the CRRR review block, not a TD section.
    let mut in_fence = false;
    let mut in_reviews_block = false;
    let mut sections = Vec::new();
    for i in body_start..lines.len() {
        let l = lines[i];
        if l.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if l.starts_with("# ") {
            in_reviews_block = l.trim() == "# Reviews";
            continue;
        }
        if in_reviews_block {
            continue;
        }
        if let Some(rest) = l.strip_prefix("## ") {
            let heading = rest.trim().to_string();
            // The annotation should be on the next non-empty line.
            let annotation_type = (i + 1..lines.len())
                .find(|j| !lines[*j].trim().is_empty())
                .and_then(|j| parse_annotation(lines[j]));
            sections.push(H2Section {
                heading,
                annotation_type,
            });
        }
    }
    Ok(sections)
}

// Parse `<!-- type: <type> lang: <lang> -->` and return the type field.
// Returns None if the line is not an annotation comment.
fn parse_annotation(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let mut ty: Option<String> = None;
    for token in inner.split_whitespace().collect::<Vec<_>>().windows(2) {
        if token[0] == "type:" {
            ty = Some(token[1].trim_end_matches(',').to_string());
        }
    }
    ty
}

fn classify_section(registry: &Registry, spec_path: &str, sec: &H2Section, out: &mut Vec<Finding>) {
    match &sec.annotation_type {
        None => out.push(Finding {
            spec_path: spec_path.to_string(),
            section_heading: sec.heading.clone(),
            kind: FindingKind::MissingTypeAnnotation,
            annotation_type: None,
            suggestion: Some(
                "add `<!-- type: <section-type> lang: <family> -->` after the heading".to_string(),
            ),
        }),
        Some(ty) if registry.section_types.contains(ty) => {}
        Some(ty) if registry.deprecated.contains(ty) => out.push(Finding {
            spec_path: spec_path.to_string(),
            section_heading: sec.heading.clone(),
            kind: FindingKind::Deprecated,
            annotation_type: Some(ty.clone()),
            suggestion: Some(format!(
                "type '{}' is deprecated; migrate per AUTHORING.md \"Deprecated types\"",
                ty
            )),
        }),
        Some(ty) => out.push(Finding {
            spec_path: spec_path.to_string(),
            section_heading: sec.heading.clone(),
            kind: FindingKind::Unknown,
            annotation_type: Some(ty.clone()),
            suggestion: Some(format!(
                "type '{}' is not in the registry; rename or extend section-type-registry",
                ty
            )),
        }),
    }
}

// Collect candidate spec files under `scope`. When `scope` is a single
// `*.md` file, return just that. When it's a directory, walk recursively
// for `*.md` files, skipping `target/`, `node_modules/`, and dot-dirs.
fn collect_specs(scope: &Path) -> Result<Vec<PathBuf>> {
    if scope.is_file() {
        if scope.extension().and_then(|e| e.to_str()) == Some("md") {
            return Ok(vec![scope.to_path_buf()]);
        }
        anyhow::bail!("path is not a .md file: {}", scope.display());
    }
    if !scope.is_dir() {
        anyhow::bail!("path does not exist: {}", scope.display());
    }
    let mut out = Vec::new();
    walk(scope, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if path.is_dir() {
            // Skip .git, target, node_modules, and hidden dirs.
            if name_s.starts_with('.') || name_s == "target" || name_s == "node_modules" {
                continue;
            }
            walk(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::spec_rules::SectionType;

    #[test]
    fn parse_annotation_extracts_type() {
        assert_eq!(
            parse_annotation("<!-- type: schema lang: yaml -->"),
            Some("schema".to_string())
        );
        assert_eq!(
            parse_annotation("<!-- type: logic lang: mermaid -->"),
            Some("logic".to_string())
        );
        assert_eq!(parse_annotation("regular line"), None);
        assert_eq!(parse_annotation("<!-- not an annotation -->"), None);
    }

    #[test]
    fn classify_routes_three_kinds() {
        let mut reg = Registry {
            section_types: HashSet::new(),
            deprecated: HashSet::new(),
        };
        reg.section_types.insert("logic".into());
        reg.deprecated.insert("overview".into());

        let mut out = Vec::new();

        classify_section(
            &reg,
            "spec.md",
            &H2Section {
                heading: "Logic".into(),
                annotation_type: Some("logic".into()),
            },
            &mut out,
        );
        assert!(out.is_empty());

        classify_section(
            &reg,
            "spec.md",
            &H2Section {
                heading: "Overview".into(),
                annotation_type: Some("overview".into()),
            },
            &mut out,
        );
        classify_section(
            &reg,
            "spec.md",
            &H2Section {
                heading: "Bogus".into(),
                annotation_type: Some("nope".into()),
            },
            &mut out,
        );
        classify_section(
            &reg,
            "spec.md",
            &H2Section {
                heading: "Headless".into(),
                annotation_type: None,
            },
            &mut out,
        );
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].kind, FindingKind::Deprecated);
        assert_eq!(out[1].kind, FindingKind::Unknown);
        assert_eq!(out[2].kind, FindingKind::MissingTypeAnnotation);
    }

    #[test]
    fn extract_yaml_block_under_heading() {
        let doc = "# Title\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nfoo: bar\n```\n\n## Other\n";
        let body = extract_yaml_block(doc, "Schema").unwrap();
        assert_eq!(body, "foo: bar");
    }

    #[test]
    fn collect_specs_accepts_configured_non_score_td_root() {
        let dir = tempfile::tempdir().unwrap();
        let td_root = dir
            .path()
            .join("projects/mamba/mambalibs/httpkit/tech_design");
        std::fs::create_dir_all(td_root.join("logic")).unwrap();
        let spec = td_root.join("logic/runtime.md");
        std::fs::write(&spec, "## Logic\n<!-- type: logic lang: mermaid -->\n").unwrap();

        let specs = collect_specs(&td_root).unwrap();

        assert_eq!(specs, vec![spec]);
    }

    #[test]
    fn load_registry_uses_score_project_td_path_from_config() {
        let dir = tempfile::tempdir().unwrap();
        let score_td = dir.path().join("projects/agentic-workflow/tech-design");
        std::fs::create_dir_all(score_td.join("surface/specs")).unwrap();
        std::fs::create_dir_all(dir.path().join(".aw")).unwrap();
        std::fs::write(
            dir.path().join(".aw/config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        )
        .unwrap();
        std::fs::write(
            score_td.join(REGISTRY_PROJECT_REL_PATH),
            r#"## Schema
<!-- type: schema lang: yaml -->

```yaml
properties:
  section_types:
    const:
      - name: logic
  deprecated:
    const: []
```
"#,
        )
        .unwrap();

        let registry = load_registry(dir.path()).unwrap();

        assert!(registry.section_types.contains("logic"));
    }

    #[test]
    fn repo_registry_covers_all_non_deprecated_section_type_variants() {
        let project_root = crate::find_project_root().unwrap();
        let registry = load_registry(&project_root).unwrap();

        for section_type in SectionType::all_in_fill_order() {
            if matches!(
                section_type,
                SectionType::Overview | SectionType::Requirements | SectionType::Doc
            ) {
                continue;
            }
            assert!(
                registry.section_types.contains(section_type.as_str()),
                "registry missing approved section type `{}`",
                section_type.as_str()
            );
        }
        assert!(
            registry.section_types.contains("source"),
            "registry should keep the legacy source-template section type"
        );
    }

    #[test]
    fn scan_spec_emits_h2_with_annotation() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("s.md");
        std::fs::write(
            &p,
            "---\nid: x\n---\n\n# H1\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nx: 1\n```\n\n## Bare\n\nnot annotated\n",
        )
        .unwrap();
        let secs = scan_spec(&p).unwrap();
        assert_eq!(secs.len(), 2);
        assert_eq!(secs[0].heading, "Schema");
        assert_eq!(secs[0].annotation_type.as_deref(), Some("schema"));
        assert_eq!(secs[1].heading, "Bare");
        assert!(secs[1].annotation_type.is_none());
    }
}

// CODEGEN-END
