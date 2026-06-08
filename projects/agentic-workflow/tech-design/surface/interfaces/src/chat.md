---
id: projects-score-src-chat-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized projects/agentic-workflow/src/cli/chat.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/chat.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AgentLastSeen` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 187 |  |
| `ChatArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 45 |  |
| `ChatCommand` | projects/agentic-workflow/src/cli/chat.rs | enum | pub | 53 |  |
| `ListArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 107 |  |
| `ListenArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 169 |  |
| `ListenState` | projects/agentic-workflow/src/cli/chat.rs | type | pub | 194 |  |
| `MembersArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 141 |  |
| `PostArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 74 |  |
| `ReadArgs` | projects/agentic-workflow/src/cli/chat.rs | struct | pub | 125 |  |
| `chat_members` | projects/agentic-workflow/src/cli/chat.rs | module | pub | 20 |  |
| `run_chat` | projects/agentic-workflow/src/cli/chat.rs | function | pub | 212 | run_chat(args: ChatArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/chat.rs -->
```rust
//! `aw chat` — cross-worktree agent messaging via a shared JSONL channel.
//!
//! All communication flows through `/tmp/aw-channel.jsonl` (append-only,
//! ephemeral, one JSON-encoded `ChannelMessage` per line). `listen` streams the
//! file via `tail -F` and applies a 4-rule default filter
//! (direct_cue / broadcast / echo / thread_member).
//! Five subcommands: `post`, `list`, `read`, `members`, `listen`.
//!
//! Identity chain: (1) git branch, (2) members.yaml branch→name lookup,
//! (3) config.toml [team] name fallback, (4) branch name fallback, (5) git-toplevel basename.
//!
//! Output format: TTY → human markdown; pipe → terse (token-efficient, ≤¼ human).
//! Both overridable per-invocation with `--terse` / `--human`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#overview

#[path = "chat_members.rs"]
pub mod chat_members;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use chat_members::{
    detect_git_branch, detect_git_toplevel, detect_team_identity, parse_channel_markdown,
    read_members_file, run_members_register, serialize_message_jsonl, ChannelMessage,
};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

const CHANNEL_PATH: &str = "/tmp/aw-channel.jsonl";
const MEMBERS_PATH: &str = "/tmp/aw-channel-members.yaml";
// ─────────────────────────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────

// Top-level args wrapper for `aw chat`. Mirrors `IssuesArgs` pattern.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ChatArgs {
    #[command(subcommand)]
    pub command: ChatCommand,
}

// Available subcommands for `aw chat`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Subcommand)]
pub enum ChatCommand {
    /// Post a new message to the shared channel.
    Post(PostArgs),
    /// List messages from the shared channel.
    List(ListArgs),
    /// Read a message thread by anchor msg-id.
    Read(ReadArgs),
    /// Register or list members.
    Members(MembersArgs),
    /// Poll for new messages addressed to the caller team.
    Listen(ListenArgs),
}

// Tightened args for `aw chat post` (PostArgsV3 per G3 spec).
///
// --from is removed (spoofing vector; identity always auto-detected).
// --to is required unless --all is set (explicit broadcast intent).
// --all broadcasts to all teams (sets to: [] in ChannelMessage); mutually exclusive with --to.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#schema
#[derive(Debug, Args)]
pub struct PostArgs {
    /// Addressee team names (comma-separated). Required unless --all is set.
    /// Mutually exclusive with --all.
    #[arg(
        long,
        value_delimiter = ',',
        required_unless_present = "all",
        conflicts_with = "all"
    )]
    pub to: Vec<String>,
    /// Broadcast to all teams. Sets stored to: [] in ChannelMessage.
    /// Mutually exclusive with --to.
    #[arg(long, conflicts_with = "to")]
    pub all: bool,
    /// Anchor msg-id to reply to.
    #[arg(long)]
    pub re: Option<i64>,
    /// Optional project tag. Written to ChannelMessage.project.
    #[arg(long)]
    pub project: Option<String>,
    /// Path to body file. Use `-` for stdin.
    #[arg(long, default_value = "-")]
    pub body_file: String,
    /// Force terse output regardless of TTY.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human (markdown) output regardless of TTY.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

// Args for `aw chat list`. @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter to messages whose `to:` includes this team. `@me` resolves to caller.
    #[arg(long)]
    pub mentions: Option<String>,
    /// Limit to the N most recent messages.
    #[arg(long)]
    pub last: Option<usize>,
    /// Message status filter. Default is `open`.
    #[arg(long, default_value = "open")]
    pub status: String,
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

// Args for `aw chat read`. @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ReadArgs {
    /// Anchor msg-id. Returns anchor + all replies in thread order.
    #[arg(long)]
    pub re: u64,
    /// Include full body. Default shows first-line summary only.
    #[arg(long)]
    pub full: bool,
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

// Args for `aw chat members`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#schema
#[derive(Debug, Args)]
pub struct MembersArgs {
    /// Write or upsert caller's Member entry in `/tmp/aw-channel-members.yaml`.
    #[arg(long)]
    pub register: bool,
    /// Print all registered members from `/tmp/aw-channel-members.yaml`.
    #[arg(long)]
    pub list: bool,
    /// Project tags this member is active in (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub projects: Vec<String>,
    /// Capability tags (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub capabilities: Vec<String>,
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

// Tightened args for `aw chat listen` (ListenArgsV3 per G3 spec).
///
// --once was removed in G3 (cron-era vestige that races with Monitor over
// shared chat-state.json). --interval was removed in the JSONL migration —
// listen now streams via `tail -F`, so polling cadence has no meaning.
// The listener is always long-running; use Ctrl-C or TaskStop to terminate.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#schema (ListenArgsV4)
#[derive(Debug, Args)]
pub struct ListenArgs {
    /// Override filter identity. @me resolves to caller team. Kept for back-compat.
    /// Mutually exclusive with --all.
    #[arg(long, conflicts_with = "all")]
    pub mentions: Option<String>,
    /// Emit every message read from the tail stream, bypassing the 4-rule default filter.
    /// Intended for debugging. Mutually exclusive with --mentions.
    #[arg(long, default_value = "false", conflicts_with = "mentions")]
    pub all: bool,
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

// Per-team entry inside `~/.aw/chat-state.json`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentLastSeen {
    pub last_seen_msg_id: u64,
    pub last_polled_at: Option<String>,
}

// `~/.aw/chat-state.json` — keyed by team name.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
pub type ListenState = HashMap<String, AgentLastSeen>;

// ─────────────────────────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
    Human,
    Terse,
}

// ─────────────────────────────────────────────────────────────────────────────
// Top-level dispatcher
// ─────────────────────────────────────────────────────────────────────────────

// Top-level dispatcher for `aw chat`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
// Node: start → detect_identity → detect_format → branch_cmd
pub fn run_chat(args: ChatArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let identity = detect_team_identity(&cwd, Path::new(MEMBERS_PATH))?;
    match args.command {
        ChatCommand::Post(a) => run_post(a, &identity),
        ChatCommand::List(a) => run_list(a, &identity),
        ChatCommand::Read(a) => run_read(a),
        ChatCommand::Members(a) => run_members(a, &identity),
        ChatCommand::Listen(a) => run_listen(a, &identity),
    }
}

// Detect output format: TTY→Human, pipe→Terse; flags override.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic  Node: detect_format
fn detect_output_format(terse: bool, human: bool) -> OutputFormat {
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
// Channel helpers
// ─────────────────────────────────────────────────────────────────────────────

// Parse channel file (with migration if old format detected).
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: read_channel → detect_format
fn parse_channel(path: &Path) -> Vec<ChannelMessage> {
    std::fs::read_to_string(path)
        .map(|c| parse_channel_markdown(&c))
        .unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Output formatting
// ─────────────────────────────────────────────────────────────────────────────

// Terse single-line-per-message format (≤¼ human token count).
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
fn format_terse(msgs: &[ChannelMessage]) -> String {
    msgs.iter()
        .map(|m| {
            let to = if m.to.is_empty() {
                "@all".into()
            } else {
                m.to.join(",")
            };
            let re = m.re.map(|r| format!(" re:{}", r)).unwrap_or_default();
            let proj = m
                .project
                .as_deref()
                .map(|p| format!(" proj:{}", p))
                .unwrap_or_default();
            let body1 = m.body.lines().next().unwrap_or("").trim();
            format!(
                "msg-{} | {} -> {}{}{} | {} | {}\n",
                m.id, m.from, to, re, proj, m.timestamp, body1
            )
        })
        .collect()
}

// Listen-format: per-message single line optimised for Monitor consumption.
///
// Replaces body-first-line with the canonical fetch command so the consumer
// (mainthread inside Monitor) can retrieve the FULL body deterministically
// via `aw chat read --re <id> --full`. Avoids the silent truncation the
// terse body1 form caused when bodies span multiple lines or notifications
// have a width limit.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
fn format_listen(msgs: &[ChannelMessage]) -> String {
    msgs.iter()
        .map(|m| {
            let to = if m.to.is_empty() {
                "@all".into()
            } else {
                m.to.join(",")
            };
            let re = m.re.map(|r| format!(" re:{}", r)).unwrap_or_default();
            let proj = m
                .project
                .as_deref()
                .map(|p| format!(" proj:{}", p))
                .unwrap_or_default();
            format!(
                "msg-{} | {} -> {}{}{} | {} | $ aw chat read --re {} --full\n",
                m.id, m.from, to, re, proj, m.timestamp, m.id
            )
        })
        .collect()
}

// Human-readable Markdown format.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic
fn format_human(msgs: &[ChannelMessage]) -> String {
    msgs.iter()
        .flat_map(|m| {
            let mut lines = vec![
                format!("## msg-{}", m.id),
                format!("**From:** {}  ", m.from),
            ];
            if !m.to.is_empty() {
                lines.push(format!("**To:** {}  ", m.to.join(", ")));
            }
            if let Some(re) = m.re {
                lines.push(format!("**Re:** msg-{}  ", re));
            }
            if let Some(ref proj) = m.project {
                lines.push(format!("**Project:** {}  ", proj));
            }
            if !m.timestamp.is_empty() {
                lines.push(format!("**Time:** {}  ", m.timestamp));
            }
            lines.push(String::new());
            lines.push(m.body.clone());
            lines.push("\n---\n".into());
            lines
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render(msgs: &[ChannelMessage], fmt: OutputFormat) -> String {
    match fmt {
        OutputFormat::Terse => format_terse(msgs),
        OutputFormat::Human => format_human(msgs),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// post
// ─────────────────────────────────────────────────────────────────────────────

// Once that primitive exists, this function regenerates from
// projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#interaction (post sequence).

// Handle `aw chat post`.
///
// Reads body from `--body-file` or stdin, allocates the next msg-id, and
// appends one JSONL line via a single `O_APPEND` `write()`. POSIX guarantees
// atomicity for writes ≤ `PIPE_BUF` (4 KB on macOS / Linux), so concurrent
// `post` calls from parallel worktrees never interleave.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (run_post)
fn run_post(args: PostArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
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
    }
    .trim()
    .to_string();

    let channel_path = Path::new(CHANNEL_PATH);
    let existing = parse_channel(channel_path);
    let next_id = existing.iter().map(|m| m.id).max().unwrap_or(0) + 1;
    let timestamp = Utc::now().to_rfc3339();

    // --all sets to: [] (broadcast); --to provides the addressee list
    let to = if args.all { vec![] } else { args.to };

    let msg = ChannelMessage {
        id: next_id,
        from: identity.to_string(),
        to,
        re: args.re,
        project: args.project,
        timestamp,
        body,
    };

    let line = serialize_message_jsonl(&msg)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(CHANNEL_PATH)
        .with_context(|| format!("opening channel file {}", CHANNEL_PATH))?;
    f.write_all(line.as_bytes())
        .context("appending JSONL line to channel")?;

    match fmt {
        OutputFormat::Terse => print!("{}", format_terse(&[msg])),
        OutputFormat::Human => {
            println!("Posted msg-{}.", next_id);
            print!("{}", format_human(&[msg]));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// list
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat list`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic  Node: run_list
fn run_list(args: ListArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let mut msgs = parse_channel(Path::new(CHANNEL_PATH));
    if let Some(ref m) = args.mentions {
        let target = if m == "@me" {
            identity.to_string()
        } else {
            m.clone()
        };
        msgs.retain(|msg| msg.to.is_empty() || msg.to.iter().any(|t| t == &target));
    }
    if let Some(n) = args.last {
        let l = msgs.len();
        if l > n {
            msgs = msgs[l - n..].to_vec();
        }
    }
    print!("{}", render(&msgs, fmt));
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// read
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat read`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic  Node: run_read
fn run_read(args: ReadArgs) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let msgs = parse_channel(Path::new(CHANNEL_PATH));
    let re_id = args.re as i64;
    let mut thread: Vec<ChannelMessage> = msgs
        .iter()
        .filter(|m| m.id == re_id || m.re == Some(re_id))
        .cloned()
        .collect();
    if thread.is_empty() {
        eprintln!("No messages found for thread anchored at msg-{}.", args.re);
        return Ok(());
    }
    if !args.full {
        for m in &mut thread {
            m.body = m.body.lines().next().unwrap_or("").trim().to_string();
        }
    }
    print!("{}", render(&thread, fmt));
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// members
// ─────────────────────────────────────────────────────────────────────────────

// Handle `aw chat members`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: branch_cmd → members_read
fn run_members(args: MembersArgs, identity: &str) -> Result<()> {
    if args.register {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let branch = detect_git_branch(&cwd).unwrap_or_else(|| "unknown".to_string());
        let wt_path = detect_git_toplevel(&cwd).unwrap_or_else(|| cwd.display().to_string());
        let fmt = detect_output_format(args.terse, args.human);
        run_members_register(
            identity,
            &branch,
            &wt_path,
            &args.projects,
            &args.capabilities,
            Path::new(MEMBERS_PATH),
        )?;
        match fmt {
            OutputFormat::Terse => println!("registered member {} (branch: {})", identity, branch),
            OutputFormat::Human => println!(
                "Registered member `{}` (branch: {}) in {}.",
                identity, branch, MEMBERS_PATH
            ),
        }
        Ok(())
    } else {
        run_members_list(&args)
    }
}

// List all registered members from members.yaml.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md#logic  Node: members_list
fn run_members_list(args: &MembersArgs) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);
    let mf = read_members_file(Path::new(MEMBERS_PATH)).unwrap_or_default();
    if mf.members.is_empty() {
        println!("No members registered. Run `aw chat members --register` first.");
        return Ok(());
    }
    match fmt {
        OutputFormat::Terse => {
            for m in &mf.members {
                println!(
                    "member-{} | {} | branch:{} | projects:[{}] | caps:[{}] | seen:{}",
                    m.name,
                    m.wt_path,
                    m.branch,
                    m.projects.join(","),
                    m.capabilities.join(","),
                    m.last_seen
                );
            }
        }
        OutputFormat::Human => {
            for m in &mf.members {
                println!("## member-{}", m.name);
                println!("**Checkout:** {}  \n**Branch:** {}  ", m.wt_path, m.branch);
                if !m.projects.is_empty() {
                    println!("**Projects:** {}  ", m.projects.join(", "));
                }
                if !m.capabilities.is_empty() {
                    println!("**Capabilities:** {}  ", m.capabilities.join(", "));
                }
                println!("**Last seen:** {}  \n", m.last_seen);
            }
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// listen
// ─────────────────────────────────────────────────────────────────────────────

// Once that primitive exists, this region regenerates from
// projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (listen branch / tail_spawn node).

// RAII guard around the `tail -F` child process. On drop, kills the child
// and reaps it via `wait()` so the listen process never leaves zombies on
// SIGINT or parent-exit.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (cleanup_child)
struct TailGuard {
    child: Child,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/chat.md#source
impl Drop for TailGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// Handle `aw chat listen`.
///
// Spawns `tail -F -n +1 <channel>`, streams its stdout line-by-line, parses
// each line as `ChannelMessage`, runs `should_emit`, and prints + flushes
// matching messages. Flushes after each emit to satisfy the Monitor latency
// contract (<200 ms in pipe mode).
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic (run_listen)
fn run_listen(args: ListenArgs, identity: &str) -> Result<()> {
    let fmt = detect_output_format(args.terse, args.human);

    let self_name = match args.mentions.as_deref() {
        Some("@me") => identity.to_string(),
        Some(name) => name.to_string(),
        None => identity.to_string(),
    };

    let mut child = Command::new("tail")
        .args(["-F", "-n", "+1", CHANNEL_PATH])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning `tail -F {}`", CHANNEL_PATH))?;

    let stdout = child
        .stdout
        .take()
        .context("capturing stdout of tail process")?;
    let _guard = TailGuard { child };
    let reader = BufReader::new(stdout);

    let mut history: Vec<ChannelMessage> = Vec::new();
    let stdout_handle = io::stdout();
    let mut out = stdout_handle.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: ChannelMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("warning: skipping malformed channel line: {}", e);
                continue;
            }
        };
        if should_emit(&msg, &self_name, &history, args.all) {
            // Listen events feed Monitor; emit the listen-format one-liner that
            // ends with `$ aw chat read --re <id> --full`. The consumer
            // (mainthread in the Monitor harness) runs that command to fetch
            // the full body, instead of trying to parse a body inlined into
            // a single notification line. The --human / --terse user flags
            // still take effect for an interactive `aw chat listen` run
            // (no Monitor wrapper).
            let rendered = match fmt {
                OutputFormat::Terse | OutputFormat::Human if args.terse || args.human => {
                    render(std::slice::from_ref(&msg), fmt)
                }
                _ => format_listen(std::slice::from_ref(&msg)),
            };
            let _ = out.write_all(rendered.as_bytes());
            let _ = out.flush();
        }
        history.push(msg);
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 4-rule listen filter
// ─────────────────────────────────────────────────────────────────────────────

// Once that primitive exists, this region regenerates from
// projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic (should_emit flowchart).

// Determine whether `msg` should be emitted to the caller named `self_name`.
///
// Rules evaluated in order:
//   0. `--all` flag: emit unconditionally.
//   1. Direct cue: `msg.to` contains `self_name`.
//   2. Broadcast: `msg.to` is empty.
//   3. Echo: `msg.from == self_name`.
//   4. Dynamic thread membership: any msg in the same thread has `from == self_name`
//      or `self_name` in `to`.
///
// `all_msgs` is the full channel snapshot used for thread-root resolution and
// membership lookup. It must include msgs seen before the current poll window.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic
fn should_emit(
    msg: &ChannelMessage,
    self_name: &str,
    all_msgs: &[ChannelMessage],
    all_flag: bool,
) -> bool {
    // Rule 0: --all override
    if all_flag {
        return true;
    }
    // Rule 1: direct cue
    if msg.to.iter().any(|t| t == self_name) {
        return true;
    }
    // Rule 2: broadcast
    if msg.to.is_empty() {
        return true;
    }
    // Rule 3: echo
    if msg.from == self_name {
        return true;
    }
    // Rule 4: dynamic thread membership
    let root_id = thread_root_of(msg, all_msgs);
    all_msgs
        .iter()
        .filter(|m| thread_root_of(m, all_msgs) == root_id)
        .any(|m| m.from == self_name || m.to.iter().any(|t| t == self_name))
}

// Walk the `re:` chain of `msg` to find the root message id.
///
// Stops when a message has `re == None` (it is the root). The walk is bounded
// by `all_msgs.len()` iterations to guard against pathological cycles in the
// re-chain (e.g. A re: B, B re: A). If the parent is not found in the snapshot
// the current message id is treated as the root.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic
fn thread_root_of(msg: &ChannelMessage, all_msgs: &[ChannelMessage]) -> i64 {
    let mut current = msg.id;
    let mut current_re = msg.re;
    // Cycle guard: bound by total msg count.
    for _ in 0..all_msgs.len() {
        let Some(parent_id) = current_re else {
            return current;
        };
        let Some(parent) = all_msgs.iter().find(|m| m.id == parent_id) else {
            return current;
        };
        current = parent.id;
        current_re = parent.re;
    }
    current // Pathological cycle — return wherever we ended up
}

// `run_listen_loop`, `read_listen_state`, `write_listen_state`, and
// `listen_state_path` were removed in the JSONL migration: the new
// `run_listen` streams via `tail -F` and tracks history in-memory, so neither
// a polling loop nor a `~/.aw/chat-state.json` file is needed.

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — empty channel returns empty vec
    #[test]
    fn test_parse_channel_markdown_empty() {
        assert!(parse_channel_markdown("").is_empty());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — message id and fields parsed correctly (new YAML format)
    #[test]
    fn test_parse_channel_markdown_single_message() {
        let content = "---\nid: 1\nfrom: \"score\"\nto: [\"mamba\"]\nre: null\nproject: null\ntimestamp: \"2024-01-01T00:00:00Z\"\n---\nhello world\n";
        let msgs = parse_channel_markdown(content);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, 1);
        assert_eq!(msgs[0].from, "score");
        assert_eq!(msgs[0].to, vec!["mamba"]);
        assert!(msgs[0].body.contains("hello world"));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — multiple messages parsed correctly (new YAML format)
    #[test]
    fn test_parse_channel_markdown_multiple_messages() {
        let content = concat!(
            "---\nid: 1\nfrom: \"a\"\nto: []\nre: null\nproject: null\ntimestamp: \"t1\"\n---\nbody1\n",
            "---\nid: 2\nfrom: \"b\"\nto: []\nre: null\nproject: null\ntimestamp: \"t2\"\n---\nbody2\n"
        );
        let msgs = parse_channel_markdown(content);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 1);
        assert_eq!(msgs[1].id, 2);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — terse format shorter than human format
    #[test]
    fn test_format_terse_shorter_than_human() {
        let msgs = vec![ChannelMessage {
            id: 1,
            from: "score".into(),
            to: vec!["mamba".into()],
            re: None,
            project: None,
            timestamp: "2024-01-01T00:00:00Z".into(),
            body: "hello from score\nmore text\neven more text here for padding and context".into(),
        }];
        assert!(format_terse(&msgs).len() < format_human(&msgs).len());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — identity fallback when no .aw/config.toml
    #[test]
    fn test_detect_team_identity_fallback_to_basename() {
        let tmp = tempfile::tempdir().unwrap();
        let no_members = tmp.path().join("no-members.yaml");
        let identity = detect_team_identity(tmp.path(), &no_members).unwrap();
        assert!(!identity.is_empty());
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#schema — ListenState serde round-trip
    #[test]
    fn test_listen_state_serde_roundtrip() {
        let mut state: ListenState = HashMap::new();
        state.insert(
            "score".into(),
            AgentLastSeen {
                last_seen_msg_id: 5,
                last_polled_at: Some("2024-01-01T00:00:00Z".into()),
            },
        );
        let json = serde_json::to_string(&state).unwrap();
        let back: ListenState = serde_json::from_str(&json).unwrap();
        assert_eq!(back["score"].last_seen_msg_id, 5);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat.md#logic — detect_team_identity reads [team] name from config
    #[test]
    fn test_detect_team_identity_reads_config() {
        let tmp = tempfile::tempdir().unwrap();
        let score_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&score_dir).unwrap();
        std::fs::write(score_dir.join("config.toml"), "[team]\nname = \"myteam\"\n").unwrap();
        let no_members = tmp.path().join("no-members.yaml");
        let identity = detect_team_identity(tmp.path(), &no_members).unwrap();
        assert_eq!(identity, "myteam");
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — Rule 1: direct cue
    #[test]
    fn test_filter_direct_cue() {
        let msg = ChannelMessage {
            id: 1,
            from: "mamba".into(),
            to: vec!["score".into()],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "hello".into(),
        };
        assert!(should_emit(&msg, "score", &[], false));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — Rule 2: broadcast
    #[test]
    fn test_filter_broadcast() {
        let msg = ChannelMessage {
            id: 2,
            from: "mamba".into(),
            to: vec![],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "broadcast msg".into(),
        };
        assert!(should_emit(&msg, "score", &[], false));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — Rule 3: echo
    #[test]
    fn test_filter_echo() {
        let msg = ChannelMessage {
            id: 3,
            from: "score".into(),
            to: vec!["mamba".into()],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "my post".into(),
        };
        assert!(should_emit(&msg, "score", &[], false));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — Rule 4: dynamic thread membership pulls in later msgs
    #[test]
    fn test_filter_dynamic_thread_membership_pulled_in() {
        // msg-1: root from A to [B]
        // msg-2: from A to [B, score] re:1 (self cued mid-thread)
        // msg-3: from B to [A] re:1 (self NOT in to — should be pulled in by thread membership)
        let msg1 = ChannelMessage {
            id: 1,
            from: "A".into(),
            to: vec!["B".into()],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "root".into(),
        };
        let msg2 = ChannelMessage {
            id: 2,
            from: "A".into(),
            to: vec!["B".into(), "score".into()],
            re: Some(1),
            project: None,
            timestamp: "t".into(),
            body: "cue self".into(),
        };
        let msg3 = ChannelMessage {
            id: 3,
            from: "B".into(),
            to: vec!["A".into()],
            re: Some(1),
            project: None,
            timestamp: "t".into(),
            body: "reply without self".into(),
        };
        let all_msgs = vec![msg1, msg2, msg3.clone()];
        assert!(should_emit(&msg3, "score", &all_msgs, false));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — unrelated thread not emitted
    #[test]
    fn test_filter_unrelated_thread() {
        let msg10 = ChannelMessage {
            id: 10,
            from: "A".into(),
            to: vec!["B".into()],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "unrelated root".into(),
        };
        let msg11 = ChannelMessage {
            id: 11,
            from: "B".into(),
            to: vec!["A".into()],
            re: Some(10),
            project: None,
            timestamp: "t".into(),
            body: "unrelated reply".into(),
        };
        let all_msgs = vec![msg10, msg11.clone()];
        assert!(!should_emit(&msg11, "score", &all_msgs, false));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — Rule 0: --all flag emits everything
    #[test]
    fn test_filter_all_flag_emits_everything() {
        let msg = ChannelMessage {
            id: 20,
            from: "A".into(),
            to: vec!["B".into()],
            re: None,
            project: None,
            timestamp: "t".into(),
            body: "unrelated msg".into(),
        };
        // With all_flag=true, should emit regardless of rules
        assert!(should_emit(&msg, "score", &[], true));
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md#logic — thread_root_of cycle guard doesn't infinite-loop
    #[test]
    fn test_thread_root_cycle_guard() {
        // Pathological cycle: msg-1 re:2, msg-2 re:1
        let msg1 = ChannelMessage {
            id: 1,
            from: "A".into(),
            to: vec![],
            re: Some(2),
            project: None,
            timestamp: "t".into(),
            body: "".into(),
        };
        let msg2 = ChannelMessage {
            id: 2,
            from: "B".into(),
            to: vec![],
            re: Some(1),
            project: None,
            timestamp: "t".into(),
            body: "".into(),
        };
        let all_msgs = vec![msg1.clone(), msg2];
        // Should not infinite-loop; returns some id
        let root = thread_root_of(&msg1, &all_msgs);
        assert!(root == 1 || root == 2); // terminated by cycle guard
    }

    // ─────────────────────────────────────────────────────────────────────────
    // G3 contract tests (T1–T6)
    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: wrap ChatArgs in a Parser so try_parse_from works in tests.
    #[derive(Debug, clap::Parser)]
    struct TestChatCli {
        #[command(subcommand)]
        command: ChatCommand,
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T1: post without --to or --all fails at clap
    #[test]
    fn test_post_without_to_or_all_fails_clap() {
        use clap::Parser;
        // Simulate: score post --body-file -   (no --to, no --all)
        let result = TestChatCli::try_parse_from(["score", "post", "--body-file", "-"]);
        assert!(
            result.is_err(),
            "Expected clap error when --to and --all are both absent"
        );
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T2: post --all parses; all=true, to=[]
    #[test]
    fn test_post_with_all_writes_empty_to() {
        use clap::Parser;
        let args = TestChatCli::try_parse_from(["score", "post", "--all", "--body-file", "-"])
            .expect("--all should parse");
        match args.command {
            ChatCommand::Post(pa) => {
                assert!(pa.all);
                assert!(pa.to.is_empty(), "to should be empty when --all is set");
            }
            _ => panic!("expected Post command"),
        }
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T3: post --to a,b parses to Vec["a","b"]
    #[test]
    fn test_post_with_to_csv_writes_vec() {
        use clap::Parser;
        let args =
            TestChatCli::try_parse_from(["score", "post", "--to", "a,b", "--body-file", "-"])
                .expect("--to a,b should parse");
        match args.command {
            ChatCommand::Post(pa) => {
                assert!(!pa.all);
                assert_eq!(pa.to, vec!["a", "b"]);
            }
            _ => panic!("expected Post command"),
        }
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T4: post --from flag is rejected by clap (removed)
    #[test]
    fn test_post_with_from_flag_rejected_at_clap() {
        use clap::Parser;
        // --from does not exist in PostArgsV3; clap should error with unrecognized argument
        let result = TestChatCli::try_parse_from([
            "score",
            "post",
            "--from",
            "anyone",
            "--to",
            "b",
            "--body-file",
            "-",
        ]);
        assert!(
            result.is_err(),
            "Expected clap error for unknown --from flag"
        );
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T5: listen --once flag is rejected by clap (removed)
    #[test]
    fn test_listen_with_once_flag_rejected_at_clap() {
        use clap::Parser;
        // --once does not exist in ListenArgsV4; clap should error
        let result = TestChatCli::try_parse_from(["score", "listen", "--once"]);
        assert!(
            result.is_err(),
            "Expected clap error for unknown --once flag"
        );
    }

    // T10: REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#tests — listen --interval flag is rejected by clap (removed in JSONL migration)
    #[test]
    fn test_listen_with_interval_flag_rejected_at_clap() {
        use clap::Parser;
        let result = TestChatCli::try_parse_from(["score", "listen", "--interval", "30"]);
        assert!(
            result.is_err(),
            "Expected clap error for removed --interval flag"
        );
    }

    // T7: REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#tests — ChannelMessage round-trips via serde_json
    // (body is no longer #[serde(skip)])
    #[test]
    fn test_channel_message_jsonl_serde_roundtrip() {
        let original = ChannelMessage {
            id: 7,
            from: "score".to_string(),
            to: vec!["mamba".to_string()],
            re: Some(3),
            project: Some("conductor".to_string()),
            timestamp: "2026-04-27T00:00:00Z".to_string(),
            body: "body content with newlines\nand tabs\there".to_string(),
        };
        let line = serde_json::to_string(&original).unwrap();
        assert!(!line.contains('\n'), "JSONL line must not contain newlines");
        let parsed: ChannelMessage = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.from, original.from);
        assert_eq!(parsed.to, original.to);
        assert_eq!(parsed.re, original.re);
        assert_eq!(parsed.project, original.project);
        assert_eq!(parsed.body, original.body);
    }

    // T8: REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#tests — JSONL line size guarantees POSIX atomic O_APPEND
    #[test]
    fn test_jsonl_line_size_under_pipe_buf() {
        // POSIX guarantees atomicity for writes ≤ PIPE_BUF.
        // Linux/macOS PIPE_BUF >= 512; kernels typically ship 4096.
        let msg = ChannelMessage {
            id: 1,
            from: "score".to_string(),
            to: vec!["mamba".to_string()],
            re: None,
            project: None,
            timestamp: "2026-04-27T00:00:00Z".to_string(),
            body: "a representative chat body with some prose to approximate typical traffic size"
                .to_string(),
        };
        let line = serialize_message_jsonl(&msg).unwrap();
        assert!(
            line.len() < 4096,
            "JSONL line exceeds 4 KB atomic-append window: {} bytes",
            line.len()
        );
    }

    // REQ projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md#logic — parse_channel_jsonl skips malformed lines
    #[test]
    fn test_parse_channel_jsonl_skips_malformed_lines() {
        use chat_members::parse_channel_jsonl;
        let m1 = ChannelMessage {
            id: 1,
            from: "score".to_string(),
            to: vec!["mamba".to_string()],
            re: None,
            project: None,
            timestamp: "t1".to_string(),
            body: "ok".to_string(),
        };
        let m2 = ChannelMessage {
            id: 2,
            from: "score".to_string(),
            to: vec![],
            re: None,
            project: None,
            timestamp: "t2".to_string(),
            body: "ok2".to_string(),
        };
        let mut content = String::new();
        content.push_str(&serde_json::to_string(&m1).unwrap());
        content.push('\n');
        content.push_str("not valid json {{}\n");
        content.push('\n'); // blank line
        content.push_str(&serde_json::to_string(&m2).unwrap());
        content.push('\n');
        let parsed = parse_channel_jsonl(&content);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].id, 1);
        assert_eq!(parsed[1].id, 2);
    }

    // REQ: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md#tests — T6: resolve_identity returns branch
    // (per-worktree identity) and does not fall through to a sibling WT's config.
    #[test]
    fn test_resolve_identity_uses_branch_not_sibling_config() {
        use chat_members::resolve_identity;

        // Create a temp dir as a fake WT with its own git repo and .aw/config.toml
        let tmp = tempfile::tempdir().unwrap();
        let tmp_path = tmp.path();

        // Init git repo so git -C <cwd> rev-parse --show-toplevel succeeds
        std::process::Command::new("git")
            .args(["init", "-b", "fake-branch"])
            .current_dir(tmp_path)
            .output()
            .expect("git init should succeed");

        // Set a fake git user to avoid config errors
        std::process::Command::new("git")
            .args([
                "-C",
                tmp_path.to_str().unwrap(),
                "config",
                "user.email",
                "fake@test",
            ])
            .output()
            .ok();
        std::process::Command::new("git")
            .args([
                "-C",
                tmp_path.to_str().unwrap(),
                "config",
                "user.name",
                "fake",
            ])
            .output()
            .ok();

        // Write .aw/config.toml with [team] name = "fake-team-name"
        // — under the new identity model, branch beats config.toml so this should
        // NOT win. (It used to win under the old order; that broke per-worktree
        // distinguishability when multiple WTs of the same repo all returned the
        // same team name.)
        let score_dir = tmp_path.join(".aw");
        std::fs::create_dir_all(&score_dir).unwrap();
        std::fs::write(
            score_dir.join("config.toml"),
            "[team]\nname = \"fake-team-name\"\n",
        )
        .unwrap();

        // Ensure /tmp/aw-channel-members.yaml does not have the fake branch
        // (it may exist from other tests, so we rely on branch name being unique)
        let identity =
            resolve_identity(tmp_path).expect("resolve_identity should succeed for fake WT");

        // Branch wins (Step 2 in new resolution chain) — per-worktree identity.
        assert_eq!(
            identity, "fake-branch",
            "resolve_identity should return the git branch (per-worktree identity), \
             not the shared config.toml team name; got: {}",
            identity
        );

        // Defence in depth: must not be main WT's name (no sibling fall-through),
        // must not be the in-tree config.toml team name (config doesn't win when
        // a real branch exists).
        assert_ne!(
            identity, "score",
            "must not fall through to main's config.toml"
        );
        assert_ne!(
            identity, "fake-team-name",
            "config.toml must not beat branch"
        );
    }
}


```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/chat.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
