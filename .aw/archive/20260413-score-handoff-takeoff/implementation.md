---
id: implementation
type: change_implementation
change_id: score-handoff-takeoff
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
D	.claude/skills/handoff/SKILL.md
A	.claude/skills/score-handoff/SKILL.md
A	.claude/skills/score-takeoff/SKILL.md
M	projects/score/cli/src/commands.rs
A	projects/score/cli/src/handoff.rs
M	projects/score/cli/src/lib.rs
```

## Diff Statistics

```
.claude/skills/handoff/SKILL.md       |  65 ----
 .claude/skills/score-handoff/SKILL.md |  55 +++
 .claude/skills/score-takeoff/SKILL.md |  54 +++
 projects/score/cli/src/commands.rs    |  23 ++
 projects/score/cli/src/handoff.rs     | 697 ++++++++++++++++++++++++++++++++++
 projects/score/cli/src/lib.rs         |   1 +
 6 files changed, 830 insertions(+), 65 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/handoff/SKILL.md b/.claude/skills/handoff/SKILL.md
deleted file mode 100644
index 03590794..00000000
--- a/.claude/skills/handoff/SKILL.md
+++ /dev/null
@@ -1,65 +0,0 @@
----
-name: handoff
-description: Write a structured handoff document for mid-flight work (context switch, branch merge, session break)
-user-invocable: true
----
-
-# /handoff
-
-Write a structured handoff document so the next session (or person) can resume without losing context.
-
-## Arguments
-
-```
-/handoff [output-path] [topic]
-```
-
-- `output-path` — file path to write (default: `/tmp/handoff-{topic}.md`)
-- `topic` — short label for the filename (default: inferred from current work)
-
-## Instructions
-
-### Step 1: Gather context
-
-1. Check git status and recent commits on current branch
-2. Review any uncommitted changes (`git diff`, `git diff --cached`)
-3. Read conversation history for what was attempted, discovered, and decided
-4. Check for any background tasks or running processes related to the work
-
-### Step 2: Write the document
-
-Use exactly these 6 sections:
-
-```markdown
-# {Title} — Handoff
-
-## 1. Problem & Current State
-What's the goal? What's the status right now? (% done, blocking/not, which branch)
-
-## 2. Findings
-What was discovered during investigation — root causes, key insights, surprises.
-Things the next person wouldn't know just from reading the code.
-
-## 3. What Was Done
-Concrete changes made — files, functions, commits. Mark each as:
-- tested/verified
-- written but untested
-- planned but not started
-
-## 4. Next Steps
-Ordered action items to resume. Include exact commands where possible.
-
-## 5. Success Criteria
-How to verify the goal is achieved. Concrete, testable conditions.
-
-## 6. Notes
-Edge cases, risks, things that surprised you, cleanup needed, related issues.
-```
-
-### Rules
-
-- **Be concrete**: file paths, function names, exact commands — not vague descriptions
-- **Mark test status**: clearly distinguish "done & tested" from "written but not tested"
-- **Include commands**: the reader should be able to copy-paste and resume
-- **No prose padding**: bullet points over paragraphs
-- **Separate what you know from what you assume**: if something is a hypothesis, say so
diff --git a/.claude/skills/score-handoff/SKILL.md b/.claude/skills/score-handoff/SKILL.md
new file mode 100644
index 00000000..e7708f20
--- /dev/null
+++ b/.claude/skills/score-handoff/SKILL.md
@@ -0,0 +1,55 @@
+---
+name: score:handoff
+description: Create a structured handoff document for session continuity — CLI owns structure, model fills content
+user-invocable: true
+---
+
+# /score:handoff
+
+<!-- @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R6 -->
+
+Create a structured handoff document so the next session can resume without losing context.
+
+## Usage
+
+```
+/score:handoff <topic>
+```
+
+## Instructions
+
+### Step 1: Create skeleton via CLI
+
+```bash
+score handoff create --topic "<topic>" --json
+```
+
+The CLI creates `~/.score/handoffs/YYYYMMDD-<topic>.md` with YAML frontmatter (topic, date, project, branch) and 5 section headers: Status, Findings, Done, Next, Criteria.
+
+### Step 2: Fill sections (mainthread)
+
+Read the created file and fill each section with concrete content:
+
+1. **Status** -- One-liner summary of current state (% done, blocking/not, which branch)
+2. **Findings** -- Key discoveries, root causes, insights the next session wouldn't know from code alone
+3. **Done** -- Concrete changes made: files, functions, commits. Mark each as tested/untested/planned
+4. **Next** -- Ordered action items to resume. Include exact commands where possible
+5. **Criteria** -- Testable conditions using `- [ ]` checkboxes. Wrap shell commands in backticks for auto-verification by `score takeoff`:
+   ```
+   - [ ] `cargo test -p sdd --lib` passes
+   - [ ] `cargo check` passes
+   - [ ] no regressions (manual)
+   ```
+
+### Rules
+
+- Be concrete: file paths, function names, exact commands -- not vague descriptions
+- Mark test status: clearly distinguish "done and tested" from "written but not tested"
+- Include commands: the reader should be able to copy-paste and resume
+- No prose padding: bullet points over paragraphs
+- Separate what you know from what you assume
+- Criteria with backtick-wrapped commands will be auto-verified by `score takeoff`
+
+### Step 3: Confirm
+
+After filling, print the file path and suggest: `score takeoff --latest` to verify criteria on resume.
diff --git a/.claude/skills/score-takeoff/SKILL.md b/.claude/skills/score-takeoff/SKILL.md
new file mode 100644
index 00000000..504d2f15
--- /dev/null
+++ b/.claude/skills/score-takeoff/SKILL.md
@@ -0,0 +1,54 @@
+---
+name: score:takeoff
+description: Resume from a handoff document — auto-verify criteria, display next steps
+user-invocable: true
+---
+
+# /score:takeoff
+
+<!-- @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R7 -->
+
+Resume work from a handoff document. Runs auto-verification on criteria and displays next steps.
+
+## Usage
+
+```
+/score:takeoff [--latest | <file>]
+```
+
+## Instructions
+
+### Step 1: Run takeoff via CLI
+
+```bash
+score takeoff --latest --json
+```
+
+Or with a specific file:
+
+```bash
+score takeoff <path-or-filename> --json
+```
+
+The CLI will:
+1. Parse the handoff frontmatter and sections
+2. Extract `## Criteria` checkbox items
+3. Run backtick-wrapped commands and record pass/fail for each
+4. Output JSON with criteria results, next steps, and overall status
+
+### Step 2: Report results (mainthread)
+
+1. Display the criteria verification results (pass/fail for each auto criterion)
+2. Note any manual criteria that need human verification
+3. If all auto criteria passed, proceed with the `## Next` steps
+4. If any failed, investigate the failure before proceeding
+
+### Step 3: Resume work
+
+Follow the `## Next` steps from the handoff document. These are the ordered action items the previous session left for you.
+
+Read the full handoff document if additional context is needed:
+
+```bash
+score handoff show --latest
+```
diff --git a/projects/score/cli/src/commands.rs b/projects/score/cli/src/commands.rs
index ca55d027..072783df 100644
--- a/projects/score/cli/src/commands.rs
+++ b/projects/score/cli/src/commands.rs
@@ -7,6 +7,7 @@ use crate::codegen;
 use crate::daemon;
 use crate::direct;
 use crate::fillback;
+use crate::handoff;
 use crate::init;
 use crate::issues;
 use crate::list;
@@ -144,6 +145,18 @@ pub enum Commands {
         payload_path: String,
     },
 
+    // =====================================================================
+    // Session Continuity
+    // =====================================================================
+
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R1
+    /// Manage handoff documents for structured session continuity
+    Handoff(handoff::HandoffArgs),
+
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R4
+    /// Resume from a handoff — auto-verify criteria and display next steps
+    Takeoff(handoff::TakeoffArgs),
+
     // =====================================================================
     // Agent Context
     // =====================================================================
@@ -866,6 +879,16 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
             println!("{}", result);
         }
 
+        // =================================================================
+        // Session Continuity
+        // =================================================================
+        Commands::Handoff(args) => {
+            handoff::run_handoff(args).await?;
+        }
+        Commands::Takeoff(args) => {
+            handoff::run_takeoff(args).await?;
+        }
+
         // =================================================================
         // Agent Context
         // =================================================================
diff --git a/projects/score/cli/src/handoff.rs b/projects/score/cli/src/handoff.rs
new file mode 100644
index 00000000..f93ba0c3
--- /dev/null
+++ b/projects/score/cli/src/handoff.rs
@@ -0,0 +1,697 @@
+//! `score handoff` + `score takeoff` CLI — structured session continuity.
+//!
+//! `handoff create` writes a skeleton with YAML frontmatter + 5 fixed sections.
+//! `handoff list` shows handoffs newest-first.
+//! `handoff show` displays full content.
+//! `takeoff` reads a handoff, auto-verifies Criteria checkboxes, and displays Next steps.
+
+use anyhow::{Context, Result};
+use chrono::Local;
+use clap::{Args, Subcommand};
+use serde::{Deserialize, Serialize};
+use std::collections::BTreeMap;
+use std::path::{Path, PathBuf};
+
+// ---------------------------------------------------------------------------
+// CLI argument types
+// ---------------------------------------------------------------------------
+
+#[derive(Debug, Args)]
+pub struct HandoffArgs {
+    #[command(subcommand)]
+    pub command: HandoffCommand,
+}
+
+#[derive(Debug, Subcommand)]
+pub enum HandoffCommand {
+    /// Create a handoff skeleton with YAML frontmatter and fixed section headers.
+    Create(CreateArgs),
+    /// List handoffs newest-first with Status one-liner.
+    List,
+    /// Display full handoff content.
+    Show(ShowArgs),
+}
+
+#[derive(Debug, Args)]
+pub struct CreateArgs {
+    /// Short topic label for the filename.
+    #[arg(long)]
+    pub topic: String,
+
+    /// Output machine-readable JSON.
+    #[arg(long)]
+    pub json: bool,
+}
+
+#[derive(Debug, Args)]
+pub struct ShowArgs {
+    /// Show the most recent handoff.
+    #[arg(long, conflicts_with = "file")]
+    pub latest: bool,
+
+    /// Path to a specific handoff file.
+    pub file: Option<String>,
+}
+
+#[derive(Debug, Args)]
+pub struct TakeoffArgs {
+    /// Use the most recent handoff.
+    #[arg(long, conflicts_with = "file")]
+    pub latest: bool,
+
+    /// Path to a specific handoff file.
+    pub file: Option<String>,
+
+    /// Output machine-readable JSON.
+    #[arg(long)]
+    pub json: bool,
+}
+
+// ---------------------------------------------------------------------------
+// Data types
+// ---------------------------------------------------------------------------
+
+/// YAML frontmatter parsed from a handoff document.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Frontmatter {
+    pub topic: String,
+    pub date: String,
+    pub project: String,
+    pub branch: String,
+}
+
+/// A single criterion extracted from the `## Criteria` section.
+#[derive(Debug, Clone, Serialize)]
+pub struct Criterion {
+    pub text: String,
+    pub command: Option<String>,
+    pub auto: bool,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub verified: Option<bool>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub output: Option<String>,
+}
+
+/// Result of takeoff verification.
+#[derive(Debug, Serialize)]
+pub struct TakeoffResult {
+    pub file: String,
+    pub frontmatter: Frontmatter,
+    pub criteria: Vec<Criterion>,
+    pub next_steps: String,
+    pub all_passed: bool,
+}
+
+// ---------------------------------------------------------------------------
+// Constants
+// ---------------------------------------------------------------------------
+
+const SECTIONS: &[&str] = &["Status", "Findings", "Done", "Next", "Criteria"];
+
+// ---------------------------------------------------------------------------
+// Handoffs directory
+// ---------------------------------------------------------------------------
+
+/// Returns `~/.score/handoffs/`, creating it if needed.
+fn handoffs_dir() -> Result<PathBuf> {
+    let home = dirs::home_dir().context("Cannot determine home directory")?;
+    let dir = home.join(".score").join("handoffs");
+    if !dir.exists() {
+        std::fs::create_dir_all(&dir)
+            .with_context(|| format!("Failed to create {}", dir.display()))?;
+    }
+    Ok(dir)
+}
+
+// ---------------------------------------------------------------------------
+// Git / project auto-detection
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R5
+/// Detect current git branch. Falls back to "unknown" if not in a git repo.
+fn detect_branch() -> String {
+    std::process::Command::new("git")
+        .args(["rev-parse", "--abbrev-ref", "HEAD"])
+        .output()
+        .ok()
+        .and_then(|o| {
+            if o.status.success() {
+                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
+            } else {
+                None
+            }
+        })
+        .unwrap_or_else(|| "unknown".to_string())
+}
+
+/// Detect project name from cwd basename or .score/config.toml.
+fn detect_project() -> String {
+    // Try to infer from project root
+    if let Ok(root) = crate::find_project_root() {
+        if let Some(name) = root.file_name() {
+            return name.to_string_lossy().to_string();
+        }
+    }
+    std::env::current_dir()
+        .ok()
+        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
+        .unwrap_or_else(|| "unknown".to_string())
+}
+
+// ---------------------------------------------------------------------------
+// Parsing helpers (public for testing)
+// ---------------------------------------------------------------------------
+
+/// Parse YAML frontmatter from a handoff markdown document.
+///
+/// Expects the document to start with `---\n...\n---`.
+pub fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
+    let content = content.trim_start();
+    if !content.starts_with("---") {
+        anyhow::bail!("No YAML frontmatter found");
+    }
+    let after_first = &content[3..];
+    let end = after_first
+        .find("\n---")
+        .context("Unterminated YAML frontmatter")?;
+    let yaml_str = &after_first[..end];
+    let fm: Frontmatter =
+        serde_yaml::from_str(yaml_str).context("Failed to parse frontmatter YAML")?;
+    Ok(fm)
+}
+
+/// Extract sections from markdown into a map of header -> body text.
+pub fn extract_sections(content: &str) -> BTreeMap<String, String> {
+    let mut sections = BTreeMap::new();
+    let mut current_header: Option<String> = None;
+    let mut current_body = String::new();
+
+    // Skip frontmatter
+    let body = skip_frontmatter(content);
+
+    for line in body.lines() {
+        if line.starts_with("## ") {
+            // Save previous section
+            if let Some(header) = current_header.take() {
+                sections.insert(header, current_body.trim().to_string());
+            }
+            current_header = Some(line[3..].trim().to_string());
+            current_body = String::new();
+        } else if current_header.is_some() {
+            current_body.push_str(line);
+            current_body.push('\n');
+        }
+    }
+    // Save last section
+    if let Some(header) = current_header {
+        sections.insert(header, current_body.trim().to_string());
+    }
+
+    sections
+}
+
+/// Extract criteria checkbox items from the `## Criteria` section.
+pub fn extract_criteria(content: &str) -> Vec<Criterion> {
+    let sections = extract_sections(content);
+    let criteria_body = match sections.get("Criteria") {
+        Some(b) => b,
+        None => return vec![],
+    };
+
+    criteria_body
+        .lines()
+        .filter(|line| {
+            let trimmed = line.trim();
+            trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]")
+        })
+        .map(|line| {
+            let trimmed = line.trim();
+            // Remove checkbox prefix
+            let text = if trimmed.starts_with("- [ ] ") {
+                &trimmed[6..]
+            } else if trimmed.starts_with("- [x] ") {
+                &trimmed[6..]
+            } else if trimmed.starts_with("- [ ]") {
+                &trimmed[5..]
+            } else {
+                &trimmed[5..]
+            };
+            let text = text.trim().to_string();
+            let command = extract_command(&text);
+            let auto = command.is_some();
+            Criterion {
+                text,
+                command,
+                auto,
+                verified: None,
+                output: None,
+            }
+        })
+        .collect()
+}
+
+/// Extract a backtick-wrapped command from a criterion text.
+///
+/// Looks for text wrapped in single backticks: `` `some command` ``.
+pub fn extract_command(text: &str) -> Option<String> {
+    let start = text.find('`')?;
+    let rest = &text[start + 1..];
+    let end = rest.find('`')?;
+    let cmd = rest[..end].trim();
+    if cmd.is_empty() {
+        None
+    } else {
+        Some(cmd.to_string())
+    }
+}
+
+/// Skip YAML frontmatter and return the rest of the document.
+fn skip_frontmatter(content: &str) -> &str {
+    let trimmed = content.trim_start();
+    if !trimmed.starts_with("---") {
+        return content;
+    }
+    let after_first = &trimmed[3..];
+    match after_first.find("\n---") {
+        Some(end) => {
+            let rest = &after_first[end + 4..];
+            // Skip the newline after closing ---
+            rest.strip_prefix('\n').unwrap_or(rest)
+        }
+        None => content,
+    }
+}
+
+// ---------------------------------------------------------------------------
+// Dispatch
+// ---------------------------------------------------------------------------
+
+pub async fn run_handoff(args: HandoffArgs) -> Result<()> {
+    match args.command {
+        HandoffCommand::Create(a) => run_create(a).await,
+        HandoffCommand::List => run_list().await,
+        HandoffCommand::Show(a) => run_show(a).await,
+    }
+}
+
+pub async fn run_takeoff(args: TakeoffArgs) -> Result<()> {
+    run_takeoff_impl(args).await
+}
+
+// ---------------------------------------------------------------------------
+// Create
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R1
+async fn run_create(args: CreateArgs) -> Result<()> {
+    let dir = handoffs_dir()?;
+    let date = Local::now().format("%Y%m%d").to_string();
+    let filename = format!("{}-{}.md", date, args.topic);
+    let path = dir.join(&filename);
+
+    let branch = detect_branch();
+    let project = detect_project();
+
+    let frontmatter = Frontmatter {
+        topic: args.topic.clone(),
+        date: date.clone(),
+        project,
+        branch,
+    };
+
+    let yaml = serde_yaml::to_string(&frontmatter).context("Failed to serialize frontmatter")?;
+    let mut doc = String::new();
+    doc.push_str("---\n");
+    doc.push_str(yaml.trim());
+    doc.push_str("\n---\n\n");
+
+    for section in SECTIONS {
+        doc.push_str(&format!("## {}\n\n\n", section));
+    }
+
+    std::fs::write(&path, &doc)
+        .with_context(|| format!("Failed to write {}", path.display()))?;
+
+    if args.json {
+        let result = serde_json::json!({
+            "path": path.display().to_string(),
+            "topic": args.topic,
+            "date": date,
+            "next_action": "Fill sections: Status, Findings, Done, Next, Criteria",
+        });
+        println!("{}", serde_json::to_string_pretty(&result)?);
+    } else {
+        println!("Created {}", path.display());
+        println!("Next: fill sections — Status, Findings, Done, Next, Criteria");
+    }
+    Ok(())
+}
+
+// ---------------------------------------------------------------------------
+// List
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R2
+async fn run_list() -> Result<()> {
+    let dir = handoffs_dir()?;
+    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)?
+        .filter_map(|e| e.ok())
+        .map(|e| e.path())
+        .filter(|p| p.extension().is_some_and(|e| e == "md"))
+        .collect();
+
+    // Sort newest-first by filename (YYYYMMDD prefix ensures lexical = chronological)
+    entries.sort();
+    entries.reverse();
+
+    if entries.is_empty() {
+        println!("No handoffs found in {}", dir.display());
+        return Ok(());
+    }
+
+    println!("{} handoff(s) in {}\n", entries.len(), dir.display());
+    for entry in &entries {
+        let name = entry.file_name().unwrap_or_default().to_string_lossy();
+        let status = read_status_oneliner(entry);
+        println!("  {} — {}", name, status);
+    }
+    Ok(())
+}
+
+/// Read the first non-empty line from the `## Status` section as a one-liner.
+fn read_status_oneliner(path: &Path) -> String {
+    let content = match std::fs::read_to_string(path) {
+        Ok(c) => c,
+        Err(_) => return "(unreadable)".to_string(),
+    };
+    let sections = extract_sections(&content);
+    sections
+        .get("Status")
+        .and_then(|body| {
+            body.lines()
+                .find(|l| !l.trim().is_empty())
+                .map(|l| l.trim().to_string())
+        })
+        .unwrap_or_else(|| "(empty)".to_string())
+}
+
+// ---------------------------------------------------------------------------
+// Show
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R3
+async fn run_show(args: ShowArgs) -> Result<()> {
+    let path = resolve_handoff_path(args.latest, args.file.as_deref())?;
+    let content = std::fs::read_to_string(&path)
+        .with_context(|| format!("Failed to read {}", path.display()))?;
+    println!("{}", content);
+    Ok(())
+}
+
+// ---------------------------------------------------------------------------
+// Takeoff
+// ---------------------------------------------------------------------------
+
+// @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R4
+async fn run_takeoff_impl(args: TakeoffArgs) -> Result<()> {
+    let path = resolve_handoff_path(args.latest, args.file.as_deref())?;
+    let content = std::fs::read_to_string(&path)
+        .with_context(|| format!("Failed to read {}", path.display()))?;
+
+    let frontmatter = parse_frontmatter(&content)?;
+    let mut criteria = extract_criteria(&content);
+    let sections = extract_sections(&content);
+    let next_steps = sections
+        .get("Next")
+        .cloned()
+        .unwrap_or_default();
+
+    // Auto-verify criteria with backtick commands
+    for criterion in &mut criteria {
+        if let Some(ref cmd) = criterion.command {
+            let result = std::process::Command::new("sh")
+                .arg("-c")
+                .arg(cmd)
+                .output();
+            match result {
+                Ok(output) => {
+                    criterion.verified = Some(output.status.success());
+                    let combined = format!(
+                        "{}{}",
+                        String::from_utf8_lossy(&output.stdout),
+                        String::from_utf8_lossy(&output.stderr)
+                    );
+                    let trimmed = combined.trim();
+                    if !trimmed.is_empty() {
+                        // Truncate long output for JSON brevity
+                        let truncated = if trimmed.len() > 500 {
+                            format!("{}... (truncated)", &trimmed[..500])
+                        } else {
+                            trimmed.to_string()
+                        };
+                        criterion.output = Some(truncated);
+                    }
+                }
+                Err(e) => {
+                    criterion.verified = Some(false);
+                    criterion.output = Some(format!("Failed to execute: {}", e));
+                }
+            }
+        }
+    }
+
+    let all_passed = criteria.iter().all(|c| {
+        if c.auto {
+            c.verified == Some(true)
+        } else {
+            true // Manual criteria don't block
+        }
+    });
+
+    let takeoff_result = TakeoffResult {
+        file: path.display().to_string(),
+        frontmatter,
+        criteria: criteria.clone(),
+        next_steps: next_steps.clone(),
+        all_passed,
+    };
+
+    if args.json {
+        println!("{}", serde_json::to_string_pretty(&takeoff_result)?);
+    } else {
+        println!("Takeoff: {}\n", path.display());
+
+        // Criteria results
+        if criteria.is_empty() {
+            println!("No criteria found.\n");
+        } else {
+            println!("Criteria:");
+            for c in &criteria {
+                let icon = if c.auto {
+                    match c.verified {
+                        Some(true) => "PASS",
+                        Some(false) => "FAIL",
+                        None => "????",
+                    }
+                } else {
+                    "MANUAL"
+                };
+                println!("  [{}] {}", icon, c.text);
+            }
+            println!();
+        }
+
+        // Summary
+        if all_passed {
+            println!("All auto criteria passed.\n");
+        } else {
+            println!("Some criteria failed.\n");
+        }
+
+        // Next steps
+        if !next_steps.is_empty() {
+            println!("Next steps:\n{}", next_steps);
+        }
+    }
+    Ok(())
+}
+
+// ---------------------------------------------------------------------------
+// Helpers
+// ---------------------------------------------------------------------------
+
+/// Resolve a handoff file path from --latest flag or explicit file argument.
+fn resolve_handoff_path(latest: bool, file: Option<&str>) -> Result<PathBuf> {
+    if let Some(f) = file {
+        let p = PathBuf::from(f);
+        if p.exists() {
+            return Ok(p);
+        }
+        // Try inside handoffs dir
+        let dir = handoffs_dir()?;
+        let in_dir = dir.join(f);
+        if in_dir.exists() {
+            return Ok(in_dir);
+        }
+        anyhow::bail!("Handoff file not found: {}", f);
+    }
+
+    if latest {
+        return find_latest_handoff();
+    }
+
+    // Default to latest
+    find_latest_handoff()
+}
+
+/// Find the most recent handoff file by filename sort.
+fn find_latest_handoff() -> Result<PathBuf> {
+    let dir = handoffs_dir()?;
+    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)?
+        .filter_map(|e| e.ok())
+        .map(|e| e.path())
+        .filter(|p| p.extension().is_some_and(|e| e == "md"))
+        .collect();
+
+    entries.sort();
+    entries
+        .last()
+        .cloned()
+        .context("No handoff files found in ~/.score/handoffs/")
+}
+
+// ---------------------------------------------------------------------------
+// Tests
+// ---------------------------------------------------------------------------
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    const SAMPLE_DOC: &str = r#"---
+topic: statemanager
+date: "20260412"
+project: cclab
+branch: main
+---
+
+## Status
+
+Refactoring state machine — 60% done
+
+## Findings
+
+- Found race condition in concurrent access
+
+## Done
+
+- Rewrote StateManager::load
+
+## Next
+
+1. Add integration tests
+2. Run `cargo test -p sdd`
+
+## Criteria
+
+- [ ] `cargo test -p sdd --lib` passes
+- [ ] `cargo check` passes
+- [ ] no regressions
+- [x] already done item
+"#;
+
+    // REQ: REQ-R5
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R5
+    #[test]
+    fn test_parse_frontmatter() {
+        let fm = parse_frontmatter(SAMPLE_DOC).unwrap();
+        assert_eq!(fm.topic, "statemanager");
+        assert_eq!(fm.date, "20260412");
+        assert_eq!(fm.project, "cclab");
+        assert_eq!(fm.branch, "main");
+    }
+
+    // REQ: REQ-R4
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R4
+    #[test]
+    fn test_extract_criteria() {
+        let criteria = extract_criteria(SAMPLE_DOC);
+        assert_eq!(criteria.len(), 4);
+
+        // First: auto-verifiable
+        assert_eq!(criteria[0].text, "`cargo test -p sdd --lib` passes");
+        assert_eq!(
+            criteria[0].command.as_deref(),
+            Some("cargo test -p sdd --lib")
+        );
+        assert!(criteria[0].auto);
+
+        // Second: auto-verifiable
+        assert_eq!(criteria[1].text, "`cargo check` passes");
+        assert_eq!(criteria[1].command.as_deref(), Some("cargo check"));
+        assert!(criteria[1].auto);
+
+        // Third: manual (no backtick command)
+        assert_eq!(criteria[2].text, "no regressions");
+        assert!(criteria[2].command.is_none());
+        assert!(!criteria[2].auto);
+
+        // Fourth: already checked item (still parsed)
+        assert_eq!(criteria[3].text, "already done item");
+        assert!(!criteria[3].auto);
+    }
+
+    // REQ: REQ-R4
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R4
+    #[test]
+    fn test_extract_command() {
+        assert_eq!(
+            extract_command("`cargo test -p sdd --lib` passes"),
+            Some("cargo test -p sdd --lib".to_string())
+        );
+        assert_eq!(
+            extract_command("`cargo check` passes"),
+            Some("cargo check".to_string())
+        );
+        assert_eq!(extract_command("no backticks here"), None);
+        assert_eq!(extract_command("empty `` backticks"), None);
+    }
+
+    // REQ: REQ-R4
+    // @spec .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md#R4
+    #[test]
+    fn test_extract_sections() {
+        let sections = extract_sections(SAMPLE_DOC);
+        assert!(sections.contains_key("Status"));
+        assert!(sections.contains_key("Findings"));
+        assert!(sections.contains_key("Done"));
+        assert!(sections.contains_key("Next"));
+        assert!(sections.contains_key("Criteria"));
+        assert_eq!(sections.len(), 5);
+
+        assert!(sections["Status"].contains("60% done"));
+        assert!(sections["Findings"].contains("race condition"));
+    }
+
+    // REQ: REQ-R4
+    #[test]
+    fn test_extract_criteria_empty() {
+        let doc = r#"---
+topic: test
+date: "20260412"
+project: cclab
+branch: main
+---
+
+## Status
+
+In progress
+
+## Next
+
+- do stuff
+"#;
+        let criteria = extract_criteria(doc);
+        assert!(criteria.is_empty());
+    }
+}
diff --git a/projects/score/cli/src/lib.rs b/projects/score/cli/src/lib.rs
index 7c6dcf13..2719ec50 100644
--- a/projects/score/cli/src/lib.rs
+++ b/projects/score/cli/src/lib.rs
@@ -13,6 +13,7 @@ pub mod commands;
 pub mod daemon;
 pub mod direct;
 pub mod fillback;
+pub mod handoff;
 pub mod init;
 pub mod issues;
 pub mod list;
```


## Alignment Warnings

11 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Requirements' at line 18 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Scenarios' at line 63 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Diagrams' at line 89 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'API Spec' at line 171 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Test Plan' at line 214 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'Changes' at line 248 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | missing_section_annotation | Section 'CLI' at line 292 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/projects/score/specs/handoff-takeoff.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
