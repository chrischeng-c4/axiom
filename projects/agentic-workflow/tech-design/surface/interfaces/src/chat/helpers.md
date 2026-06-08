---
id: projects-score-src-chat-helpers-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized projects/agentic-workflow/src/cli/chat/helpers.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/chat/helpers.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OutputFormat` | projects/agentic-workflow/src/cli/chat/helpers.rs | enum | pub | 28 |  |
| `detect_output_format` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 37 | detect_output_format(terse: bool, human: bool) -> OutputFormat |
| `detect_team_identity` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 60 | detect_team_identity(cwd: &Path) -> Result<String> |
| `format_human` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 231 | format_human(msgs: &[ChannelMessage]) -> String |
| `format_terse` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 209 | format_terse(msgs: &[ChannelMessage]) -> String |
| `parse_channel` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 112 | parse_channel(path: &Path) -> Vec<ChannelMessage> |
| `parse_channel_markdown` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 123 | parse_channel_markdown(content: &str) -> Vec<ChannelMessage> |
| `replace_agent_section` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 687 | replace_agent_section(existing: &str, name: &str, yaml_block: &str) -> String |
| `run_agents` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 427 | run_agents(args: AgentsArgs, identity: &str) -> Result<()> |
| `run_list` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 340 | run_list(args: ListArgs, identity: &str) -> Result<()> |
| `run_listen` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 524 | run_listen(args: ListenArgs, identity: &str) -> Result<()> |
| `run_post` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 260 | run_post(args: PostArgs, identity: &str) -> Result<()> |
| `run_read` | projects/agentic-workflow/src/cli/chat/helpers.rs | function | pub | 381 | run_read(args: ReadArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/chat/helpers.rs -->
```rust
//! Imperative implementation helpers for `aw chat` subcommands.
//!
//! This module contains all hand-written logic: identity detection, channel
//! parsing, output formatting, and per-subcommand handlers.
//!
//! /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use super::{
    AGENTS_PATH, AgentLastSeen, AgentRegistration, AgentsArgs, CHANNEL_PATH, ChannelMessage,
    ListArgs, ListenArgs, ListenState, PostArgs, ReadArgs,
};

// ─────────────────────────────────────────────────────────────────────────────
// Output format
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/chat/helpers.md#source
pub enum OutputFormat {
    Human,
    Terse,
}

// Detect output format from TTY status and flag overrides.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: detect_format
pub fn detect_output_format(terse: bool, human: bool) -> OutputFormat {
    if terse {
        return OutputFormat::Terse;
    }
    if human {
        return OutputFormat::Human;
    }
    if io::stdout().is_terminal() {
        OutputFormat::Human
    } else {
        OutputFormat::Terse
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// Identity detection
// ─────────────────────────────────────────────────────────────────────────────

// Walk up from `cwd` looking for `.aw/config.toml` with a `[team]` block.
// Returns `[team] name` if found; falls back to git-toplevel basename; falls
// back to CWD basename.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: detect_identity
pub fn detect_team_identity(cwd: &Path) -> Result<String> {
    let mut dir = cwd.to_path_buf();
    loop {
        let config_path = dir.join(".aw/config.toml");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(table) = content.parse::<toml::Value>() {
                    if let Some(name) = table
                        .get("team")
                        .and_then(|t| t.get("name"))
                        .and_then(|n| n.as_str())
                    {
                        return Ok(name.to_string());
                    }
                }
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    // Fallback: git toplevel basename.
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(basename) = Path::new(&toplevel).file_name() {
                return Ok(basename.to_string_lossy().to_string());
            }
        }
    }
    // Final fallback: CWD basename.
    Ok(cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Channel parsing
// ─────────────────────────────────────────────────────────────────────────────

// Parse `/tmp/aw-channel.md` into a `Vec<ChannelMessage>`.
///
// Hand-coded substitute for the deferred `parse_markdown` primitive (#18).
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_list (parse step); @spec-primitive parse_markdown (deferred)
pub fn parse_channel(path: &Path) -> Vec<ChannelMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    parse_channel_markdown(&content)
}

// Parse channel markdown content into messages.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
pub fn parse_channel_markdown(content: &str) -> Vec<ChannelMessage> {
    let mut messages = Vec::new();
    let mut sections: Vec<(u64, &str)> = Vec::new();
    let mut last_id: Option<u64> = None;
    let mut last_start = 0usize;

    for (i, line) in content.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("## msg-") {
            if let Ok(id) = rest.trim().parse::<u64>() {
                if let Some(prev_id) = last_id {
                    let byte_offset = content.lines().take(i).map(|l| l.len() + 1).sum::<usize>();
                    sections.push((prev_id, &content[last_start..byte_offset]));
                    last_start = byte_offset;
                }
                last_id = Some(id);
            }
        }
    }
    if let Some(id) = last_id {
        sections.push((id, &content[last_start..]));
    }

    for (id, section_text) in sections {
        if let Some(msg) = parse_message_section(id, section_text) {
            messages.push(msg);
        }
    }
    messages
}

fn parse_message_section(id: u64, section_text: &str) -> Option<ChannelMessage> {
    let after_heading = section_text.lines().skip(1).collect::<Vec<_>>().join("\n");
    let after_heading = after_heading.trim_start();

    let (yaml_str, body) = if after_heading.starts_with("---") {
        let rest = &after_heading[3..];
        if let Some(end) = rest.find("\n---") {
            let yaml_part = &rest[..end];
            let body_part = rest[end + 4..].trim_start();
            (yaml_part.to_string(), body_part.to_string())
        } else {
            (String::new(), after_heading.to_string())
        }
    } else {
        (String::new(), after_heading.to_string())
    };

    if yaml_str.is_empty() {
        return Some(ChannelMessage {
            id,
            from: "unknown".to_string(),
            to: vec![],
            re: None,
            timestamp: String::new(),
            body,
        });
    }

    #[derive(Deserialize)]
    struct MsgMeta {
        from: Option<String>,
        #[serde(default)]
        to: Vec<String>,
        re: Option<u64>,
        timestamp: Option<String>,
    }

    let meta: MsgMeta = serde_yaml::from_str(&yaml_str).ok()?;
    Some(ChannelMessage {
        id,
        from: meta.from.unwrap_or_else(|| "unknown".to_string()),
        to: meta.to,
        re: meta.re,
        timestamp: meta.timestamp.unwrap_or_default(),
        body,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Output formatting
// ─────────────────────────────────────────────────────────────────────────────

// Format messages in terse single-line-per-message format (≤¼ of human token count).
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: detect_format → terse branch
pub fn format_terse(msgs: &[ChannelMessage]) -> String {
    let mut out = String::new();
    for msg in msgs {
        let to_str = if msg.to.is_empty() {
            "@all".to_string()
        } else {
            msg.to.join(",")
        };
        let re_str = msg.re.map(|r| format!(" re:{}", r)).unwrap_or_default();
        let body_first = msg.body.lines().next().unwrap_or("").trim();
        out.push_str(&format!(
            "msg-{} | {} -> {}{} | {} | {}\n",
            msg.id, msg.from, to_str, re_str, msg.timestamp, body_first
        ));
    }
    out
}

// Format messages in human-readable Markdown.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: detect_format → human branch
pub fn format_human(msgs: &[ChannelMessage]) -> String {
    let mut out = String::new();
    for msg in msgs {
        out.push_str(&format!("## msg-{}\n", msg.id));
        out.push_str(&format!("**From:** {}  \n", msg.from));
        if !msg.to.is_empty() {
            out.push_str(&format!("**To:** {}  \n", msg.to.join(", ")));
        }
        if let Some(re) = msg.re {
            out.push_str(&format!("**Re:** msg-{}  \n", re));
        }
        if !msg.timestamp.is_empty() {
            out.push_str(&format!("**Time:** {}  \n", msg.timestamp));
        }
        out.push('\n');
        out.push_str(&msg.body);
        out.push_str("\n\n---\n\n");
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// post
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat post`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_post
pub fn run_post(args: PostArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);

    // @spec-primitive read_file (body)
    let body = if args.body_file == "-" {
        let mut buf = String::new();
        io::stdin()
            .lock()
            .read_to_string(&mut buf)
            .context("reading body from stdin")?;
        buf
    } else {
        std::fs::read_to_string(&args.body_file)
            .with_context(|| format!("reading body from {}", args.body_file))?
    };
    let body = body.trim().to_string();

    // @spec-primitive read_file (channel for next_msg_id)
    let existing = parse_channel(Path::new(CHANNEL_PATH));

    // next_msg_id: hand-coded (Issue E primitive deferred)
    let next_id = existing.iter().map(|m| m.id).max().unwrap_or(0) + 1;
    let timestamp = Utc::now().to_rfc3339();

    // @spec-primitive format_template (hand-coded)
    let to_yaml = if args.to.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            args.to
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let re_yaml = args.re.map(|r| format!("re: {}\n", r)).unwrap_or_default();

    let block = format!(
        "## msg-{}\n---\nfrom: \"{}\"\nto: {}\n{}timestamp: \"{}\"\n---\n\n{}\n\n",
        next_id, identity, to_yaml, re_yaml, timestamp, body
    );

    // @spec-primitive append_file
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(CHANNEL_PATH)
        .with_context(|| format!("opening channel file {}", CHANNEL_PATH))?;
    file.write_all(block.as_bytes())
        .context("writing to channel file")?;

    let posted = ChannelMessage {
        id: next_id,
        from: identity.to_string(),
        to: args.to,
        re: args.re,
        timestamp,
        body,
    };

    match fmt {
        OutputFormat::Terse => print!("{}", format_terse(&[posted])),
        OutputFormat::Human => {
            println!("Posted msg-{}.", next_id);
            print!("{}", format_human(&[posted]));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// list
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat list`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_list
pub fn run_list(args: ListArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);

    // @spec-primitive read_file + parse (hand-coded parse_markdown substitute)
    let mut msgs = parse_channel(Path::new(CHANNEL_PATH));

    // @spec-primitive filter (hand-coded; Issue E deferred)
    if let Some(ref mentions) = args.mentions {
        let target = if mentions == "@me" {
            identity.to_string()
        } else {
            mentions.clone()
        };
        msgs.retain(|m| m.to.is_empty() || m.to.iter().any(|t| t == &target));
    }
    if let Some(n) = args.last {
        let len = msgs.len();
        if len > n {
            msgs = msgs[len - n..].to_vec();
        }
    }

    // @spec-primitive tty_check → format_terse OR format_human
    print!(
        "{}",
        match fmt {
            OutputFormat::Terse => format_terse(&msgs),
            OutputFormat::Human => format_human(&msgs),
        }
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// read
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat read`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_read
pub fn run_read(args: ReadArgs) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let msgs = parse_channel(Path::new(CHANNEL_PATH));

    // @spec-primitive filter (hand-coded; deferred primitive)
    let thread: Vec<ChannelMessage> = msgs
        .iter()
        .filter(|m| m.id == args.re || m.re == Some(args.re))
        .cloned()
        .collect();

    if thread.is_empty() {
        eprintln!("No messages found for thread anchored at msg-{}.", args.re);
        return Ok(());
    }

    let display: Vec<ChannelMessage> = if args.full {
        thread
    } else {
        thread
            .into_iter()
            .map(|mut m| {
                m.body = m.body.lines().next().unwrap_or("").trim().to_string();
                m
            })
            .collect()
    };

    print!(
        "{}",
        match fmt {
            OutputFormat::Terse => format_terse(&display),
            OutputFormat::Human => format_human(&display),
        }
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// agents
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat agents` — route to --register or --list.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: branch_agents
pub fn run_agents(args: AgentsArgs, identity: &str) -> Result<()> {
    if args.register {
        run_agents_register(&args, identity)
    } else {
        run_agents_list(&args)
    }
}
// Register or replace caller's `AgentRegistration`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_agents_register
fn run_agents_register(args: &AgentsArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (display, capabilities) = read_team_metadata(&cwd);
    let wt_path = cwd.display().to_string();
    let branch = current_git_branch(&cwd);
    let timestamp = Utc::now().to_rfc3339();

    let reg = AgentRegistration {
        name: identity.to_string(),
        display,
        wt_path,
        branch,
        capabilities,
        last_seen: timestamp,
    };

    // @spec-primitive serialize_yaml
    let yaml_block = serde_yaml::to_string(&reg).context("serializing AgentRegistration")?;
    let existing_content = std::fs::read_to_string(AGENTS_PATH).unwrap_or_default();

    // @spec-primitive write_file (idempotent replace)
    let new_content = replace_agent_section(&existing_content, &reg.name, &yaml_block);
    std::fs::write(AGENTS_PATH, new_content)
        .with_context(|| format!("writing agents file {}", AGENTS_PATH))?;

    match fmt {
        OutputFormat::Terse => println!("registered agent-{}", reg.name),
        OutputFormat::Human => println!("Registered `## agent-{}` in {}.", reg.name, AGENTS_PATH),
    }
    Ok(())
}

// List all registered agents.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_agents_list
fn run_agents_list(args: &AgentsArgs) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let agents = parse_agents_file(Path::new(AGENTS_PATH));

    if agents.is_empty() {
        println!("No agents registered. Run `aw chat agents --register` first.");
        return Ok(());
    }

    match fmt {
        OutputFormat::Terse => {
            for a in &agents {
                println!(
                    "agent-{} | {} | branch:{} | caps:[{}] | seen:{}",
                    a.name,
                    a.wt_path,
                    a.branch,
                    a.capabilities.join(","),
                    a.last_seen
                );
            }
        }
        OutputFormat::Human => {
            for a in &agents {
                println!("## agent-{}", a.name);
                if let Some(ref d) = a.display {
                    println!("**Display:** {}  ", d);
                }
                println!("**Checkout:** {}  ", a.wt_path);
                println!("**Branch:** {}  ", a.branch);
                if !a.capabilities.is_empty() {
                    println!("**Capabilities:** {}  ", a.capabilities.join(", "));
                }
                println!("**Last seen:** {}  ", a.last_seen);
                println!();
            }
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// listen
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat listen`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: branch_listen
pub fn run_listen(args: ListenArgs, identity: &str) -> Result<()> {
    if args.once {
        run_listen_once(&args, identity)
    } else {
        run_listen_loop(&args, identity)
    }
}

// Single-poll listen.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_listen_once
fn run_listen_once(args: &ListenArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let mut state = read_listen_state()?;
    let last_id = state.get(identity).map(|e| e.last_seen_msg_id).unwrap_or(0);

    // @spec-primitive read_file + parse
    let msgs = parse_channel(Path::new(CHANNEL_PATH));
    let mut new_msgs: Vec<ChannelMessage> = msgs.into_iter().filter(|m| m.id > last_id).collect();

    // @spec-primitive filter (hand-coded)
    if let Some(ref mentions) = args.mentions {
        if mentions == "@all" {
            new_msgs.retain(|m| m.to.is_empty());
        } else {
            let target = if mentions == "@me" {
                identity.to_string()
            } else {
                mentions.clone()
            };
            new_msgs.retain(|m| m.to.is_empty() || m.to.iter().any(|t| t == &target));
        }
    }

    if !new_msgs.is_empty() {
        print!(
            "{}",
            match fmt {
                OutputFormat::Terse => format_terse(&new_msgs),
                OutputFormat::Human => format_human(&new_msgs),
            }
        );
    }

    // Update state.
    let max_id = new_msgs.iter().map(|m| m.id).max().unwrap_or(last_id);
    let entry = state.entry(identity.to_string()).or_default();
    if max_id > entry.last_seen_msg_id {
        entry.last_seen_msg_id = max_id;
    }
    entry.last_polled_at = Some(Utc::now().to_rfc3339());
    write_listen_state(&state)?;
    Ok(())
}

// Polling loop.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_listen_loop
// @spec-primitive sleep (deferred Issue E primitive — using std::thread::sleep)
fn run_listen_loop(args: &ListenArgs, identity: &str) -> Result<()> {
    eprintln!(
        "Listening on channel (interval: {}s). Press Ctrl-C to stop.",
        args.interval
    );
    loop {
        run_listen_once(args, identity)?;
        // @spec-primitive sleep (std::thread::sleep substitute)
        std::thread::sleep(std::time::Duration::from_secs(args.interval));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Listen state helpers
// ─────────────────────────────────────────────────────────────────────────────

// Read `~/.aw/chat-state.json`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
fn read_listen_state() -> Result<ListenState> {
    let path = listen_state_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    // @spec-primitive read_file + parse_json
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading listen state {}", path.display()))?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

// Write `~/.aw/chat-state.json`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
fn write_listen_state(state: &ListenState) -> Result<()> {
    let path = listen_state_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating dir {}", parent.display()))?;
    }
    // @spec-primitive serialize_json + write_file
    let content = serde_json::to_string_pretty(state).context("serializing listen state")?;
    std::fs::write(&path, content)
        .with_context(|| format!("writing listen state {}", path.display()))?;
    Ok(())
}

fn listen_state_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".aw/chat-state.json"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Agents file helpers
// ─────────────────────────────────────────────────────────────────────────────

// Parse `/tmp/aw-channel-agents.md` into `Vec<AgentRegistration>`.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
fn parse_agents_file(path: &Path) -> Vec<AgentRegistration> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut agents = Vec::new();
    let mut sections: Vec<(String, &str)> = Vec::new();
    let mut last_name: Option<String> = None;
    let mut last_start = 0usize;
    let mut line_offset = 0usize;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("## agent-") {
            let name = rest.trim().to_string();
            if let Some(prev_name) = last_name.take() {
                sections.push((prev_name, &content[last_start..line_offset]));
                last_start = line_offset;
            }
            last_name = Some(name);
        }
        line_offset += line.len() + 1;
    }
    if let Some(name) = last_name {
        sections.push((name, &content[last_start..]));
    }

    for (_name, section_text) in sections {
        let yaml_str = section_text.lines().skip(1).collect::<Vec<_>>().join("\n");
        let yaml_str = yaml_str.trim();
        if yaml_str.is_empty() {
            continue;
        }
        if let Ok(reg) = serde_yaml::from_str::<AgentRegistration>(yaml_str) {
            agents.push(reg);
        }
    }
    agents
}

// Replace `## agent-{name}` section with `new_yaml`. Appends if absent.
///
// /// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
// Node: run_agents_register (write_file / replace step)
pub fn replace_agent_section(existing: &str, name: &str, yaml_block: &str) -> String {
    let heading = format!("## agent-{}", name);
    let new_section = format!("{}\n{}\n", heading, yaml_block.trim_end());

    if !existing.contains(&heading) {
        if existing.is_empty() {
            return new_section;
        }
        return format!("{}\n{}", existing.trim_end(), new_section);
    }

    let mut result = String::new();
    let mut in_target = false;
    let mut replaced = false;

    for line in existing.lines() {
        if line == heading {
            in_target = true;
            if !replaced {
                result.push_str(&new_section);
                replaced = true;
            }
            continue;
        }
        if in_target && line.starts_with("## agent-") {
            in_target = false;
        }
        if !in_target {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Team metadata helpers
// ─────────────────────────────────────────────────────────────────────────────

fn read_team_metadata(cwd: &Path) -> (Option<String>, Vec<String>) {
    let mut dir = cwd.to_path_buf();
    loop {
        let config_path = dir.join(".aw/config.toml");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(table) = content.parse::<toml::Value>() {
                    if let Some(team) = table.get("team") {
                        let display = team
                            .get("display")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let capabilities = team
                            .get("capabilities")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default();
                        return (display, capabilities);
                    }
                }
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    (None, vec![])
}

fn current_git_branch(cwd: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — empty channel
    #[test]
    fn test_parse_channel_markdown_empty() {
        assert!(parse_channel_markdown("").is_empty());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — message id and fields parsed correctly
    #[test]
    fn test_parse_channel_markdown_single_message() {
        let content = "## msg-1\n---\nfrom: \"score\"\nto: [\"mamba\"]\ntimestamp: \"2024-01-01T00:00:00Z\"\n---\n\nhello world\n";
        let msgs = parse_channel_markdown(content);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, 1);
        assert_eq!(msgs[0].from, "score");
        assert_eq!(msgs[0].to, vec!["mamba"]);
        assert!(msgs[0].body.contains("hello world"));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — multiple messages parsed
    #[test]
    fn test_parse_channel_markdown_multiple_messages() {
        let content = "## msg-1\n---\nfrom: \"a\"\nto: []\ntimestamp: \"t1\"\n---\n\nbody1\n\n## msg-2\n---\nfrom: \"b\"\nto: []\ntimestamp: \"t2\"\n---\n\nbody2\n";
        let msgs = parse_channel_markdown(content);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 1);
        assert_eq!(msgs[1].id, 2);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — terse format shorter than human
    #[test]
    fn test_format_terse_shorter_than_human() {
        let msgs = vec![ChannelMessage {
            id: 1,
            from: "score".to_string(),
            to: vec!["mamba".to_string()],
            re: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            body: "hello from score\nmore text\neven more text here for padding and context"
                .to_string(),
        }];
        assert!(format_terse(&msgs).len() < format_human(&msgs).len());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — replace_agent_section replaces existing entry
    #[test]
    fn test_replace_agent_section_replaces() {
        let existing = "## agent-score\nname: score\nwt_path: /tmp/a\nbranch: main\ncapabilities: []\nlast_seen: old\n\n## agent-other\nname: other\n";
        let new_yaml =
            "name: score\nwt_path: /tmp/b\nbranch: feat\ncapabilities: []\nlast_seen: new\n";
        let result = replace_agent_section(existing, "score", new_yaml);
        assert!(result.contains("wt_path: /tmp/b"));
        assert!(result.contains("last_seen: new"));
        assert!(result.contains("## agent-other"));
        assert_eq!(result.matches("## agent-score").count(), 1);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — replace_agent_section appends new entry
    #[test]
    fn test_replace_agent_section_appends() {
        let result = replace_agent_section(
            "",
            "score",
            "name: score\nwt_path: /tmp/b\nbranch: main\ncapabilities: []\nlast_seen: now\n",
        );
        assert!(result.contains("## agent-score"));
        assert!(result.contains("name: score"));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — detect_team_identity fallback
    #[test]
    fn test_detect_team_identity_fallback_to_basename() {
        let tmp = tempfile::tempdir().unwrap();
        let identity = detect_team_identity(tmp.path()).unwrap();
        assert!(!identity.is_empty());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — ListenState serde round-trip
    #[test]
    fn test_listen_state_serde_roundtrip() {
        let mut state: super::super::ListenState = HashMap::new();
        state.insert(
            "score".to_string(),
            AgentLastSeen {
                last_seen_msg_id: 5,
                last_polled_at: Some("2024-01-01T00:00:00Z".to_string()),
            },
        );
        let json = serde_json::to_string(&state).unwrap();
        let back: super::super::ListenState = serde_json::from_str(&json).unwrap();
        assert_eq!(back["score"].last_seen_msg_id, 5);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/chat/helpers.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
