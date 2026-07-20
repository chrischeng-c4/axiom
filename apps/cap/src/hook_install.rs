// SPEC-MANAGED: apps/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Hook registration backend for `cap on` / `cap off` — wires the cap
//! PreToolUse hook into Claude Code, Codex CLI, or AGY.
//!
//! - Claude Code: merges `hooks.PreToolUse[]` into
//!   `~/.claude/settings.json`.
//! - Codex CLI:   merges `[[hooks.PreToolUse]]` into `~/.codex/config.toml`.
//! - AGY: merges a named `PreToolUse` entry into
//!   `~/.gemini/config/hooks.json` (or the legacy global path when present).
//!
//! Hooks are intentionally global: cap protects the machine, not one checkout.
//!
//! Idempotent: if a PreToolUse entry already points at our cap
//! binary it's left in place. Existing unrelated hooks are
//! preserved.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value as JsonValue};

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
    Agy,
}

impl Agent {
    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Agy => "agy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookState {
    Enabled,
    Disabled,
}

impl HookState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
pub fn enable(agent: Agent, print: bool) -> Result<()> {
    // Use the absolute path so the hook fires correctly even when
    // the parent process's PATH doesn't include cap's install dir.
    let cap_path = public_cap_exe().context("locating cap binary")?;

    match agent {
        Agent::Claude => install_claude(&cap_path, print),
        Agent::Codex => install_codex(&cap_path, print),
        Agent::Agy => install_agy(&cap_path, print),
    }
}

/// Backward-compatible Rust API for callers that used the former installer.
pub fn run(agent: Agent, print: bool) -> Result<()> {
    enable(agent, print)
}

pub fn disable(agent: Agent, print: bool) -> Result<()> {
    match agent {
        Agent::Claude => disable_claude(print),
        Agent::Codex => disable_codex(print),
        Agent::Agy => disable_agy(print),
    }
}

pub fn status(agent: Agent) -> Result<HookState> {
    match agent {
        Agent::Claude => claude_hook_status(),
        Agent::Codex => codex_hook_status(),
        Agent::Agy => agy_hook_status(),
    }
}

fn public_cap_exe() -> Result<String> {
    if let Ok(path) = std::env::var("CAP_PUBLIC_EXE") {
        if !path.trim().is_empty() {
            return Ok(path);
        }
    }
    let cap = std::env::current_exe().context("locating cap binary")?;
    Ok(cap.to_string_lossy().to_string())
}

// ---------------------------------------------------------------- Claude

fn install_claude(cap_path: &str, print: bool) -> Result<()> {
    let hook_cmd = format!("{cap_path} hook bash --claude-code");
    let snippet = json!({
        "hooks": {
            "PreToolUse": [{
                "matcher": "Bash",
                "hooks": [{ "type": "command", "command": hook_cmd }]
            }]
        }
    });

    if print {
        println!("{}", serde_json::to_string_pretty(&snippet)?);
        return Ok(());
    }

    let path = claude_settings_path()?;
    let merged_status = merge_claude(&path, &hook_cmd)?;
    println!("{}: {}", merged_status.describe(), path.display());
    Ok(())
}

fn claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
    Ok(home.join(".claude").join("settings.json"))
}

fn disable_claude(print: bool) -> Result<()> {
    let path = claude_settings_path()?;
    if print {
        println!("would remove cap hook from: {}", path.display());
        return Ok(());
    }
    if !path.exists() {
        println!("cap hook already absent from: {}", path.display());
        return Ok(());
    }

    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut root: JsonValue = if text.trim().is_empty() {
        JsonValue::Object(Default::default())
    } else {
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
    };
    let removed = remove_claude_cap_hooks(&mut root)?;
    if removed {
        std::fs::write(&path, serde_json::to_string_pretty(&root)? + "\n")
            .with_context(|| format!("writing {}", path.display()))?;
        println!("removed cap hook from: {}", path.display());
    } else {
        println!("cap hook already absent from: {}", path.display());
    }
    Ok(())
}

fn claude_hook_status() -> Result<HookState> {
    let path = claude_settings_path()?;
    claude_hook_status_at(&path)
}

fn claude_hook_status_at(path: &Path) -> Result<HookState> {
    if !path.exists() {
        return Ok(HookState::Disabled);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(HookState::Disabled);
    }
    let root: JsonValue =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    let entries = root
        .get("hooks")
        .and_then(|hooks| hooks.get("PreToolUse"))
        .and_then(JsonValue::as_array);
    Ok(
        if entries.is_some_and(|entries| pretool_has_cap_hook(entries)) {
            HookState::Enabled
        } else {
            HookState::Disabled
        },
    )
}

fn merge_claude(path: &Path, hook_cmd: &str) -> Result<MergeStatus> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }

    let mut root: JsonValue = if path.exists() {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        if text.trim().is_empty() {
            JsonValue::Object(Default::default())
        } else {
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
        }
    } else {
        JsonValue::Object(Default::default())
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("{}: root is not a JSON object", path.display()))?;

    let hooks = root_obj
        .entry("hooks".to_string())
        .or_insert_with(|| JsonValue::Object(Default::default()))
        .as_object_mut()
        .ok_or_else(|| anyhow!("{}: hooks is not an object", path.display()))?;

    let pretool = hooks
        .entry("PreToolUse".to_string())
        .or_insert_with(|| JsonValue::Array(vec![]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("{}: PreToolUse is not an array", path.display()))?;

    if pretool_has_cap_hook(pretool) {
        return Ok(MergeStatus::AlreadyPresent);
    }

    pretool.push(json!({
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": hook_cmd }]
    }));

    let serialized = serde_json::to_string_pretty(&root)?;
    std::fs::write(path, serialized + "\n")
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(MergeStatus::Installed)
}

fn pretool_has_cap_hook(entries: &[JsonValue]) -> bool {
    entries.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(is_cap_hook_command)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    })
}

fn remove_claude_cap_hooks(root: &mut JsonValue) -> Result<bool> {
    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("Claude settings root is not a JSON object"))?;
    let Some(hooks) = root_obj.get_mut("hooks") else {
        return Ok(false);
    };
    let hooks = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow!("Claude settings hooks is not an object"))?;
    let Some(pretool) = hooks.get_mut("PreToolUse") else {
        return Ok(false);
    };
    let pretool = pretool
        .as_array_mut()
        .ok_or_else(|| anyhow!("Claude settings PreToolUse is not an array"))?;

    let mut removed = false;
    let entries = std::mem::take(pretool);
    for mut entry in entries {
        let Some(entry_obj) = entry.as_object_mut() else {
            pretool.push(entry);
            continue;
        };
        let Some(commands) = entry_obj.get_mut("hooks").and_then(JsonValue::as_array_mut) else {
            pretool.push(entry);
            continue;
        };
        let before = commands.len();
        commands.retain(|hook| {
            !hook
                .get("command")
                .and_then(JsonValue::as_str)
                .is_some_and(is_cap_hook_command)
        });
        removed |= commands.len() != before;
        if !commands.is_empty() {
            pretool.push(entry);
        }
    }
    if pretool.is_empty() {
        hooks.remove("PreToolUse");
    }
    if hooks.is_empty() {
        root_obj.remove("hooks");
    }
    Ok(removed)
}

fn is_cap_hook_command(s: &str) -> bool {
    is_cap_hook_command_kind(s, "bash")
}

fn is_cap_hook_command_kind(s: &str, adapter: &str) -> bool {
    // Any command line that contains `hook <adapter>` (with optional flags)
    // and whose first token's basename is `cap` counts as ours.
    let mut tokens = s.split_whitespace();
    let prog = match tokens.next() {
        Some(p) => p,
        None => return false,
    };
    let base = prog.rsplit('/').next().unwrap_or(prog);
    if base != "cap" {
        return false;
    }
    let rest: Vec<&str> = tokens.collect();
    rest.windows(2).any(|w| w[0] == "hook" && w[1] == adapter)
}

// ---------------------------------------------------------------- Codex

fn install_codex(cap_path: &str, print: bool) -> Result<()> {
    let hook_cmd = format!("{cap_path} hook bash --codex");
    let snippet = format!(
        "[[hooks.PreToolUse]]\n\
         matcher = \"^Bash$\"\n\n\
         [[hooks.PreToolUse.hooks]]\n\
         type = \"command\"\n\
         command = \"{}\"\n\
         timeout = 10\n",
        hook_cmd.replace('\\', "\\\\").replace('"', "\\\"")
    );

    if print {
        println!("{snippet}");
        return Ok(());
    }

    let path = codex_config_path()?;
    let merged_status = merge_codex(&path, &hook_cmd)?;
    println!("{}: {}", merged_status.describe(), path.display());
    Ok(())
}

fn codex_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
    Ok(home.join(".codex").join("config.toml"))
}

fn disable_codex(print: bool) -> Result<()> {
    let path = codex_config_path()?;
    if print {
        println!("would remove cap hook from: {}", path.display());
        return Ok(());
    }
    if !path.exists() {
        println!("cap hook already absent from: {}", path.display());
        return Ok(());
    }

    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut root: toml::Value = if text.trim().is_empty() {
        toml::Value::Table(Default::default())
    } else {
        text.parse::<toml::Value>()
            .with_context(|| format!("parsing {}", path.display()))?
    };
    let removed = remove_codex_cap_hooks(&mut root)?;
    if removed {
        std::fs::write(&path, toml::to_string_pretty(&root)?)
            .with_context(|| format!("writing {}", path.display()))?;
        println!("removed cap hook from: {}", path.display());
    } else {
        println!("cap hook already absent from: {}", path.display());
    }
    Ok(())
}

fn codex_hook_status() -> Result<HookState> {
    let path = codex_config_path()?;
    codex_hook_status_at(&path)
}

fn codex_hook_status_at(path: &Path) -> Result<HookState> {
    if !path.exists() {
        return Ok(HookState::Disabled);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(HookState::Disabled);
    }
    let root = text
        .parse::<toml::Value>()
        .with_context(|| format!("parsing {}", path.display()))?;
    let entries = root
        .get("hooks")
        .and_then(|hooks| hooks.get("PreToolUse"))
        .and_then(toml::Value::as_array);
    Ok(
        if entries.is_some_and(|entries| codex_pretool_has_cap_hook(entries)) {
            HookState::Enabled
        } else {
            HookState::Disabled
        },
    )
}

fn merge_codex(path: &Path, hook_cmd: &str) -> Result<MergeStatus> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }

    let mut root: toml::Value = if path.exists() {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        if text.trim().is_empty() {
            toml::Value::Table(Default::default())
        } else {
            text.parse::<toml::Value>()
                .with_context(|| format!("parsing {}", path.display()))?
        }
    } else {
        toml::Value::Table(Default::default())
    };

    let root_tbl = root
        .as_table_mut()
        .ok_or_else(|| anyhow!("{}: root is not a TOML table", path.display()))?;

    let hooks_v = root_tbl
        .entry("hooks".to_string())
        .or_insert_with(|| toml::Value::Table(Default::default()));
    let hooks_tbl = hooks_v
        .as_table_mut()
        .ok_or_else(|| anyhow!("{}: hooks is not a table", path.display()))?;

    let pretool_v = hooks_tbl
        .entry("PreToolUse".to_string())
        .or_insert_with(|| toml::Value::Array(vec![]));
    let pretool = pretool_v
        .as_array_mut()
        .ok_or_else(|| anyhow!("{}: hooks.PreToolUse is not an array", path.display()))?;

    if codex_pretool_has_cap_hook(pretool) {
        return Ok(MergeStatus::AlreadyPresent);
    }

    let mut inner_hook = toml::value::Table::new();
    inner_hook.insert("type".into(), toml::Value::String("command".into()));
    inner_hook.insert("command".into(), toml::Value::String(hook_cmd.to_string()));
    inner_hook.insert("timeout".into(), toml::Value::Integer(10));

    let mut entry = toml::value::Table::new();
    entry.insert("matcher".into(), toml::Value::String("^Bash$".into()));
    entry.insert(
        "hooks".into(),
        toml::Value::Array(vec![toml::Value::Table(inner_hook)]),
    );
    pretool.push(toml::Value::Table(entry));

    let serialized = toml::to_string_pretty(&root)?;
    std::fs::write(path, serialized).with_context(|| format!("writing {}", path.display()))?;
    Ok(MergeStatus::Installed)
}

fn codex_pretool_has_cap_hook(entries: &[toml::Value]) -> bool {
    entries.iter().any(|entry| {
        let Some(tbl) = entry.as_table() else {
            return false;
        };
        let Some(arr) = tbl.get("hooks").and_then(|h| h.as_array()) else {
            return false;
        };
        arr.iter().any(|h| {
            h.as_table()
                .and_then(|t| t.get("command"))
                .and_then(|c| c.as_str())
                .map(is_cap_hook_command)
                .unwrap_or(false)
        })
    })
}

fn remove_codex_cap_hooks(root: &mut toml::Value) -> Result<bool> {
    let root_tbl = root
        .as_table_mut()
        .ok_or_else(|| anyhow!("Codex config root is not a TOML table"))?;
    let Some(hooks) = root_tbl.get_mut("hooks") else {
        return Ok(false);
    };
    let hooks = hooks
        .as_table_mut()
        .ok_or_else(|| anyhow!("Codex config hooks is not a table"))?;
    let Some(pretool) = hooks.get_mut("PreToolUse") else {
        return Ok(false);
    };
    let pretool = pretool
        .as_array_mut()
        .ok_or_else(|| anyhow!("Codex config PreToolUse is not an array"))?;

    let mut removed = false;
    let entries = std::mem::take(pretool);
    for mut entry in entries {
        let Some(entry_tbl) = entry.as_table_mut() else {
            pretool.push(entry);
            continue;
        };
        let Some(commands) = entry_tbl
            .get_mut("hooks")
            .and_then(toml::Value::as_array_mut)
        else {
            pretool.push(entry);
            continue;
        };
        let before = commands.len();
        commands.retain(|hook| {
            !hook
                .as_table()
                .and_then(|table| table.get("command"))
                .and_then(toml::Value::as_str)
                .is_some_and(is_cap_hook_command)
        });
        removed |= commands.len() != before;
        if !commands.is_empty() {
            pretool.push(entry);
        }
    }
    if pretool.is_empty() {
        hooks.remove("PreToolUse");
    }
    if hooks.is_empty() {
        root_tbl.remove("hooks");
    }
    Ok(removed)
}

// ---------------------------------------------------------------- AGY

/// AGY's current global hook path. Older AGY installations used the legacy
/// antigravity-cli directory; when it already exists we preserve that choice
/// instead of silently creating a second active config.
fn agy_hooks_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
    let current = home.join(".gemini").join("config").join("hooks.json");
    let legacy = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("hooks.json");
    if current.exists() || !legacy.exists() {
        Ok(current)
    } else {
        Ok(legacy)
    }
}

fn agy_hook_paths() -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
    Ok(vec![
        home.join(".gemini").join("config").join("hooks.json"),
        home.join(".gemini")
            .join("antigravity-cli")
            .join("hooks.json"),
    ])
}

fn install_agy(cap_path: &str, print: bool) -> Result<()> {
    let hook_cmd = format!("{cap_path} hook agy");
    let snippet = json!({
        "cap-agent-guard": {
            "PreToolUse": [{
                "matcher": "run_command",
                "hooks": [{ "type": "command", "command": hook_cmd, "timeout": 10 }]
            }]
        }
    });
    if print {
        println!("{}", serde_json::to_string_pretty(&snippet)?);
        return Ok(());
    }

    let path = agy_hooks_path()?;
    let merged_status = merge_agy(&path, &hook_cmd)?;
    println!("{}: {}", merged_status.describe(), path.display());
    Ok(())
}

fn disable_agy(print: bool) -> Result<()> {
    for path in agy_hook_paths()? {
        if print {
            println!("would remove cap hook from: {}", path.display());
            continue;
        }
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let mut root: JsonValue = if text.trim().is_empty() {
            JsonValue::Object(Default::default())
        } else {
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
        };
        if remove_agy_cap_hooks(&mut root)? {
            std::fs::write(&path, serde_json::to_string_pretty(&root)? + "\n")
                .with_context(|| format!("writing {}", path.display()))?;
            println!("removed cap hook from: {}", path.display());
        }
    }
    if !print {
        println!("cap AGY hook disabled");
    }
    Ok(())
}

fn agy_hook_status() -> Result<HookState> {
    for path in agy_hook_paths()? {
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        if text.trim().is_empty() {
            continue;
        }
        let root: JsonValue =
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        if agy_has_cap_hook(&root) {
            return Ok(HookState::Enabled);
        }
    }
    Ok(HookState::Disabled)
}

fn merge_agy(path: &Path, hook_cmd: &str) -> Result<MergeStatus> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut root: JsonValue = if path.exists() {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        if text.trim().is_empty() {
            JsonValue::Object(Default::default())
        } else {
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
        }
    } else {
        JsonValue::Object(Default::default())
    };
    if agy_has_cap_hook(&root) {
        return Ok(MergeStatus::AlreadyPresent);
    }
    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("{}: root is not a JSON object", path.display()))?;
    let cap_config = root_obj
        .entry("cap-agent-guard".to_string())
        .or_insert_with(|| JsonValue::Object(Default::default()))
        .as_object_mut()
        .ok_or_else(|| anyhow!("{}: cap-agent-guard is not an object", path.display()))?;
    let pretool = cap_config
        .entry("PreToolUse".to_string())
        .or_insert_with(|| JsonValue::Array(vec![]))
        .as_array_mut()
        .ok_or_else(|| {
            anyhow!(
                "{}: cap-agent-guard.PreToolUse is not an array",
                path.display()
            )
        })?;
    pretool.push(json!({
        "matcher": "run_command",
        "hooks": [{ "type": "command", "command": hook_cmd, "timeout": 10 }]
    }));
    std::fs::write(path, serde_json::to_string_pretty(&root)? + "\n")
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(MergeStatus::Installed)
}

fn agy_has_cap_hook(root: &JsonValue) -> bool {
    root.as_object().is_some_and(|configs| {
        configs.values().any(|config| {
            config
                .get("PreToolUse")
                .and_then(JsonValue::as_array)
                .is_some_and(|entries| pretool_has_agy_cap_hook(entries))
        })
    })
}

fn pretool_has_agy_cap_hook(entries: &[JsonValue]) -> bool {
    entries.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(JsonValue::as_array)
            .is_some_and(|hooks| {
                hooks.iter().any(|hook| {
                    hook.get("command")
                        .and_then(JsonValue::as_str)
                        .is_some_and(is_cap_agy_hook_command)
                })
            })
    })
}

fn remove_agy_cap_hooks(root: &mut JsonValue) -> Result<bool> {
    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("AGY hooks root is not a JSON object"))?;
    let keys: Vec<String> = root_obj.keys().cloned().collect();
    let mut removed = false;
    for key in keys {
        let Some(config) = root_obj.get_mut(&key).and_then(JsonValue::as_object_mut) else {
            continue;
        };
        let Some(pretool) = config
            .get_mut("PreToolUse")
            .and_then(JsonValue::as_array_mut)
        else {
            continue;
        };
        let entries = std::mem::take(pretool);
        for mut entry in entries {
            let keep = entry
                .get_mut("hooks")
                .and_then(JsonValue::as_array_mut)
                .map(|hooks| {
                    let before = hooks.len();
                    hooks.retain(|hook| {
                        !hook
                            .get("command")
                            .and_then(JsonValue::as_str)
                            .is_some_and(is_cap_agy_hook_command)
                    });
                    removed |= hooks.len() != before;
                    !hooks.is_empty()
                })
                .unwrap_or(true);
            if keep {
                pretool.push(entry);
            }
        }
        if pretool.is_empty() {
            config.remove("PreToolUse");
        }
        if config.is_empty() {
            root_obj.remove(&key);
        }
    }
    Ok(removed)
}

fn is_cap_agy_hook_command(s: &str) -> bool {
    is_cap_hook_command_kind(s, "agy")
}

// ---------------------------------------------------------------- shared

enum MergeStatus {
    Installed,
    AlreadyPresent,
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl MergeStatus {
    fn describe(&self) -> &'static str {
        match self {
            MergeStatus::Installed => "installed cap hook into",
            MergeStatus::AlreadyPresent => "cap hook already present in",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cap_command_detection() {
        assert!(is_cap_hook_command("cap hook bash"));
        assert!(is_cap_hook_command("/usr/local/bin/cap hook bash"));
        assert!(is_cap_hook_command("/abs/path/cap hook bash --foo"));
        assert!(!is_cap_hook_command("npm hook bash")); // wrong program
        assert!(!is_cap_hook_command("cap status")); // not the bash hook
        assert!(is_cap_agy_hook_command("cap hook agy"));
        assert!(!is_cap_agy_hook_command("cap hook bash"));
        assert!(!is_cap_hook_command("")); // empty
    }

    #[test]
    fn claude_install_creates_new_file_then_idempotent() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        let cmd = "/usr/local/bin/cap hook bash";

        assert!(matches!(
            merge_claude(&path, cmd).unwrap(),
            MergeStatus::Installed
        ));
        assert!(path.exists());
        // Second call: idempotent.
        assert!(matches!(
            merge_claude(&path, cmd).unwrap(),
            MergeStatus::AlreadyPresent
        ));

        // Verify the structure round-trips.
        let v: JsonValue = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let pretool = v.pointer("/hooks/PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pretool.len(), 1);
        assert_eq!(
            pretool[0]
                .pointer("/hooks/0/command")
                .unwrap()
                .as_str()
                .unwrap(),
            cmd
        );
    }

    #[test]
    fn claude_install_preserves_existing_unrelated_hooks() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        // Seed an unrelated hook the user already had.
        let seed = json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Edit",
                    "hooks": [{ "type": "command", "command": "some-other-tool" }]
                }],
                "PostToolUse": []
            },
            "model": "claude-opus-4-7"
        });
        std::fs::write(&path, serde_json::to_string_pretty(&seed).unwrap()).unwrap();

        let cmd = "/abs/cap hook bash";
        merge_claude(&path, cmd).unwrap();

        let v: JsonValue = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v.get("model").unwrap().as_str().unwrap(), "claude-opus-4-7");
        let pretool = v.pointer("/hooks/PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pretool.len(), 2, "unrelated hook must be preserved");
        // Existing entry intact.
        assert_eq!(
            pretool[0]
                .pointer("/hooks/0/command")
                .unwrap()
                .as_str()
                .unwrap(),
            "some-other-tool"
        );
        // Our entry appended.
        assert_eq!(
            pretool[1]
                .pointer("/hooks/0/command")
                .unwrap()
                .as_str()
                .unwrap(),
            cmd
        );
    }

    #[test]
    fn codex_install_creates_new_file_then_idempotent() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let cmd = "/usr/local/bin/cap hook bash";

        assert!(matches!(
            merge_codex(&path, cmd).unwrap(),
            MergeStatus::Installed
        ));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("[[hooks.PreToolUse]]"));
        assert!(text.contains(cmd));

        assert!(matches!(
            merge_codex(&path, cmd).unwrap(),
            MergeStatus::AlreadyPresent
        ));
    }

    #[test]
    fn codex_install_preserves_existing_keys() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "model = \"gpt-5\"\n").unwrap();

        let cmd = "/abs/cap hook bash";
        merge_codex(&path, cmd).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("model = \"gpt-5\""), "existing key preserved");
        assert!(text.contains("[[hooks.PreToolUse]]"), "hook added");
        assert!(text.contains(cmd));
    }

    #[test]
    fn claude_disable_removes_only_cap_hooks_and_is_idempotent() {
        let mut root = json!({
            "model": "claude-opus-4-7",
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "/opt/bin/cap hook bash --claude-code" },
                        { "type": "command", "command": "keep-this-hook" }
                    ]
                }, {
                    "matcher": "Edit",
                    "hooks": [{ "type": "command", "command": "other-hook" }]
                }],
                "PostToolUse": []
            }
        });

        assert!(remove_claude_cap_hooks(&mut root).unwrap());
        assert!(!remove_claude_cap_hooks(&mut root).unwrap());
        assert_eq!(root["model"], "claude-opus-4-7");
        let pretool = root
            .pointer("/hooks/PreToolUse")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(pretool.len(), 2);
        assert_eq!(pretool[0]["hooks"].as_array().unwrap().len(), 1);
        assert_eq!(pretool[0]["hooks"][0]["command"], "keep-this-hook");
        assert!(!pretool_has_cap_hook(pretool));
    }

    #[test]
    fn claude_disable_cleans_empty_cap_entry_without_removing_other_settings() {
        let mut root = json!({
            "model": "claude-opus-4-7",
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": "cap hook bash --claude-code" }]
                }],
                "PostToolUse": []
            }
        });

        assert!(remove_claude_cap_hooks(&mut root).unwrap());
        assert!(root.pointer("/hooks/PreToolUse").is_none());
        assert!(root.pointer("/hooks/PostToolUse").is_some());
        assert_eq!(root["model"], "claude-opus-4-7");
    }

    #[test]
    fn codex_disable_removes_only_cap_hooks_and_is_idempotent() {
        let mut root: toml::Value = r#"
model = "gpt-5"

[[hooks.PreToolUse]]
matcher = "^Bash$"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "/opt/bin/cap hook bash --codex"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "keep-this-hook"

[[hooks.PreToolUse]]
matcher = "^Edit$"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "other-hook"
"#
        .parse()
        .unwrap();

        assert!(remove_codex_cap_hooks(&mut root).unwrap());
        assert!(!remove_codex_cap_hooks(&mut root).unwrap());
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        let pretool = root["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pretool.len(), 2);
        assert_eq!(pretool[0]["hooks"].as_array().unwrap().len(), 1);
        assert_eq!(
            pretool[0]["hooks"][0]["command"].as_str(),
            Some("keep-this-hook")
        );
        assert!(!codex_pretool_has_cap_hook(pretool));
    }

    #[test]
    fn agy_install_disable_and_idempotency_preserve_other_entries() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("hooks.json");
        std::fs::write(
            &path,
            r#"{"other":{"PreToolUse":[{"matcher":"run_command","hooks":[{"command":"keep-this-hook"}]}]},"setting":true}"#,
        )
        .unwrap();
        let cmd = "/opt/bin/cap hook agy";

        assert!(matches!(
            merge_agy(&path, cmd).unwrap(),
            MergeStatus::Installed
        ));
        assert!(matches!(
            merge_agy(&path, cmd).unwrap(),
            MergeStatus::AlreadyPresent
        ));
        let mut root: JsonValue =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(agy_has_cap_hook(&root));
        assert!(remove_agy_cap_hooks(&mut root).unwrap());
        assert!(!remove_agy_cap_hooks(&mut root).unwrap());
        assert_eq!(root["setting"], true);
        assert_eq!(
            root["other"]["PreToolUse"][0]["hooks"][0]["command"],
            "keep-this-hook"
        );
        assert!(!agy_has_cap_hook(&root));
    }

    #[test]
    fn hook_status_labels_cover_all_reported_states() {
        assert_eq!(HookState::Enabled.label(), "enabled");
        assert_eq!(HookState::Disabled.label(), "disabled");
        assert_eq!(Agent::Claude.label(), "claude");
        assert_eq!(Agent::Codex.label(), "codex");
        assert_eq!(Agent::Agy.label(), "agy");
    }

    #[test]
    fn hook_status_detects_enabled_disabled_and_unreadable_configs() {
        let tmp = TempDir::new().unwrap();
        let claude = tmp.path().join("settings.json");
        let codex = tmp.path().join("config.toml");

        assert_eq!(claude_hook_status_at(&claude).unwrap(), HookState::Disabled);
        assert_eq!(codex_hook_status_at(&codex).unwrap(), HookState::Disabled);

        std::fs::write(
            &claude,
            r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"/bin/cap hook bash --claude-code"}]}]}}"#,
        )
        .unwrap();
        std::fs::write(
            &codex,
            "[[hooks.PreToolUse]]\n[[hooks.PreToolUse.hooks]]\ncommand = \"/bin/cap hook bash --codex\"\n",
        )
        .unwrap();
        assert_eq!(claude_hook_status_at(&claude).unwrap(), HookState::Enabled);
        assert_eq!(codex_hook_status_at(&codex).unwrap(), HookState::Enabled);

        std::fs::write(&claude, "{").unwrap();
        std::fs::write(&codex, "[").unwrap();
        assert!(claude_hook_status_at(&claude).is_err());
        assert!(codex_hook_status_at(&codex).is_err());
    }
}
// CODEGEN-END
