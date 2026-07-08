---
id: projects-score-src-chat-mod-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized apps/agentic-workflow/src/cli/chat/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/chat/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AGENTS_PATH` | apps/agentic-workflow/src/cli/chat/mod.rs | constant | pub | 32 |  |
| `AgentLastSeen` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 206 |  |
| `AgentRegistration` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 191 |  |
| `AgentsArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 131 |  |
| `CHANNEL_PATH` | apps/agentic-workflow/src/cli/chat/mod.rs | constant | pub | 31 |  |
| `ChannelMessage` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 175 |  |
| `ChatArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 42 |  |
| `ChatCommand` | apps/agentic-workflow/src/cli/chat/mod.rs | enum | pub | 51 |  |
| `ListArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 90 |  |
| `ListenArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 150 |  |
| `ListenState` | apps/agentic-workflow/src/cli/chat/mod.rs | type | pub | 214 |  |
| `PostArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 68 |  |
| `ReadArgs` | apps/agentic-workflow/src/cli/chat/mod.rs | struct | pub | 112 |  |
| `run_chat` | apps/agentic-workflow/src/cli/chat/mod.rs | function | pub | 224 | run_chat(args: ChatArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/chat/mod.rs -->
```rust
//! `aw chat` — cross-worktree agent messaging via a shared plain-text channel.
//!
//! All communication flows through `/tmp/aw/chat/channel.md` (append-only, ephemeral).
//! Five subcommands: `post`, `list`, `read`, `agents`, `listen`.
//!
//! Agent identity is resolved by walking up from CWD for `.aw/config.toml` and
//! reading `[team] name`. Falls back to git-toplevel basename when no `[team]` block
//! is found.
//!
//! Output format: TTY → human markdown; pipe → terse (token-efficient, ≤¼ human).
//! Both can be overridden per-invocation with `--terse` / `--human`.
//!
//! /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#overview

mod helpers;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) use helpers::{OutputFormat, detect_output_format};

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) const CHANNEL_PATH: &str = "/tmp/aw/chat/channel.md";
pub(crate) const AGENTS_PATH: &str = "/tmp/aw/chat/agents.md";

// ─────────────────────────────────────────────────────────────────────────────
// Schema
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level args wrapper for `aw chat`. Mirrors `IssuesArgs` pattern.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ChatArgs {
    #[command(subcommand)]
    pub command: ChatCommand,
}

/// Available subcommands for `aw chat`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Subcommand)]
pub enum ChatCommand {
    /// Post a new message to the shared channel.
    Post(PostArgs),
    /// List messages from the shared channel.
    List(ListArgs),
    /// Read a message thread by anchor msg-id.
    Read(ReadArgs),
    /// Register or list agent capabilities.
    Agents(AgentsArgs),
    /// Poll for new messages addressed to the caller team.
    Listen(ListenArgs),
}

/// Args for `aw chat post`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct PostArgs {
    /// Addressee team names (comma-separated). Empty means broadcast.
    #[arg(long, value_delimiter = ',')]
    pub to: Vec<String>,
    /// Anchor msg-id to reply to.
    #[arg(long)]
    pub re: Option<u64>,
    /// Path to body file. Use `-` for stdin.
    #[arg(long, default_value = "-")]
    pub body_file: String,
    /// Force terse (token-efficient) output regardless of TTY.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human (markdown) output regardless of TTY.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

/// Args for `aw chat list`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter to messages whose `to:` includes this team. `@me` resolves to caller.
    #[arg(long)]
    pub mentions: Option<String>,
    /// Limit to the N most recent messages.
    #[arg(long)]
    pub last: Option<usize>,
    /// Message status filter (`open` or `all`). Default is `open`.
    #[arg(long, default_value = "open")]
    pub status: String,
    /// Force terse output.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human output.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

/// Args for `aw chat read`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ReadArgs {
    /// Anchor msg-id. Returns anchor + all replies in thread order.
    #[arg(long)]
    pub re: u64,
    /// Include full body. Default shows first-line summary only.
    #[arg(long)]
    pub full: bool,
    /// Force terse output.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human output.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

/// Args for `aw chat agents`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct AgentsArgs {
    /// Write or replace caller's AgentRegistration in `/tmp/aw/chat/agents.md`.
    #[arg(long)]
    pub register: bool,
    /// Print all registered agents from `/tmp/aw/chat/agents.md`.
    #[arg(long)]
    pub list: bool,
    /// Force terse output.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human output.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

/// Args for `aw chat listen`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Args)]
pub struct ListenArgs {
    /// Single poll, then exit 0. Cron-friendly.
    #[arg(long)]
    pub once: bool,
    /// Poll interval in seconds. Default 60.
    #[arg(long, default_value = "60")]
    pub interval: u64,
    /// Filter printed messages to `@me` (caller team) or `@all` (broadcast).
    #[arg(long)]
    pub mentions: Option<String>,
    /// Force terse output.
    #[arg(long, conflicts_with = "human")]
    pub terse: bool,
    /// Force human output.
    #[arg(long, conflicts_with = "terse")]
    pub human: bool,
}

/// One message block inside `/tmp/aw/chat/channel.md`.
///
/// Serialised as a `## msg-NNN` Markdown heading followed by a YAML
/// frontmatter block (`---`) then the body text.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: u64,
    pub from: String,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re: Option<u64>,
    pub timestamp: String,
    #[serde(skip)]
    pub body: String,
}

/// One entry in `/tmp/aw/chat/agents.md`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    pub wt_path: String,
    pub branch: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub last_seen: String,
}

/// Per-team entry inside `~/.aw/chat-state.json`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentLastSeen {
    pub last_seen_msg_id: u64,
    pub last_polled_at: Option<String>,
}

/// `~/.aw/chat-state.json` — keyed by team name.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#schema
pub type ListenState = HashMap<String, AgentLastSeen>;

// ─────────────────────────────────────────────────────────────────────────────
// Top-level dispatcher
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level dispatcher for `aw chat`.
///
/// /// @spec apps/agentic-workflow/tech-design/surface/specs/score-chat.md#logic
/// Node: start → detect_identity → detect_format → branch_cmd
pub fn run_chat(args: ChatArgs) -> Result<()> {
    // @spec-node detect_identity
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let identity = helpers::detect_team_identity(&cwd)?;

    match args.command {
        // @spec-node branch_cmd → run_post
        ChatCommand::Post(a) => helpers::run_post(a, &identity),
        // @spec-node branch_cmd → run_list
        ChatCommand::List(a) => helpers::run_list(a, &identity),
        // @spec-node branch_cmd → run_read
        ChatCommand::Read(a) => helpers::run_read(a),
        // @spec-node branch_cmd → branch_agents
        ChatCommand::Agents(a) => helpers::run_agents(a, &identity),
        // @spec-node branch_cmd → branch_listen
        ChatCommand::Listen(a) => helpers::run_listen(a, &identity),
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/chat/mod.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
