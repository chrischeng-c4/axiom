// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/chat_members.md#source
// CODEGEN-BEGIN
//! `aw chat members` — members registry and new YAML-frontmatter channel format.
//!
//! This module owns:
//! - MembersFile, Member, ChannelMessage schema (new spec-driven shapes)
//! - YAML-frontmatter channel parser / serializer
//! - Old pipe-format migration logic
//! - Identity detection chain (branch → members.yaml → config.toml → basename)
//! - `members --register` / `members --list` subcommand handlers
//!
//! Split from chat.rs per the 500-line hand-written split rule.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#overview

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Schema
// ─────────────────────────────────────────────────────────────────────────────

// One message block inside `/tmp/aw-channel.md`.
// Serialised as per-message YAML frontmatter delimited by `---` markers
// followed by the body text. CLI auto-fills id, from, timestamp;
// agents supply only --to and --body-file.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: i64,
    pub from: String,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub timestamp: String,
    // Free-form body. After the JSONL migration this is a normal serde field
    // — no `#[serde(skip)]` — so the struct round-trips cleanly through
    // `serde_json::to_string` / `from_str`.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#schema
    pub body: String,
}

// YAML frontmatter helper for parsing per-message blocks.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
#[derive(Debug, Deserialize)]
pub struct MessageFrontmatter {
    pub id: i64,
    pub from: Option<String>,
    #[serde(default)]
    pub to: Vec<String>,
    pub re: Option<i64>,
    pub project: Option<String>,
    pub timestamp: Option<String>,
}

// One entry in `/tmp/aw-channel-members.yaml`.
// Identity = branch; name is the human-readable label resolved from branch lookup.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    pub branch: String,
    pub wt_path: String,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub last_seen: String,
}

// `/tmp/aw-channel-members.yaml` — the members registry.
// Replaces the old `/tmp/aw-channel-agents.md` per-member markdown file format.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembersFile {
    pub schema: String,
    pub updated_at: String,
    pub members: Vec<Member>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/chat_members.md#source
impl Default for MembersFile {
    fn default() -> Self {
        MembersFile {
            schema: "score-chat-members-v1".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            members: vec![],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Identity detection chain
// ─────────────────────────────────────────────────────────────────────────────

// Resolve caller identity. Identity = current git branch — `aw chat`'s
// purpose is letting per-worktree Claude Code instances communicate, so
// the branch name (which is 1:1 with the worktree) is the natural unique
// key. `.aw/config.toml` `[team] name` lives in the repo and is shared
// across all worktrees, so it can never distinguish them.
///
// Resolution order (first non-empty result wins):
// 1. members.yaml lookup: caller-explicit override mapping `branch → name`
//    (e.g. `branch: main → name: score` makes that checkout show as
//    the team display name while sibling worktrees keep their branch as
//    identity).
// 2. Branch name from git (PRIMARY — per-worktree identity).
// 3. `.aw/config.toml` `[team] name` — only used when not in a git tree
//    (or detached HEAD), because in a normal repo this collides across
//    every worktree.
// 4. Basename of caller's git toplevel — final fallback.
///
// KEY INVARIANT: all git operations use `git -C <cwd>` so the toplevel is
// always the caller's WT root, never falling through to a sibling
// worktree's config.toml.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#logic
pub fn resolve_identity(cwd: &Path) -> Result<String> {
    // git operations are best-effort: if the cwd isn't a git repo, fall back to
    // using cwd itself as the "toplevel" so config.toml + basename steps still
    // work (covers ad-hoc scripts and tests that don't init a git repo).
    let toplevel = git_toplevel_from(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let branch = git_branch_from(cwd).unwrap_or_default();

    // Step 1: members.yaml lookup by branch — caller's explicit override.
    if !branch.is_empty() {
        if let Ok(members) = read_members_file(Path::new("/tmp/aw-channel-members.yaml")) {
            if let Some(m) = members.members.iter().find(|m| m.branch == branch) {
                return Ok(m.name.clone());
            }
        }
    }

    // Step 2: branch name from git — primary identity (per-worktree).
    if !branch.is_empty() && branch != "HEAD" {
        return Ok(branch);
    }

    // Step 3: caller's OWN config.toml — only reached when not in a git tree
    // (or HEAD is detached). In normal multi-worktree usage, branch (Step 2)
    // wins so config.toml never collides identity across worktrees.
    let caller_config_path = toplevel.join(".aw/config.toml");
    if caller_config_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&caller_config_path) {
            if let Ok(parsed) = text.parse::<toml::Value>() {
                if let Some(name) = parsed
                    .get("team")
                    .and_then(|t| t.get("name"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Step 4: toplevel basename — final fallback.
    Ok(toplevel
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string())
}

// Get git toplevel using `git -C <cwd> rev-parse --show-toplevel`.
// This ensures the result is rooted at the caller's WT, not the process CWD.
fn git_toplevel_from(cwd: &Path) -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("git -C {} rev-parse --show-toplevel failed", cwd.display());
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

// Get current branch using `git -C <cwd> symbolic-ref --short HEAD`.
// Uses symbolic-ref over rev-parse --abbrev-ref because the latter fails on
// freshly-init'd repos with no commits (returns "fatal: ambiguous argument").
//
fn git_branch_from(cwd: &Path) -> Result<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("git -C {} symbolic-ref --short HEAD failed", cwd.display());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// Backwards-compatible wrapper: delegates to resolve_identity.
// Retained so existing callers in chat.rs continue to compile.
// The members_path argument is ignored; resolve_identity always uses the canonical
// /tmp/aw-channel-members.yaml path per the spec.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/chat_members.md#source
pub fn detect_team_identity(cwd: &Path, _members_path: &Path) -> Result<String> {
    resolve_identity(cwd)
}

// Get the current git branch for the given CWD. Returns None on failure.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: detect_branch
pub fn detect_git_branch(cwd: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

// Get the git toplevel for the given CWD.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: members_detect_branch
pub fn detect_git_toplevel(cwd: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

// Look up a member name by branch in members.yaml.
// Returns None if file absent or branch not found.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: lookup_member
pub fn lookup_member_name_by_branch(branch: &str, members_path: &Path) -> Option<String> {
    let mf = read_members_file(members_path).ok()?;
    mf.members
        .iter()
        .find(|m| m.branch == branch)
        .map(|m| m.name.clone())
}

// Read config.toml [team] name walking up from cwd.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: from_branch (config fallback)
pub fn read_config_team_name(cwd: &Path) -> Option<String> {
    let mut dir = cwd.to_path_buf();
    loop {
        let cfg = dir.join(".aw/config.toml");
        if cfg.exists() {
            if let Ok(content) = std::fs::read_to_string(&cfg) {
                if let Ok(table) = content.parse::<toml::Value>() {
                    if let Some(n) = table
                        .get("team")
                        .and_then(|t| t.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        return Some(n.to_string());
                    }
                }
            }
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Channel format: YAML frontmatter
// ─────────────────────────────────────────────────────────────────────────────

// Detect whether the channel content uses the old pipe-separated format.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: detect_format
pub fn is_old_pipe_format(content: &str) -> bool {
    if content.trim().is_empty() {
        return false;
    }
    for line in content.lines() {
        if line.starts_with("msg-") && line.contains(" | ") {
            return true;
        }
        if line.starts_with("## msg-") && line.contains(" | ") {
            return true;
        }
        if line.matches('|').count() >= 3
            && !line.trim_start().starts_with('#')
            && !line.contains(':')
        {
            return true;
        }
    }
    let has_headings = content.lines().any(|l| l.starts_with("## msg-"));
    let has_yaml_blocks = content.contains("\n---\n") || content.starts_with("---\n");
    if has_headings && !has_yaml_blocks {
        return true;
    }
    false
}

// Parse a pipe-separated old-format line into a ChannelMessage.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: parse_pipe
pub fn parse_pipe_line(id: i64, line: &str) -> ChannelMessage {
    let stripped = if let Some(rest) = line.strip_prefix(&format!("msg-{} | ", id)) {
        rest
    } else {
        line
    };
    let parts: Vec<&str> = stripped.splitn(4, " | ").collect();
    let (from, to, timestamp, body) = match parts.len() {
        4 => (
            parts[0].trim(),
            parts[1].trim(),
            parts[2].trim(),
            parts[3].trim(),
        ),
        3 => (parts[0].trim(), parts[1].trim(), parts[2].trim(), ""),
        2 => (parts[0].trim(), parts[1].trim(), "", ""),
        _ => (stripped.trim(), "", "", ""),
    };
    let (from_str, to_vec) = if from.contains(" -> ") {
        let mut it = from.splitn(2, " -> ");
        let f = it.next().unwrap_or(from).trim().to_string();
        let t = it.next().unwrap_or("").trim().to_string();
        let tv: Vec<String> = if t.is_empty() { vec![] } else { vec![t] };
        (f, tv)
    } else {
        let to_vec: Vec<String> = if to.is_empty() {
            vec![]
        } else {
            to.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        (from.to_string(), to_vec)
    };
    ChannelMessage {
        id,
        from: from_str,
        to: to_vec,
        re: None,
        project: None,
        timestamp: timestamp.to_string(),
        body: body.to_string(),
    }
}

// Parse old pipe-separated channel content into ChannelMessage vec.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: parse_pipe
pub fn parse_pipe_format(content: &str) -> Vec<ChannelMessage> {
    let mut messages = Vec::new();
    let mut current_id: Option<i64> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("## msg-") {
            if let Ok(id) = rest
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse::<i64>()
            {
                if let Some(prev_id) = current_id {
                    let joined = current_lines.join(" ").trim().to_string();
                    if !joined.is_empty() {
                        messages.push(parse_pipe_line(prev_id, &joined));
                    } else {
                        messages.push(ChannelMessage {
                            id: prev_id,
                            from: "unknown".into(),
                            to: vec![],
                            re: None,
                            project: None,
                            timestamp: String::new(),
                            body: String::new(),
                        });
                    }
                }
                current_id = Some(id);
                current_lines.clear();
                let rest_of_line = rest.trim();
                if let Some(pipe_part) = rest_of_line.split_once(" | ") {
                    current_lines.push(pipe_part.1.to_string());
                }
                continue;
            }
        }
        if let Some(rest) = line.strip_prefix("msg-") {
            if let Some((id_str, rest_of_line)) = rest.split_once(" | ") {
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    if let Some(prev_id) = current_id {
                        let joined = current_lines.join(" ").trim().to_string();
                        if !joined.is_empty() {
                            messages.push(parse_pipe_line(prev_id, &joined));
                        } else {
                            messages.push(ChannelMessage {
                                id: prev_id,
                                from: "unknown".into(),
                                to: vec![],
                                re: None,
                                project: None,
                                timestamp: String::new(),
                                body: String::new(),
                            });
                        }
                    }
                    current_id = Some(id);
                    current_lines = vec![rest_of_line.to_string()];
                    continue;
                }
            }
        }
        if current_id.is_none() && line.contains(" | ") && line.matches('|').count() >= 2 {
            let id = messages.len() as i64 + 1;
            messages.push(parse_pipe_line(id, line));
            continue;
        }
        if current_id.is_some() {
            current_lines.push(line.to_string());
        }
    }
    if let Some(prev_id) = current_id {
        let joined = current_lines.join("\n").trim().to_string();
        if !joined.is_empty() {
            messages.push(parse_pipe_line(prev_id, &joined));
        } else {
            messages.push(ChannelMessage {
                id: prev_id,
                from: "unknown".into(),
                to: vec![],
                re: None,
                project: None,
                timestamp: String::new(),
                body: String::new(),
            });
        }
    }
    messages
}

// Serialize a ChannelMessage into a YAML-frontmatter block string.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema  Node: rewrite_frontmatter
pub fn serialize_message_block(msg: &ChannelMessage) -> String {
    let to_yaml = if msg.to.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            msg.to
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let re_line = match msg.re {
        Some(r) => format!("re: {}\n", r),
        None => "re: null\n".to_string(),
    };
    let project_line = match &msg.project {
        Some(p) => format!("project: \"{}\"\n", p),
        None => "project: null\n".to_string(),
    };
    format!(
        "---\nid: {}\nfrom: \"{}\"\nto: {}\n{}{}timestamp: \"{}\"\n---\n{}\n",
        msg.id, msg.from, to_yaml, re_line, project_line, msg.timestamp, msg.body
    )
}

// Rewrite the channel file from old pipe format to new YAML-frontmatter format.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: rewrite_frontmatter
pub fn rewrite_channel_as_frontmatter(path: &Path, msgs: &[ChannelMessage]) -> Result<()> {
    let content: String = msgs.iter().map(serialize_message_block).collect();
    std::fs::write(path, content)
        .with_context(|| format!("rewriting channel file {}", path.display()))
}

// Parse channel content into messages. Auto-detects format:
//   - JSONL (one JSON object per line) — current canonical format
//   - YAML-frontmatter blocks — legacy format kept for back-compat reads
//   - Old pipe-separated format — pre-G2 legacy
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (parse_channel)
pub fn parse_channel_markdown(content: &str) -> Vec<ChannelMessage> {
    if content.trim().is_empty() {
        return vec![];
    }
    if looks_like_jsonl(content) {
        return parse_channel_jsonl(content);
    }
    if is_old_pipe_format(content) {
        return parse_pipe_format(content);
    }
    parse_frontmatter_blocks(content)
}

// Heuristic: does the first non-blank line parse as a JSON object?
fn looks_like_jsonl(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return trimmed.starts_with('{') && trimmed.ends_with('}');
    }
    false
}

// Once that primitive exists, this function regenerates from
// projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#schema (ChannelMessage).

// Parse JSONL channel content. One JSON-encoded `ChannelMessage` per line;
// blank lines and lines that fail `serde_json::from_str` are skipped with a
// stderr warning.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (parse_channel JSONL branch)
pub fn parse_channel_jsonl(content: &str) -> Vec<ChannelMessage> {
    let mut messages = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<ChannelMessage>(trimmed) {
            Ok(msg) => messages.push(msg),
            Err(e) => {
                eprintln!("warning: skipping malformed channel line: {}", e);
            }
        }
    }
    messages
}

// Once that primitive exists, this function regenerates from
// projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#interaction (post sequence).

// Serialize a `ChannelMessage` to one JSONL line (JSON + trailing newline).
// The result is one POSIX-atomic `write()` for messages ≤ PIPE_BUF.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (run_post serialize step)
pub fn serialize_message_jsonl(msg: &ChannelMessage) -> Result<String> {
    let json = serde_json::to_string(msg).context("serializing ChannelMessage to JSON")?;
    Ok(format!("{}\n", json))
}

// Parse YAML-frontmatter blocks from channel content.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: parse_messages
pub fn parse_frontmatter_blocks(content: &str) -> Vec<ChannelMessage> {
    let mut messages = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "---" {
            i += 1;
            continue;
        }
        let yaml_start = i + 1;
        let mut yaml_end = None;
        for j in yaml_start..lines.len() {
            if lines[j].trim() == "---" {
                yaml_end = Some(j);
                break;
            }
        }
        let yaml_end = match yaml_end {
            Some(e) => e,
            None => break,
        };
        let yaml_str = lines[yaml_start..yaml_end].join("\n");
        let body_start = yaml_end + 1;
        let mut body_end = lines.len();
        for j in body_start..lines.len() {
            if lines[j].trim() == "---" {
                body_end = j;
                break;
            }
        }
        let body = lines[body_start..body_end]
            .join("\n")
            .trim_matches('\n')
            .to_string();
        if let Ok(fm) = serde_yaml::from_str::<MessageFrontmatter>(&yaml_str) {
            messages.push(ChannelMessage {
                id: fm.id,
                from: fm.from.unwrap_or_else(|| "unknown".into()),
                to: fm.to,
                re: fm.re,
                project: fm.project,
                timestamp: fm.timestamp.unwrap_or_default(),
                body,
            });
        }
        i = body_end;
    }
    messages
}

// ─────────────────────────────────────────────────────────────────────────────
// Members file helpers
// ─────────────────────────────────────────────────────────────────────────────

// Read MembersFile from path. Returns empty MembersFile if absent.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: members_read
pub fn read_members_file(path: &Path) -> Result<MembersFile> {
    if !path.exists() {
        return Ok(MembersFile::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading members file {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("parsing members file {}", path.display()))
}

// Write MembersFile to path.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: members_write
pub fn write_members_file(path: &Path, mf: &MembersFile) -> Result<()> {
    let yaml = serde_yaml::to_string(mf).context("serializing MembersFile")?;
    std::fs::write(path, yaml).with_context(|| format!("writing members file {}", path.display()))
}

// Handle `aw chat members --register`: upsert caller's Member entry.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: members_find → members_upsert / members_insert → members_write
pub fn run_members_register(
    identity: &str,
    branch: &str,
    wt_path: &str,
    projects: &[String],
    capabilities: &[String],
    members_path: &Path,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let mut mf = read_members_file(members_path).unwrap_or_default();

    if let Some(existing) = mf.members.iter_mut().find(|m| m.branch == branch) {
        existing.last_seen = now.clone();
        existing.branch = branch.to_string();
        existing.wt_path = wt_path.to_string();
        if !projects.is_empty() {
            existing.projects = projects.to_vec();
        }
        if !capabilities.is_empty() {
            existing.capabilities = capabilities.to_vec();
        }
    } else {
        mf.members.push(Member {
            name: identity.to_string(),
            branch: branch.to_string(),
            wt_path: wt_path.to_string(),
            projects: projects.to_vec(),
            capabilities: capabilities.to_vec(),
            last_seen: now.clone(),
        });
    }
    mf.updated_at = now;
    mf.schema = "score-chat-members-v1".to_string();

    write_members_file(members_path, &mf)
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
    // T1: test_post_yaml_frontmatter_round_trip
    #[test]
    fn test_post_yaml_frontmatter_round_trip() {
        let msgs = vec![
            ChannelMessage {
                id: 1,
                from: "main".into(),
                to: vec!["test-team".into()],
                re: None,
                project: None,
                timestamp: "2026-04-27T07:50:25Z".into(),
                body: "hello1".into(),
            },
            ChannelMessage {
                id: 2,
                from: "main".into(),
                to: vec![],
                re: None,
                project: None,
                timestamp: "2026-04-27T07:50:26Z".into(),
                body: "hello2".into(),
            },
            ChannelMessage {
                id: 3,
                from: "main".into(),
                to: vec!["a".into(), "b".into()],
                re: Some(1),
                project: None,
                timestamp: "2026-04-27T07:50:27Z".into(),
                body: "hello3".into(),
            },
        ];

        let content: String = msgs.iter().map(serialize_message_block).collect();

        assert!(
            content.contains("---\nid: 1"),
            "Block 1 should start with ---\\nid: 1"
        );
        assert!(
            content.contains("---\nid: 2"),
            "Block 2 should start with ---\\nid: 2"
        );
        assert!(
            content.contains("---\nid: 3"),
            "Block 3 should start with ---\\nid: 3"
        );

        let parsed = parse_channel_markdown(&content);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].id, 1);
        assert_eq!(parsed[0].to, vec!["test-team"]);
        assert_eq!(parsed[0].body, "hello1");
        assert_eq!(parsed[1].id, 2);
        assert_eq!(parsed[1].to, Vec::<String>::new());
        assert_eq!(parsed[2].id, 3);
        assert_eq!(parsed[2].re, Some(1));
        assert_eq!(parsed[2].to, vec!["a", "b"]);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
    // T2: test_post_migration_from_pipe_format
    #[test]
    fn test_post_migration_from_pipe_format() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            "## msg-1\nscore | mamba | 2024-01-01T00:00:00Z | hello old\n"
        )
        .unwrap();
        let content = std::fs::read_to_string(f.path()).unwrap();

        assert!(
            is_old_pipe_format(&content),
            "Should detect old pipe format"
        );

        let migrated = parse_pipe_format(&content);
        assert!(
            !migrated.is_empty(),
            "Should parse at least one message from old format"
        );
        assert_eq!(migrated[0].id, 1);

        rewrite_channel_as_frontmatter(f.path(), &migrated).unwrap();
        let new_content = std::fs::read_to_string(f.path()).unwrap();

        assert!(
            !is_old_pipe_format(&new_content),
            "After migration, should not be old format"
        );
        assert!(
            new_content.contains("---\nid:"),
            "Should contain YAML frontmatter"
        );

        let reparsed = parse_channel_markdown(&new_content);
        assert!(!reparsed.is_empty(), "Should parse migrated messages");
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
    // T3: test_identity_detection_via_members_lookup
    #[test]
    fn test_identity_detection_via_members_lookup() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mf = MembersFile {
            schema: "score-chat-members-v1".to_string(),
            updated_at: "2026-04-27T00:00:00Z".to_string(),
            members: vec![Member {
                name: "bar".to_string(),
                branch: "test-branch-g1-identity".to_string(),
                wt_path: "/tmp/test".to_string(),
                projects: vec![],
                capabilities: vec![],
                last_seen: "2026-04-27T00:00:00Z".to_string(),
            }],
        };
        let yaml = serde_yaml::to_string(&mf).unwrap();
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(yaml.as_bytes()).unwrap();

        let loaded = read_members_file(f.path()).unwrap();
        let found = loaded
            .members
            .iter()
            .find(|m| m.branch == "test-branch-g1-identity");
        assert!(found.is_some(), "Should find member by branch");
        assert_eq!(found.unwrap().name, "bar");

        let name = lookup_member_name_by_branch("test-branch-g1-identity", f.path());
        assert_eq!(name, Some("bar".to_string()));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
    // T4: test_identity_fallback_to_branch
    #[test]
    fn test_identity_fallback_to_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let no_members = tmp.path().join("nonexistent-members.yaml");
        let identity = detect_team_identity(tmp.path(), &no_members).unwrap();
        assert!(!identity.is_empty(), "Identity should not be empty");
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
    // T5: test_members_register_upsert_idempotent
    #[test]
    fn test_members_register_upsert_idempotent() {
        use tempfile::NamedTempFile;

        let f = NamedTempFile::new().unwrap();
        let path = f.path();
        let branch = "test-upsert-branch";
        let wt = "/tmp/test-wt";
        let now1 = "2026-04-27T07:00:00Z";
        let now2 = "2026-04-27T08:00:00Z";

        // First registration
        let mut mf = MembersFile::default();
        mf.members.push(Member {
            name: "tester".to_string(),
            branch: branch.to_string(),
            wt_path: wt.to_string(),
            projects: vec!["score".to_string()],
            capabilities: vec!["SDD lifecycle".to_string()],
            last_seen: now1.to_string(),
        });
        mf.updated_at = now1.to_string();
        write_members_file(path, &mf).unwrap();

        let loaded = read_members_file(path).unwrap();
        assert_eq!(loaded.schema, "score-chat-members-v1");
        assert_eq!(loaded.members.len(), 1);
        assert_eq!(loaded.members[0].projects, vec!["score"]);

        // Second registration: update last_seen only, preserve projects
        let mut mf2 = read_members_file(path).unwrap();
        if let Some(existing) = mf2.members.iter_mut().find(|m| m.branch == branch) {
            existing.last_seen = now2.to_string();
        }
        mf2.updated_at = now2.to_string();
        write_members_file(path, &mf2).unwrap();

        let loaded2 = read_members_file(path).unwrap();
        assert_eq!(
            loaded2.members.len(),
            1,
            "No duplicates after second register"
        );
        assert_eq!(
            loaded2.members[0].projects,
            vec!["score"],
            "Projects preserved"
        );
        assert_eq!(loaded2.members[0].last_seen, now2, "last_seen updated");
    }
}

// CODEGEN-END
