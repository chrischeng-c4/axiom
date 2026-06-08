// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Hook registration backend for `cap init` — wires the cap PreToolUse
//! hook into Claude Code or Codex CLI.
//!
//! - Claude Code: merges `hooks.PreToolUse[]` into
//!   `~/.claude/settings.json` (or `.claude/settings.json` with
//!   `--project`).
//! - Codex CLI:   merges `[[hooks.PreToolUse]]` into
//!   `~/.codex/config.toml` (or `.codex/config.toml` with `--project`).
//!
//! Idempotent: if a PreToolUse entry already points at our cap
//! binary it's left in place. Existing unrelated hooks are
//! preserved.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value as JsonValue};

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    Project,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn run(agent: Agent, scope: Scope, print: bool) -> Result<()> {
    // Use the absolute path so the hook fires correctly even when
    // the parent process's PATH doesn't include cap's install dir.
    let cap = std::env::current_exe().context("locating cap binary")?;
    let cap_path = cap.to_string_lossy().to_string();

    match agent {
        Agent::Claude => install_claude(&cap_path, scope, print),
        Agent::Codex => install_codex(&cap_path, scope, print),
    }
}

// ---------------------------------------------------------------- Claude

fn install_claude(cap_path: &str, scope: Scope, print: bool) -> Result<()> {
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

    let path = claude_settings_path(scope)?;
    let merged_status = merge_claude(&path, &hook_cmd)?;
    println!("{}: {}", merged_status.describe(), path.display());
    Ok(())
}

fn claude_settings_path(scope: Scope) -> Result<PathBuf> {
    match scope {
        Scope::User => {
            let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
            Ok(home.join(".claude").join("settings.json"))
        }
        Scope::Project => Ok(std::env::current_dir()?
            .join(".claude")
            .join("settings.json")),
    }
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

fn is_cap_hook_command(s: &str) -> bool {
    // Any command line that ends in `hook bash` (with optional flags)
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
    rest.windows(2).any(|w| w == ["hook", "bash"])
}

// ---------------------------------------------------------------- Codex

fn install_codex(cap_path: &str, scope: Scope, print: bool) -> Result<()> {
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

    let path = codex_config_path(scope)?;
    let merged_status = merge_codex(&path, &hook_cmd)?;
    println!("{}: {}", merged_status.describe(), path.display());
    Ok(())
}

fn codex_config_path(scope: Scope) -> Result<PathBuf> {
    match scope {
        Scope::User => {
            let home = dirs::home_dir().ok_or_else(|| anyhow!("no $HOME"))?;
            Ok(home.join(".codex").join("config.toml"))
        }
        Scope::Project => Ok(std::env::current_dir()?.join(".codex").join("config.toml")),
    }
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

// ---------------------------------------------------------------- shared

enum MergeStatus {
    Installed,
    AlreadyPresent,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
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
}
// CODEGEN-END
