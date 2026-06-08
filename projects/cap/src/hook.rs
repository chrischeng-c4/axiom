// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Claude Code / Codex CLI `PreToolUse` hook adapter.
//!
//! Reads the hook event JSON from stdin, decides whether the
//! `Bash` command should be wrapped with `cap`, and writes a
//! agent-specific `hookSpecificOutput` JSON to stdout.
//!
//! Strategy: **wrap every external Bash command** so the daemon
//! always sees the process group and can pause / kill it under
//! memory pressure. Rewrite form is
//! `cap run --label='<orig>' -- bash -c '<escaped>'`, which lets bash
//! handle shell builtins (`cd`, `export`), pipes, redirections, `&&`
//! chains, and heredocs — cap just sees one bash process group — while
//! `--label` keeps the original command in the run log (otherwise every
//! entry would read `bash -c …`).
//!
//! Per-invocation overhead: ~10 ms cap startup + ~5 ms extra
//! bash layer. Imperceptible for any non-trivial command.
//!
//! Decision logic:
//!
//!   1. Not a Bash tool call → allow.
//!   2. Empty command → allow.
//!   3. Effective first token (after stripping `env`, `time`,
//!      `nice`, `nohup`, `exec`, and leading `VAR=val`) is
//!      already `cap` → allow (avoid recursive wrapping).
//!   4. Anything else → allow + input rewrite command
//!      `"cap run --label='<orig>' -- bash -c '<escaped original>'"`.
//!
//! Always exits 0 — a hook crash must never wedge the agent.

use std::io::Read;

use serde::{Deserialize, Serialize};

/// Wrappers stripped while looking for the "real" first program.
const PASSTHROUGH_WRAPPERS: &[&str] = &["env", "time", "nice", "nohup", "exec"];

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookAgent {
    Auto,
    Claude,
    Codex,
}

#[derive(Debug, Deserialize)]
struct HookInput {
    turn_id: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
    tool_use_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    command: Option<String>,
}

#[derive(Debug, Serialize)]
struct HookOutput {
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: HookSpecific,
}

#[derive(Debug, Serialize)]
struct HookSpecific {
    #[serde(rename = "hookEventName")]
    hook_event_name: &'static str,
    #[serde(rename = "permissionDecision")]
    permission_decision: &'static str,
    #[serde(rename = "modifiedInput", skip_serializing_if = "Option::is_none")]
    modified_input: Option<ModifiedInput>,
    #[serde(rename = "updatedInput", skip_serializing_if = "Option::is_none")]
    updated_input: Option<ModifiedInput>,
    #[serde(
        rename = "permissionDecisionReason",
        skip_serializing_if = "Option::is_none"
    )]
    permission_decision_reason: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct ModifiedInput {
    command: String,
}

/// Read JSON from stdin, decide, print JSON to stdout. Always
/// returns Ok — the binary exits 0 even on malformed input.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn run_bash_hook(agent: HookAgent) -> anyhow::Result<()> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let input: HookInput = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    let is_bash = input.tool_name.as_deref() == Some("Bash");
    let command = input
        .tool_input
        .as_ref()
        .and_then(|t| t.command.as_deref())
        .unwrap_or("");

    if !is_bash || command.is_empty() {
        return Ok(());
    }

    // Use the absolute path of THIS binary (the hook is `<abs>/cap hook
    // bash`, so `current_exe()` is the installed cap) rather than a bare
    // `cap`. The rewritten command runs in the agent's shell, whose PATH
    // we don't control — a bare `cap` would become "command not found"
    // and the agent's command would silently never run. Falls back to
    // `cap` only if the exe path can't be resolved.
    let cap_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| "cap".to_string());

    let Some(rewritten) = maybe_rewrite(command, &cap_bin) else {
        return Ok(());
    };

    let rewrite = ModifiedInput { command: rewritten };
    let agent = resolve_agent(agent, &input);
    let out = HookOutput {
        hook_specific_output: hook_specific_for_rewrite(agent, rewrite),
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

fn resolve_agent(agent: HookAgent, input: &HookInput) -> HookAgent {
    match agent {
        HookAgent::Auto if input.tool_use_id.is_some() || input.turn_id.is_some() => {
            HookAgent::Codex
        }
        HookAgent::Auto => HookAgent::Claude,
        explicit => explicit,
    }
}

fn hook_specific_for_rewrite(agent: HookAgent, rewrite: ModifiedInput) -> HookSpecific {
    let (modified_input, updated_input) = match agent {
        HookAgent::Claude => (Some(rewrite), None),
        HookAgent::Codex => (None, Some(rewrite)),
        HookAgent::Auto => unreachable!("auto hook agent must be resolved before output"),
    };

    HookSpecific {
        hook_event_name: "PreToolUse",
        permission_decision: "allow",
        modified_input,
        updated_input,
        permission_decision_reason: Some(
            "wrapped with cap so the daemon can throttle under memory pressure".into(),
        ),
    }
}

/// Returns the rewritten command, or None if no rewrite is needed.
/// `cap_bin` is the path to invoke cap with — production passes the
/// absolute path of the running binary; tests pass `"cap"` for stable
/// expectations.
fn maybe_rewrite(command: &str, cap_bin: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    if first_program_is_cap(trimmed) {
        return None;
    }
    // `cap run --label=<orig> -- bash -c <orig>`:
    //   * the `bash -c` layer is what actually runs (so cap sees one
    //     process group for the whole shell line — pipes, &&, builtins),
    //   * `--label` carries the original command verbatim so the run log
    //     records `ls -la | wc -l`, not `bash -c ls -la | wc -l`.
    // `--label=` (attached form) sidesteps clap treating a command that
    // starts with `-` as a flag.
    let quoted = shell_single_quote(command);
    Some(format!(
        "{} run --label={quoted} -- bash -c {quoted}",
        shell_quote_arg(cap_bin),
    ))
}

/// Quote the cap binary path for use as the first word of a shell
/// command, but only when it actually needs it. A clean path
/// (`cap`, `/home/u/.local/bin/cap`) is emitted verbatim so the
/// rewrite stays readable; a path with spaces or shell metacharacters
/// (e.g. `/Users/My Name/.local/bin/cap`) gets single-quoted.
fn shell_quote_arg(s: &str) -> String {
    let safe = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'));
    if safe {
        s.to_string()
    } else {
        shell_single_quote(s)
    }
}

/// True iff the first non-wrapper, non-assignment token resolves
/// to `cap` (or `/path/to/cap`).
fn first_program_is_cap(command: &str) -> bool {
    for tok in command.split_whitespace() {
        if PASSTHROUGH_WRAPPERS.contains(&tok) {
            continue;
        }
        if is_var_assignment(tok) {
            continue;
        }
        return basename(tok) == "cap";
    }
    false
}

fn is_var_assignment(tok: &str) -> bool {
    let Some(eq) = tok.find('=') else {
        return false;
    };
    let name = &tok[..eq];
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

/// Shells whose `-c <script>` form we look through to find the real
/// program (so a hook-wrapped `bash -c 'cargo test'` reports `cargo`,
/// not `bash`).
const SHELLS: &[&str] = &["bash", "sh", "zsh", "dash", "ksh"];

/// Resolve the *effective* program for logging + kill-strategy hints.
///
/// `cap run` is almost always invoked by the hook as `bash -c '<script>'`,
/// so the literal `program` (`bash`) is useless — the meaningful tool is
/// the first real token of the script. For a shell invocation we look
/// inside `-c <script>`; otherwise we just take the basename. Returns a
/// basename like `cargo` / `pytest` / `bash`.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn effective_program(program: &str, args: &[String]) -> String {
    let base = basename(program);
    if SHELLS.contains(&base) {
        if let Some(pos) = args.iter().position(|a| a == "-c") {
            if let Some(script) = args.get(pos + 1) {
                if let Some(prog) = script_main_program(script) {
                    return prog;
                }
            }
        }
    }
    base.to_string()
}

/// Best-effort "what's the real program" for a shell script line.
///
/// Splits on sequencing operators (`&&`, `||`, `;`, `|`) into segments,
/// then returns the first segment's leading program (skipping env
/// wrappers and `VAR=val` assignments) that isn't `cd`. So
/// `cd foo && cargo test` and `cd a; cd b; pytest` both resolve past the
/// `cd` prefixes. Not a real shell lexer — good enough for log labels
/// and kill-strategy hints.
fn script_main_program(script: &str) -> Option<String> {
    // Normalize every operator to one sentinel so a plain split isolates
    // segments regardless of surrounding whitespace (`a;b`, `a && b`).
    const SEP: char = '\u{1}';
    let norm = script
        .replace("&&", "\u{1}")
        .replace("||", "\u{1}")
        .replace([';', '|'], "\u{1}");
    for segment in norm.split(SEP) {
        let prog = segment
            .split_whitespace()
            .find(|t| !PASSTHROUGH_WRAPPERS.contains(t) && !is_var_assignment(t))
            .map(basename);
        match prog {
            Some("cd") => continue, // leading `cd <dir>` — keep looking
            Some(p) => return Some(p.to_string()),
            None => continue,
        }
    }
    None
}

/// Wrap `s` in POSIX-shell single quotes, escaping any embedded
/// single quotes via the standard `'\''` pattern. The result is
/// safe to splice directly after `bash -c `.
fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            // Close the quote, emit an escaped single quote, reopen.
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn input_with_agent_ids(turn_id: Option<&str>, tool_use_id: Option<&str>) -> HookInput {
        HookInput {
            turn_id: turn_id.map(str::to_string),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(ToolInput {
                command: Some("pwd".to_string()),
            }),
            tool_use_id: tool_use_id.map(str::to_string),
        }
    }

    #[test]
    fn auto_detects_codex_hook_payload() {
        let input = input_with_agent_ids(Some("turn-1"), Some("tool-1"));
        assert_eq!(resolve_agent(HookAgent::Auto, &input), HookAgent::Codex);
    }

    #[test]
    fn auto_defaults_to_claude_hook_payload() {
        let input = input_with_agent_ids(None, None);
        assert_eq!(resolve_agent(HookAgent::Auto, &input), HookAgent::Claude);
    }

    #[test]
    fn codex_output_uses_updated_input_only() {
        let output = HookOutput {
            hook_specific_output: hook_specific_for_rewrite(
                HookAgent::Codex,
                ModifiedInput {
                    command: "cap run -- bash -c pwd".to_string(),
                },
            ),
        };

        let value: Value = serde_json::to_value(output).unwrap();
        let hook_specific = value.get("hookSpecificOutput").unwrap();
        assert!(hook_specific.get("updatedInput").is_some());
        assert!(hook_specific.get("modifiedInput").is_none());
    }

    #[test]
    fn claude_output_uses_modified_input_only() {
        let output = HookOutput {
            hook_specific_output: hook_specific_for_rewrite(
                HookAgent::Claude,
                ModifiedInput {
                    command: "cap run -- bash -c pwd".to_string(),
                },
            ),
        };

        let value: Value = serde_json::to_value(output).unwrap();
        let hook_specific = value.get("hookSpecificOutput").unwrap();
        assert!(hook_specific.get("modifiedInput").is_some());
        assert!(hook_specific.get("updatedInput").is_none());
    }

    // Tests pin `cap_bin = "cap"` for stable expectations; production
    // passes the absolute path of the running binary (see
    // `absolute_cap_path_*` below for that path).
    fn rewrite(command: &str) -> Option<String> {
        maybe_rewrite(command, "cap")
    }

    #[test]
    fn wraps_plain_command() {
        assert_eq!(
            rewrite("cargo test -p cap").unwrap(),
            "cap run --label='cargo test -p cap' -- bash -c 'cargo test -p cap'"
        );
    }

    #[test]
    fn wraps_lightweight_command_too() {
        // The whole point — every external command goes through cap
        // so the daemon sees uniform process groups.
        assert_eq!(
            rewrite("ls -la").unwrap(),
            "cap run --label='ls -la' -- bash -c 'ls -la'"
        );
        assert_eq!(
            rewrite("git status").unwrap(),
            "cap run --label='git status' -- bash -c 'git status'"
        );
    }

    #[test]
    fn wraps_shell_pipeline() {
        assert_eq!(
            rewrite("ls -la | wc -l").unwrap(),
            "cap run --label='ls -la | wc -l' -- bash -c 'ls -la | wc -l'"
        );
        assert_eq!(
            rewrite("cd projects/cap && cargo test").unwrap(),
            "cap run --label='cd projects/cap && cargo test' -- bash -c 'cd projects/cap && cargo test'"
        );
    }

    #[test]
    fn label_carries_the_clean_command_for_the_run_log() {
        // The `--label=` segment must be the verbatim original command
        // (this is what the run log records as `command`).
        let got = rewrite("pytest -k foo").unwrap();
        assert!(
            got.contains("--label='pytest -k foo'"),
            "label must carry the clean original command, got {got}"
        );
        assert!(
            got.ends_with("-- bash -c 'pytest -k foo'"),
            "the bash -c payload is still the original command, got {got}"
        );
    }

    #[test]
    fn absolute_cap_path_clean_is_unquoted() {
        // A clean absolute path (the normal install case) is emitted
        // verbatim so the rewrite stays readable.
        assert_eq!(
            maybe_rewrite("cargo test", "/home/u/.local/bin/cap").unwrap(),
            "/home/u/.local/bin/cap run --label='cargo test' -- bash -c 'cargo test'"
        );
    }

    #[test]
    fn absolute_cap_path_with_spaces_is_quoted() {
        // A home dir with a space (common on macOS) must be quoted or
        // the shell would split it into two words.
        assert_eq!(
            maybe_rewrite("cargo test", "/Users/My Name/.local/bin/cap").unwrap(),
            "'/Users/My Name/.local/bin/cap' run --label='cargo test' -- bash -c 'cargo test'"
        );
    }

    #[test]
    fn already_wrapped_passes_through() {
        // Top-level cap invocation — don't double-wrap.
        assert!(rewrite("cap cargo test").is_none());
        assert!(rewrite("cap run -- cargo test").is_none());
        assert!(rewrite("cap status").is_none());
        assert!(rewrite("cap daemon start").is_none());
        // env-prefixed cap invocation.
        assert!(rewrite("FOO=1 cap cargo test").is_none());
        assert!(rewrite("env FOO=1 cap cargo test").is_none());
        assert!(rewrite("/usr/local/bin/cap cargo test").is_none());
    }

    #[test]
    fn empty_command_no_rewrite() {
        assert!(rewrite("").is_none());
        assert!(rewrite("   ").is_none());
    }

    #[test]
    fn single_quote_in_command_escaped() {
        let got = rewrite("echo 'hello world'").unwrap();
        // Both the label and the bash -c payload are escaped the same way.
        let q = "'echo '\\''hello world'\\'''";
        assert_eq!(got, format!("cap run --label={q} -- bash -c {q}"));
    }

    #[test]
    fn newline_preserved_in_quotes() {
        let got = rewrite("for i in 1 2 3\ndo echo $i\ndone").unwrap();
        // Single-quoted multi-line literal — bash handles it natively.
        let q = "'for i in 1 2 3\ndo echo $i\ndone'";
        assert_eq!(got, format!("cap run --label={q} -- bash -c {q}"));
    }

    #[test]
    fn effective_program_looks_through_bash_dash_c() {
        let ep = |args: &[&str]| {
            let v: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            effective_program("bash", &v)
        };
        // Plain command.
        assert_eq!(ep(&["-c", "cargo test -p cap"]), "cargo");
        // Leading `cd <dir> &&` prefix (the common agent pattern).
        assert_eq!(ep(&["-c", "cd projects/cap && cargo test"]), "cargo");
        // Chained cd, semicolon separator.
        assert_eq!(ep(&["-c", "cd a; cd b; pytest -k foo"]), "pytest");
        // env wrapper + assignment skipped.
        assert_eq!(ep(&["-c", "env FOO=1 pytest"]), "pytest");
        // Pipeline → first real token.
        assert_eq!(ep(&["-c", "ls -la | wc -l"]), "ls");
        // Path-qualified program in the script → basename.
        assert_eq!(ep(&["-c", "/usr/local/bin/cargo build"]), "cargo");
    }

    #[test]
    fn effective_program_non_shell_is_basename() {
        // Direct `cap run -- cargo test` (no shell layer) → basename.
        assert_eq!(
            effective_program("/usr/bin/cargo", &["test".into()]),
            "cargo"
        );
        // A shell with no `-c` falls back to the shell name.
        assert_eq!(effective_program("bash", &["script.sh".into()]), "bash");
    }

    #[test]
    fn var_assignment_detector() {
        assert!(is_var_assignment("FOO=bar"));
        assert!(is_var_assignment("_FOO=bar"));
        assert!(is_var_assignment("FOO123=bar"));
        assert!(!is_var_assignment("foo"));
        assert!(!is_var_assignment("=bar"));
        assert!(!is_var_assignment("1FOO=bar"));
    }

    #[test]
    fn shell_quote_handles_specials() {
        // Inside single quotes, EVERYTHING is literal except `'`.
        assert_eq!(shell_single_quote("foo"), "'foo'");
        assert_eq!(shell_single_quote("$HOME"), "'$HOME'");
        assert_eq!(shell_single_quote("a\\b"), "'a\\b'");
        assert_eq!(shell_single_quote("a'b"), "'a'\\''b'");
    }
}
// CODEGEN-END
