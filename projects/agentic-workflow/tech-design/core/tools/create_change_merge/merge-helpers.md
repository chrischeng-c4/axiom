---
id: sdd-tools-create-change-merge-merge-helpers
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge merge helpers

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── 3-Way Merge Support ─────────────────────────────────────────────────────

/// Perform a 3-way merge using `git merge-file`.
///
/// Writes ours/base/theirs to temp files, invokes `git merge-file --stdout`,
/// and returns the merged content on clean merge (exit 0) or an error with
/// conflict details on exit >0.
fn merge_3way(
    git: &Path,
    ours: &str,
    base: &str,
    theirs: &str,
) -> std::result::Result<String, String> {
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("failed to create tempdir: {}", e))?;

    let ours_path = tmp_dir.path().join("ours.md");
    let base_path = tmp_dir.path().join("base.md");
    let theirs_path = tmp_dir.path().join("theirs.md");

    std::fs::write(&ours_path, ours).map_err(|e| format!("write ours: {}", e))?;
    std::fs::write(&base_path, base).map_err(|e| format!("write base: {}", e))?;
    std::fs::write(&theirs_path, theirs).map_err(|e| format!("write theirs: {}", e))?;

    let output = std::process::Command::new(git)
        .arg("merge-file")
        .arg("--stdout")
        .arg(&ours_path)
        .arg(&base_path)
        .arg(&theirs_path)
        .output()
        .map_err(|e| format!("git merge-file failed to execute: {}", e))?;

    let merged = String::from_utf8_lossy(&output.stdout).to_string();

    if output.status.success() {
        Ok(merged)
    } else {
        // exit code >0 means conflicts
        let conflict_count = merged.matches("<<<<<<<").count();
        Err(format!(
            "merge conflict ({} marker{})",
            conflict_count,
            if conflict_count == 1 { "" } else { "s" }
        ))
    }
}

// ─── Section-Level Merge ────────────────────────────────────────────────────

/// Merge change-spec sections into an existing target spec, preserving
/// sections that the change doesn't touch.
///
/// Sections are delimited by `## Heading` lines. The frontmatter block
/// (between `---` fences) is always taken from the change spec (it contains
/// the updated metadata). Content before the first `## ` heading (the title
/// / preamble) is taken from the change spec if it is non-empty, otherwise
/// preserved from the target.
///
/// For each `## Heading` in the change spec:
/// - If the target has a section with the same heading, it is **replaced**.
/// - If the target does not have it, it is **appended**.
///
/// Sections in the target that are NOT present in the change spec are
/// **preserved** in their original order (inserted between the last
/// preceding change-spec section and the next one).
///
/// REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
fn merge_sections_into_target(target: &str, change: &str) -> String {
    let target_sections = parse_markdown_sections(target);
    let change_sections = parse_markdown_sections(change);

    // Use the change spec's frontmatter (updated metadata).
    let mut result = String::new();
    result.push_str(&change_sections.frontmatter);

    // Use the change spec's preamble (title + intro before first ## section)
    // if non-empty; otherwise keep the target's preamble.
    let preamble = if change_sections.preamble.trim().is_empty() {
        &target_sections.preamble
    } else {
        &change_sections.preamble
    };
    result.push_str(preamble);

    // Build a set of headings present in the change spec for quick lookup.
    let change_headings: std::collections::HashSet<&str> = change_sections
        .sections
        .iter()
        .map(|s| s.heading.as_str())
        .collect();

    // Build a set of headings already emitted to avoid duplicates.
    let mut emitted: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Target heading -> content lookup for sections NOT in the change spec.
    let target_section_map: std::collections::HashMap<&str, &str> = target_sections
        .sections
        .iter()
        .map(|s| (s.heading.as_str(), s.body.as_str()))
        .collect();

    // Walk the change spec sections in order.
    // Before emitting each change section, emit any target-only sections
    // that appeared before this position in the target.
    let target_order: Vec<&str> = target_sections
        .sections
        .iter()
        .map(|s| s.heading.as_str())
        .collect();

    let mut target_cursor = 0; // next target section index to check

    for cs in &change_sections.sections {
        // Emit any target-only sections that come before this change section
        // in the target's ordering.
        if let Some(cs_target_idx) = target_order.iter().position(|h| *h == cs.heading.as_str()) {
            while target_cursor < cs_target_idx {
                let th = target_order[target_cursor];
                if !change_headings.contains(th) && !emitted.contains(th) {
                    if let Some(body) = target_section_map.get(th) {
                        result.push_str(&format!("## {}\n", th));
                        result.push_str(body);
                        emitted.insert(th.to_string());
                    }
                }
                target_cursor += 1;
            }
            target_cursor = cs_target_idx + 1;
        }

        // Emit the change section (replaces any existing target section).
        result.push_str(&format!("## {}\n", cs.heading));
        result.push_str(&cs.body);
        emitted.insert(cs.heading.clone());
    }

    // Emit remaining target-only sections that come after the last
    // change section in the target's ordering.
    for th in &target_order[target_cursor..] {
        if !change_headings.contains(th) && !emitted.contains(*th) {
            if let Some(body) = target_section_map.get(th) {
                result.push_str(&format!("## {}\n", *th));
                result.push_str(body);
                emitted.insert(th.to_string());
            }
        }
    }

    result
}

/// Parsed representation of a markdown spec file split into sections.
struct ParsedSections {
    /// The YAML frontmatter block including `---` delimiters + trailing newline.
    frontmatter: String,
    /// Content between end of frontmatter and the first `## ` heading.
    preamble: String,
    /// Ordered list of `## `-level sections.
    sections: Vec<MdSection>,
}

struct MdSection {
    /// The heading text (without the `## ` prefix).
    heading: String,
    /// Everything after the heading line up to (but not including) the next
    /// `## ` heading or end of file.
    body: String,
}

/// Split a markdown spec file into frontmatter, preamble, and `## `-delimited sections.
fn parse_markdown_sections(content: &str) -> ParsedSections {
    let mut frontmatter = String::new();
    let mut rest = content;

    // Extract frontmatter if present
    if content.starts_with("---\n") || content.starts_with("---\r\n") {
        if let Some(end_idx) = content[4..].find("\n---") {
            let fm_end = 4 + end_idx + 4; // skip past closing "---\n"
                                          // Advance past the newline after closing ---
            let fm_end = if content[fm_end..].starts_with('\n') {
                fm_end + 1
            } else {
                fm_end
            };
            frontmatter = content[..fm_end].to_string();
            rest = &content[fm_end..];
        }
    }

    let mut preamble = String::new();
    let mut sections: Vec<MdSection> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_body = String::new();

    for line in rest.lines() {
        if line.starts_with("## ") && !line.starts_with("### ") {
            // New section boundary
            if let Some(heading) = current_heading.take() {
                sections.push(MdSection {
                    heading,
                    body: current_body.clone(),
                });
                current_body.clear();
            }
            let heading_text = line[3..].trim().to_string();
            current_heading = Some(heading_text);
        } else if current_heading.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        } else {
            preamble.push_str(line);
            preamble.push('\n');
        }
    }

    // Flush last section
    if let Some(heading) = current_heading {
        sections.push(MdSection {
            heading,
            body: current_body,
        });
    }

    ParsedSections {
        frontmatter,
        preamble,
        sections,
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "merge_3way"
      - "merge_sections_into_target"
      - "ParsedSections"
      - "MdSection"
      - "parse_markdown_sections"
    description: "Three-way merge and section-level markdown merge helpers."
```
