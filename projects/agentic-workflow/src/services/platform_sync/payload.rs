// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/payload.md#source
// CODEGEN-BEGIN
//! Payload building for platform sync

use super::{PlatformConfig, SpecPayload, SyncPayload};
use crate::Result;
use std::fs;
use std::path::Path;

/// Extract YAML frontmatter from markdown content
/// Supports both Unix (\n) and Windows (\r\n) line endings
fn extract_frontmatter(content: &str) -> Option<&str> {
    let content = content.trim_start_matches('\u{feff}'); // Strip BOM

    // Check for frontmatter start (--- possibly with trailing whitespace)
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = &content[..first_line_end].trim_end();
    if *first_line != "---" {
        return None;
    }

    // Find the closing --- (handles both \n--- and \r\n---)
    let rest = &content[first_line_end + 1..];

    // Look for closing separator on its own line
    for (i, line) in rest.lines().enumerate() {
        if line.trim() == "---" {
            // Calculate byte offset to the start of this line
            let mut offset = 0;
            for (j, l) in rest.lines().enumerate() {
                if j == i {
                    break;
                }
                offset += l.len() + 1; // +1 for newline
                                       // Account for \r\n
                if rest[offset..].starts_with('\r') {
                    offset += 1;
                }
            }
            return Some(&rest[..offset].trim_end_matches(['\r', '\n']));
        }
    }
    None
}

/// Detect line ending style in content
fn detect_line_ending(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Extract github_issue number from frontmatter
fn extract_github_issue(content: &str) -> Option<u64> {
    if let Some(frontmatter) = extract_frontmatter(content) {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
            if let Some(issue) = yaml.get("github_issue").and_then(|v| v.as_u64()) {
                return Some(issue);
            }
        }
    }
    None
}

/// Write issue number to frontmatter of a markdown file
/// Preserves original line endings and formatting
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/payload.md#source
pub fn write_issue_to_frontmatter(path: &Path, issue_number: u64) -> Result<()> {
    let raw_content = fs::read_to_string(path)?;
    let line_ending = detect_line_ending(&raw_content);
    let content = raw_content.trim_start_matches('\u{feff}'); // Strip BOM

    // Check for frontmatter start
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = content[..first_line_end].trim_end();

    let updated = if first_line == "---" {
        // Has frontmatter - find closing separator
        let rest = &content[first_line_end + 1..];

        // Find closing --- position
        let mut closing_pos = None;
        let mut byte_offset = 0;
        for line in rest.lines() {
            if line.trim() == "---" {
                closing_pos = Some(byte_offset);
                break;
            }
            byte_offset += line.len();
            // Account for line ending
            if rest[byte_offset..].starts_with("\r\n") {
                byte_offset += 2;
            } else if rest[byte_offset..].starts_with('\n') {
                byte_offset += 1;
            }
        }

        if let Some(end_pos) = closing_pos {
            let frontmatter = &rest[..end_pos];
            let after_closing = &rest[end_pos..];

            // Check if github_issue already exists
            if frontmatter.contains("github_issue:") {
                // Update existing line
                let updated_fm: String = frontmatter
                    .lines()
                    .map(|line| {
                        if line.trim_start().starts_with("github_issue:") {
                            format!("github_issue: {}", issue_number)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(line_ending);
                format!("---{}{}{}", line_ending, updated_fm, after_closing)
            } else {
                // Add github_issue before closing ---
                format!(
                    "---{}{}github_issue: {}{}{}",
                    line_ending, frontmatter, issue_number, line_ending, after_closing
                )
            }
        } else {
            content.to_string()
        }
    } else {
        // No frontmatter - add one
        format!(
            "---{}github_issue: {}{}---{}{}",
            line_ending, issue_number, line_ending, line_ending, content
        )
    };

    fs::write(path, updated)?;
    Ok(())
}

/// Build sync payload from change artifacts
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/payload.md#source
pub fn build_payload(project_root: &Path, change_id: &str) -> Result<SyncPayload> {
    // Load config for label configuration
    let config = match PlatformConfig::load(project_root) {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!(
                "Warning: Failed to load platform config, using defaults: {}",
                e
            );
            None
        }
    };
    build_payload_with_config(project_root, change_id, config.as_ref())
}

/// Build sync payload with explicit config
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/payload.md#source
pub fn build_payload_with_config(
    project_root: &Path,
    change_id: &str,
    config: Option<&PlatformConfig>,
) -> Result<SyncPayload> {
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Read proposal
    let proposal_path = change_dir.join("proposal.md");
    let proposal = if proposal_path.exists() {
        fs::read_to_string(&proposal_path)?
    } else {
        anyhow::bail!("proposal.md not found for change '{}'", change_id);
    };

    // Extract summary from proposal frontmatter
    let summary = extract_summary(&proposal).unwrap_or_else(|| change_id.to_string());

    // Format title using config or default
    let title = config
        .map(|c| c.format_proposal_title(change_id, &summary))
        .unwrap_or_else(|| format!("[{}] {}", change_id, summary));

    // Read tasks
    let tasks_path = change_dir.join("tasks.md");
    let tasks = if tasks_path.exists() {
        Some(fs::read_to_string(&tasks_path)?)
    } else {
        None
    };

    // Read specs
    let specs_dir = change_dir.join("specs");
    let raw_specs = if specs_dir.exists() {
        read_specs(&specs_dir)?
    } else {
        Vec::new()
    };

    // Build spec payloads
    let spec_payloads = build_spec_payloads(change_id, &raw_specs, config);

    // Build parent body (without embedded specs, they'll be linked)
    let body = build_parent_body(change_id, &proposal, tasks.as_deref(), &spec_payloads);

    // Build labels
    let mut labels = Vec::new();

    // Add proposal label from config
    if let Some(label) = config.and_then(|c| c.proposal_label()) {
        labels.push(label.to_string());
    }

    // Extract affected_code from proposal and generate scope labels
    let affected_code = extract_affected_code(&proposal);
    if let Some(config) = config {
        let scope_labels = config.extract_scope_labels(&affected_code);
        labels.extend(scope_labels);
    }

    // Extract existing issue number from frontmatter
    let existing_issue = extract_github_issue(&proposal);

    Ok(SyncPayload {
        change_id: change_id.to_string(),
        title,
        body,
        labels,
        existing_issue,
        specs: spec_payloads,
    })
}

/// Build payloads for spec issues
fn build_spec_payloads(
    change_id: &str,
    specs: &[(String, String)],
    config: Option<&PlatformConfig>,
) -> Vec<SpecPayload> {
    specs
        .iter()
        .map(|(spec_id, content)| {
            let title = config
                .map(|c| c.format_spec_title(change_id, spec_id))
                .unwrap_or_else(|| format!("[{}/spec] {}", change_id, spec_id));

            let body = build_spec_body(change_id, spec_id, content);

            let mut labels = Vec::new();
            if let Some(label) = config.and_then(|c| c.spec_label()) {
                labels.push(label.to_string());
            }

            // Extract existing issue from spec frontmatter
            let existing_issue = extract_github_issue(content);

            SpecPayload {
                spec_id: spec_id.clone(),
                title,
                body,
                labels,
                existing_issue,
            }
        })
        .collect()
}

/// Build body for a spec issue
fn build_spec_body(change_id: &str, _spec_id: &str, content: &str) -> String {
    let mut body = String::new();

    body.push_str(&format!(
        "> **Parent**: Search for `[{}]` in issues\n\n",
        change_id
    ));

    body.push_str(content);

    body.push_str("\n\n---\n");
    body.push_str("*Synced by [score](https://github.com/chrischeng-c4/cclab)*\n");

    body
}

/// Build parent issue body (proposal + tasks, with spec links placeholder)
fn build_parent_body(
    change_id: &str,
    proposal: &str,
    tasks: Option<&str>,
    specs: &[SpecPayload],
) -> String {
    let mut body = String::new();

    body.push_str(&format!("# SDD Change: {}\n\n", change_id));
    body.push_str("> This issue was auto-generated by SDD Platform Sync.\n\n");

    // Proposal section
    body.push_str("## Proposal\n\n");
    body.push_str("<details>\n<summary>Click to expand proposal</summary>\n\n");
    body.push_str(proposal);
    body.push_str("\n</details>\n\n");

    // Specs section (as tasklist placeholder - will be updated with actual issue links)
    if !specs.is_empty() {
        body.push_str("## Specifications\n\n");
        body.push_str("<!-- SPEC_TASKLIST_START -->\n");
        for spec in specs {
            // Placeholder format - will be replaced with actual issue links after creation
            body.push_str(&format!("- [ ] `{}` (pending)\n", spec.spec_id));
        }
        body.push_str("<!-- SPEC_TASKLIST_END -->\n\n");
    }

    // Tasks section
    if let Some(tasks_content) = tasks {
        body.push_str("## Tasks\n\n");
        body.push_str("<details>\n<summary>Click to expand tasks</summary>\n\n");
        body.push_str(tasks_content);
        body.push_str("\n</details>\n\n");
    }

    body.push_str("---\n");
    body.push_str("*Synced by [score](https://github.com/chrischeng-c4/cclab)*\n");

    body
}

/// Update parent body with actual spec issue links
/// Preserves checkbox state from existing body if present
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/payload.md#source
pub fn update_body_with_spec_links(
    body: &str,
    spec_links: &[(String, u64, String)], // (spec_id, issue_number, url)
) -> String {
    let start_marker = "<!-- SPEC_TASKLIST_START -->";
    let end_marker = "<!-- SPEC_TASKLIST_END -->";

    if let (Some(start), Some(end)) = (body.find(start_marker), body.find(end_marker)) {
        let before = &body[..start + start_marker.len()];
        let after = &body[end..];

        // Extract existing checkbox states from current body
        let existing_section = &body[start + start_marker.len()..end];
        let mut checked_issues: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for line in existing_section.lines() {
            if line.contains("[x]") || line.contains("[X]") {
                // Extract issue number from line like "- [x] #123 `spec-id`"
                if let Some(num_start) = line.find('#') {
                    let num_str: String = line[num_start + 1..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(num) = num_str.parse::<u64>() {
                        checked_issues.insert(num);
                    }
                }
            }
        }

        // Build new list preserving checkbox state
        let mut new_list = String::from("\n");
        for (spec_id, issue_num, _url) in spec_links {
            let checkbox = if checked_issues.contains(issue_num) {
                "[x]"
            } else {
                "[ ]"
            };
            new_list.push_str(&format!("- {} #{} `{}`\n", checkbox, issue_num, spec_id));
        }

        format!("{}{}{}", before, new_list, after)
    } else {
        body.to_string()
    }
}

/// Extract summary from proposal frontmatter
fn extract_summary(proposal: &str) -> Option<String> {
    // Try proper YAML parsing first
    if let Some(frontmatter) = extract_frontmatter(proposal) {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
            if let Some(summary) = yaml.get("summary").and_then(|v| v.as_str()) {
                if !summary.is_empty() {
                    return Some(summary.to_string());
                }
            }
        }
    }

    // Fallback to line-by-line parsing for malformed frontmatter
    for line in proposal.lines() {
        if line.starts_with("summary:") {
            let summary = line.trim_start_matches("summary:").trim();
            // Remove quotes if present
            let summary = summary.trim_matches('"').trim_matches('\'');
            if !summary.is_empty() {
                return Some(summary.to_string());
            }
        }
    }
    None
}

/// Extract affected_code paths from proposal frontmatter
fn extract_affected_code(proposal: &str) -> Vec<String> {
    // Try proper YAML parsing first
    if let Some(frontmatter) = extract_frontmatter(proposal) {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
            if let Some(affected) = yaml.get("affected_code") {
                if let Some(arr) = affected.as_sequence() {
                    return arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }
    }

    // Fallback to line-by-line parsing for malformed frontmatter
    let mut in_affected_code = false;
    let mut paths = Vec::new();

    for line in proposal.lines() {
        let trimmed = line.trim();

        // End of frontmatter
        if trimmed == "---" && !paths.is_empty() {
            break;
        }

        // Start of affected_code list
        if trimmed.starts_with("affected_code:") {
            in_affected_code = true;
            // Check for inline list: affected_code: ["path1", "path2"]
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.rfind(']') {
                    let list_str = &trimmed[start + 1..end];
                    for item in list_str.split(',') {
                        let path = item.trim().trim_matches('"').trim_matches('\'');
                        if !path.is_empty() {
                            paths.push(path.to_string());
                        }
                    }
                    in_affected_code = false;
                }
            }
            continue;
        }

        // Inside affected_code list (YAML format)
        if in_affected_code {
            if trimmed.starts_with('-') {
                let path = trimmed
                    .trim_start_matches('-')
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !path.is_empty() {
                    paths.push(path.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with(' ') {
                // End of list (new key started)
                in_affected_code = false;
            }
        }
    }

    paths
}

/// Read all spec files from specs directory
/// Reports errors instead of silently ignoring them
fn read_specs(specs_dir: &Path) -> Result<Vec<(String, String)>> {
    let mut specs = Vec::new();

    let entries = fs::read_dir(specs_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read specs directory {:?}: {}", specs_dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "md") {
            let name = path
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name: {:?}", path))?
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("Failed to read spec file {:?}: {}", path, e))?;
            specs.push((name, content));
        }
    }

    specs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(specs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_change(temp_dir: &Path, change_id: &str) {
        let change_dir = temp_dir.join(".aw/changes").join(change_id);
        fs::create_dir_all(&change_dir).unwrap();

        let proposal = r#"---
id: test-change
summary: "Test change for platform sync"
---

# Test Proposal

This is a test proposal.
"#;
        fs::write(change_dir.join("proposal.md"), proposal).unwrap();

        let tasks = r#"# Tasks

## T1: First task
- [ ] Do something
"#;
        fs::write(change_dir.join("tasks.md"), tasks).unwrap();
    }

    #[test]
    fn test_build_payload() {
        let temp_dir = TempDir::new().unwrap();
        create_test_change(temp_dir.path(), "test-change");

        let payload = build_payload(temp_dir.path(), "test-change").unwrap();

        assert_eq!(payload.change_id, "test-change");
        assert!(payload.title.contains("Test change for platform sync"));
        assert!(payload.body.contains("Test Proposal"));
        assert!(payload.body.contains("T1: First task"));
        assert!(payload.existing_issue.is_none()); // No github_issue in frontmatter
    }

    #[test]
    fn test_build_payload_with_existing_issue() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        fs::create_dir_all(&change_dir).unwrap();

        let proposal = r#"---
id: test-change
summary: "Test change"
github_issue: 42
---

# Test Proposal
"#;
        fs::write(change_dir.join("proposal.md"), proposal).unwrap();

        let payload = build_payload(temp_dir.path(), "test-change").unwrap();
        assert_eq!(payload.existing_issue, Some(42));
    }

    #[test]
    fn test_extract_summary() {
        let proposal = r#"---
summary: "Add new feature"
---
# Proposal
"#;
        assert_eq!(
            extract_summary(proposal),
            Some("Add new feature".to_string())
        );

        let no_summary = "# Proposal without frontmatter";
        assert_eq!(extract_summary(no_summary), None);
    }

    #[test]
    fn test_extract_affected_code_yaml_list() {
        let proposal = r#"---
summary: "Test"
affected_code:
  - "projects/agentic-workflow/src/"
  - "projects/agentic-workflow/src/generate/"
---
# Proposal
"#;
        let paths = extract_affected_code(proposal);
        assert_eq!(
            paths,
            vec![
                "projects/agentic-workflow/src/",
                "projects/agentic-workflow/src/generate/"
            ]
        );
    }

    #[test]
    fn test_extract_affected_code_inline_list() {
        let proposal = r#"---
summary: "Test"
affected_code: ["projects/agentic-workflow/", "crates/cclab-lens/"]
---
# Proposal
"#;
        let paths = extract_affected_code(proposal);
        assert_eq!(
            paths,
            vec!["projects/agentic-workflow/", "crates/cclab-lens/"]
        );
    }

    #[test]
    fn test_extract_affected_code_empty() {
        let proposal = r#"---
summary: "Test"
---
# Proposal
"#;
        let paths = extract_affected_code(proposal);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_update_body_with_spec_links_preserves_checkbox() {
        let body = r#"# SDD Change

## Specifications
<!-- SPEC_TASKLIST_START -->
- [x] #100 `spec-a`
- [ ] #101 `spec-b`
<!-- SPEC_TASKLIST_END -->

## Tasks
"#;

        let spec_links = vec![
            ("spec-a".to_string(), 100, "url".to_string()),
            ("spec-b".to_string(), 101, "url".to_string()),
            ("spec-c".to_string(), 102, "url".to_string()),
        ];

        let updated = update_body_with_spec_links(body, &spec_links);

        // spec-a should still be checked
        assert!(updated.contains("- [x] #100 `spec-a`"));
        // spec-b should still be unchecked
        assert!(updated.contains("- [ ] #101 `spec-b`"));
        // spec-c is new, should be unchecked
        assert!(updated.contains("- [ ] #102 `spec-c`"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\nsummary: test\n---\n# Body";
        let fm = extract_frontmatter(content);
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("summary: test"));

        let no_frontmatter = "# Body without frontmatter";
        assert_eq!(extract_frontmatter(no_frontmatter), None);

        // With BOM
        let with_bom = "\u{feff}---\nkey: value\n---\nbody";
        let fm_bom = extract_frontmatter(with_bom);
        assert!(fm_bom.is_some());
        assert!(fm_bom.unwrap().contains("key: value"));

        // Windows line endings
        let windows_content = "---\r\nsummary: test\r\n---\r\n# Body";
        let fm_windows = extract_frontmatter(windows_content);
        assert!(fm_windows.is_some());
        assert!(fm_windows.unwrap().contains("summary: test"));

        // Trailing spaces on separator
        let trailing_spaces = "---  \nsummary: test\n---  \n# Body";
        let fm_spaces = extract_frontmatter(trailing_spaces);
        assert!(fm_spaces.is_some());
        assert!(fm_spaces.unwrap().contains("summary: test"));
    }

    #[test]
    fn test_write_issue_preserves_windows_line_endings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Create file with Windows line endings
        fs::write(&file_path, "---\r\nsummary: test\r\n---\r\n# Body").unwrap();

        write_issue_to_frontmatter(&file_path, 123).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("github_issue: 123"));
        // Should preserve \r\n line endings
        assert!(content.contains("\r\n"));
    }

    #[test]
    fn test_extract_github_issue() {
        let with_issue = "---\nsummary: test\ngithub_issue: 42\n---\n# Body";
        assert_eq!(extract_github_issue(with_issue), Some(42));

        let without_issue = "---\nsummary: test\n---\n# Body";
        assert_eq!(extract_github_issue(without_issue), None);
    }

    #[test]
    fn test_write_issue_to_frontmatter_new() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Create file with existing frontmatter
        fs::write(&file_path, "---\nsummary: test\n---\n# Body").unwrap();

        write_issue_to_frontmatter(&file_path, 123).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("github_issue: 123"));
        assert!(content.contains("summary: test"));
    }

    #[test]
    fn test_write_issue_to_frontmatter_update() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Create file with existing github_issue
        fs::write(
            &file_path,
            "---\nsummary: test\ngithub_issue: 42\n---\n# Body",
        )
        .unwrap();

        write_issue_to_frontmatter(&file_path, 123).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("github_issue: 123"));
        assert!(!content.contains("github_issue: 42"));
    }

    #[test]
    fn test_write_issue_to_frontmatter_no_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Create file without frontmatter
        fs::write(&file_path, "# Body without frontmatter").unwrap();

        write_issue_to_frontmatter(&file_path, 123).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("github_issue: 123"));
        assert!(content.contains("# Body without frontmatter"));
    }
}
// CODEGEN-END
