// `uv tool install` — isolated tool environments (Tick 40).
//
// Mirrors uv's "tool" subcommand family:
//   * `tool install <pkg>` — drop a wheel + its closure into a
//     dedicated venv at `<tool_root>/<name>/`, expose its
//     entry-point launchers from `<bin_root>/`.
//   * `tool list` — enumerate installed tool receipts.
//   * `tool uninstall <pkg>` — verify + delete the venv + remove
//     launchers it owns (no orphan launchers).
//   * `tool upgrade <pkg>` — recompute closure, diff against the
//     pinned receipt, write the updated receipt only after the new
//     venv replaces the old one atomically.
//
// What's in this module:
//   * `ToolLayout` — derives every per-tool path from
//     `(tool_root, bin_root, tool_name)`; pure data.
//   * `ToolReceipt` — the on-disk state file written into each
//     tool venv as `receipt.toml`. Captures the *pin* (name, version,
//     extras, requires-python, dep closure, entry-point inventory) so
//     `list` / `upgrade` / `uninstall` can act without reaching back
//     to the index. Round-trips through TOML deterministically.
//   * `default_tool_root` / `default_bin_root` — XDG-aware path
//     resolution matching uv's `~/.local/share/uv/tools` and
//     `~/.local/bin` defaults.
//   * `plan_uninstall` — pure decision: given a receipt + the
//     current `<bin_root>` listing, decide which launchers belong to
//     this tool (so we never delete a launcher that another tool
//     also exposes).
//   * `plan_upgrade` — diff a candidate `ToolReceipt` against an
//     existing one; produce `UpgradeDecision` (NoOp / Reinstall /
//     EntryPointDelta).
//
// Driver-layer work (`install_tool`, etc.) is intentionally NOT in
// this Tick — it's the natural composition of
// {venv (Tick 37), wheel_build (39), installer/scripts (existing),
//  cache/cache_prune (28/38)}. Wiring them together lives in a
// future cli-side tick once we're ready to expose the verb.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Per-tool layout. `venv_root` is the prefix that `venv::create_venv`
/// would lay PEP 405 files into; `launchers_dir` is the user's
/// `bin/` that hosts the launcher symlinks/scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLayout {
    pub tool_name: String,
    pub venv_root: PathBuf,
    pub launchers_dir: PathBuf,
    pub receipt_path: PathBuf,
}

impl ToolLayout {
    /// Build a layout from three inputs:
    ///   * `tool_root` — where per-tool venvs live (default
    ///     `~/.local/share/uv/tools`).
    ///   * `bin_root` — where launcher scripts go (default
    ///     `~/.local/bin`).
    ///   * `tool_name` — the user-facing project name; we PEP 503-
    ///     normalize for the venv directory but keep the launcher
    ///     names verbatim (entry-point names are case-sensitive).
    pub fn for_name(tool_root: &Path, bin_root: &Path, tool_name: &str) -> Self {
        let normalized = pep503_normalize(tool_name);
        let venv_root = tool_root.join(&normalized);
        let receipt_path = venv_root.join("receipt.toml");
        ToolLayout {
            tool_name: normalized,
            venv_root,
            launchers_dir: bin_root.to_path_buf(),
            receipt_path,
        }
    }
}

/// On-disk record of one installed tool. Every field is what a
/// future `tool upgrade` / `tool uninstall` needs to act without an
/// index round-trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolReceipt {
    /// Schema version pin so we can evolve the format.
    pub schema_version: u32,
    /// PEP 503-normalized project name (matches `ToolLayout.tool_name`).
    pub name: String,
    /// Pinned version (PEP 440 string).
    pub version: String,
    /// Extras requested at install time, sorted.
    pub extras: Vec<String>,
    /// Optional Requires-Python lower bound recorded at install.
    pub requires_python: Option<String>,
    /// Full transitive dep closure (`name -> pinned version`).
    /// BTreeMap so iteration is sorted + deterministic.
    pub closure: BTreeMap<String, String>,
    /// Entry-point launcher names owned by this tool (the basenames
    /// under `bin_root`). Verbatim, *not* normalized.
    pub launchers: Vec<String>,
    /// Python interpreter the tool venv was built on (absolute path).
    /// Used so we can warn / refuse to upgrade if the interpreter has
    /// moved.
    pub python: PathBuf,
}

impl ToolReceipt {
    pub const CURRENT_SCHEMA: u32 = 1;

    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        python: impl Into<PathBuf>,
    ) -> Self {
        ToolReceipt {
            schema_version: Self::CURRENT_SCHEMA,
            name: name.into(),
            version: version.into(),
            extras: Vec::new(),
            requires_python: None,
            closure: BTreeMap::new(),
            launchers: Vec::new(),
            python: python.into(),
        }
    }
}

/// Render a `ToolReceipt` to a deterministic TOML body. Keys come in
/// canonical order; the closure table is sorted alphabetically.
pub fn render_receipt_toml(receipt: &ToolReceipt) -> String {
    let mut out = String::new();
    out.push_str(&format!("schema_version = {}\n", receipt.schema_version));
    out.push_str(&format!("name = {}\n", toml_quote(&receipt.name)));
    out.push_str(&format!("version = {}\n", toml_quote(&receipt.version)));
    out.push_str(&format!(
        "python = {}\n",
        toml_quote(&receipt.python.display().to_string())
    ));
    if let Some(rp) = &receipt.requires_python {
        out.push_str(&format!("requires_python = {}\n", toml_quote(rp)));
    }
    // Lists in alphabetical order for deterministic diffs.
    let mut extras = receipt.extras.clone();
    extras.sort();
    out.push_str("extras = [");
    push_str_list(&mut out, &extras);
    out.push_str("]\n");

    let mut launchers = receipt.launchers.clone();
    launchers.sort();
    out.push_str("launchers = [");
    push_str_list(&mut out, &launchers);
    out.push_str("]\n");

    if !receipt.closure.is_empty() {
        out.push('\n');
        out.push_str("[closure]\n");
        for (name, version) in &receipt.closure {
            out.push_str(&format!("{name} = {}\n", toml_quote(version)));
        }
    }
    out
}

fn push_str_list(out: &mut String, xs: &[String]) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&toml_quote(x));
    }
}

fn toml_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Parse a `ToolReceipt` from a TOML body. Errors surface as
/// `IndexError::ParseError` with a stable `<receipt.toml>` URL.
pub fn parse_receipt_toml(src: &str) -> Result<ToolReceipt, IndexError> {
    let doc: toml::Value = toml::from_str(src).map_err(|e| IndexError::ParseError {
        url: "<receipt.toml>".into(),
        detail: format!("receipt.toml: {e}"),
    })?;
    let table = doc.as_table().ok_or_else(|| IndexError::ParseError {
        url: "<receipt.toml>".into(),
        detail: "receipt.toml: top-level must be a table".into(),
    })?;

    let schema_version = table
        .get("schema_version")
        .and_then(|v| v.as_integer())
        .ok_or_else(|| ParseError("schema_version missing or non-integer"))?
        as u32;
    if schema_version > ToolReceipt::CURRENT_SCHEMA {
        return Err(IndexError::ParseError {
            url: "<receipt.toml>".into(),
            detail: format!(
                "receipt schema_version {schema_version} newer than supported ({})",
                ToolReceipt::CURRENT_SCHEMA
            ),
        });
    }
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError("name missing"))?
        .to_string();
    let version = table
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError("version missing"))?
        .to_string();
    let python = table
        .get("python")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError("python missing"))?
        .to_string();
    let requires_python = table
        .get("requires_python")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let extras = table
        .get("extras")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let launchers = table
        .get("launchers")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut closure = BTreeMap::new();
    if let Some(t) = table.get("closure").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(ver) = v.as_str() {
                closure.insert(k.clone(), ver.to_string());
            }
        }
    }

    Ok(ToolReceipt {
        schema_version,
        name,
        version,
        extras,
        requires_python,
        closure,
        launchers,
        python: PathBuf::from(python),
    })
}

#[allow(non_snake_case)]
fn ParseError(detail: &str) -> IndexError {
    IndexError::ParseError {
        url: "<receipt.toml>".into(),
        detail: format!("receipt.toml: {detail}"),
    }
}

/// Decision returned by `plan_upgrade`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeDecision {
    /// New receipt is identical to the current one — nothing to do.
    NoOp,
    /// Version, extras, requires-python, or any closure entry changed
    /// — full reinstall required.
    Reinstall { changes: Vec<UpgradeChange> },
    /// Only the launcher list changed — caller may patch the bin dir
    /// without rebuilding the venv. Rare in practice but worth
    /// catching so we don't reinstall for no-op cases.
    LauncherDelta {
        added: Vec<String>,
        removed: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeChange {
    Version {
        from: String,
        to: String,
    },
    Extras {
        from: Vec<String>,
        to: Vec<String>,
    },
    RequiresPython {
        from: Option<String>,
        to: Option<String>,
    },
    Python {
        from: PathBuf,
        to: PathBuf,
    },
    ClosureAdded {
        name: String,
        version: String,
    },
    ClosureRemoved {
        name: String,
        version: String,
    },
    ClosureChanged {
        name: String,
        from: String,
        to: String,
    },
}

/// Pure diff: produce an upgrade plan from a current receipt to a
/// candidate one. Both must be for the same `name` — caller is
/// responsible for checking that before invoking.
pub fn plan_upgrade(current: &ToolReceipt, candidate: &ToolReceipt) -> UpgradeDecision {
    let mut changes = Vec::new();

    if current.version != candidate.version {
        changes.push(UpgradeChange::Version {
            from: current.version.clone(),
            to: candidate.version.clone(),
        });
    }
    if current.extras != candidate.extras {
        changes.push(UpgradeChange::Extras {
            from: current.extras.clone(),
            to: candidate.extras.clone(),
        });
    }
    if current.requires_python != candidate.requires_python {
        changes.push(UpgradeChange::RequiresPython {
            from: current.requires_python.clone(),
            to: candidate.requires_python.clone(),
        });
    }
    if current.python != candidate.python {
        changes.push(UpgradeChange::Python {
            from: current.python.clone(),
            to: candidate.python.clone(),
        });
    }
    let cur_keys: BTreeSet<&String> = current.closure.keys().collect();
    let new_keys: BTreeSet<&String> = candidate.closure.keys().collect();
    for added in new_keys.difference(&cur_keys) {
        changes.push(UpgradeChange::ClosureAdded {
            name: (*added).clone(),
            version: candidate.closure[*added].clone(),
        });
    }
    for removed in cur_keys.difference(&new_keys) {
        changes.push(UpgradeChange::ClosureRemoved {
            name: (*removed).clone(),
            version: current.closure[*removed].clone(),
        });
    }
    for both in cur_keys.intersection(&new_keys) {
        let from = &current.closure[*both];
        let to = &candidate.closure[*both];
        if from != to {
            changes.push(UpgradeChange::ClosureChanged {
                name: (*both).clone(),
                from: from.clone(),
                to: to.clone(),
            });
        }
    }

    if !changes.is_empty() {
        return UpgradeDecision::Reinstall { changes };
    }

    // Same package internals — see if only the launcher list moved.
    if current.launchers != candidate.launchers {
        let cur: BTreeSet<&String> = current.launchers.iter().collect();
        let new: BTreeSet<&String> = candidate.launchers.iter().collect();
        let added: Vec<String> = new.difference(&cur).map(|s| (*s).clone()).collect();
        let removed: Vec<String> = cur.difference(&new).map(|s| (*s).clone()).collect();
        return UpgradeDecision::LauncherDelta { added, removed };
    }

    UpgradeDecision::NoOp
}

/// Pure planning function for `tool uninstall`. Given the receipt of
/// the tool we're removing and the set of *other* receipts present in
/// `tool_root`, return the launcher basenames it's safe to delete
/// from `bin_root` (launchers another tool also exposes are spared so
/// we never break a coexisting `tool install`).
pub fn plan_launcher_removal(target: &ToolReceipt, other_receipts: &[ToolReceipt]) -> Vec<String> {
    let claimed: BTreeSet<&String> = other_receipts
        .iter()
        .flat_map(|r| r.launchers.iter())
        .collect();
    target
        .launchers
        .iter()
        .filter(|name| !claimed.contains(*name))
        .cloned()
        .collect()
}

/// Default per-tool root (`~/.local/share/uv/tools`, matching uv).
/// Falls back to `$TMPDIR/uv-tools` if `$HOME` is unset.
pub fn default_tool_root() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("uv/tools");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".local/share/uv/tools");
        }
    }
    std::env::temp_dir().join("uv-tools")
}

/// Default launcher directory (`~/.local/bin`).
pub fn default_bin_root() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".local/bin");
        }
    }
    std::env::temp_dir().join("uv-bin")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt(name: &str, version: &str) -> ToolReceipt {
        ToolReceipt::new(name, version, "/usr/bin/python3")
    }

    #[test]
    fn layout_normalizes_name_for_dir_but_not_launchers() {
        let layout = ToolLayout::for_name(Path::new("/tools"), Path::new("/bin"), "My_Tool.X");
        assert_eq!(layout.tool_name, "my-tool-x");
        assert_eq!(layout.venv_root, PathBuf::from("/tools/my-tool-x"));
        assert_eq!(
            layout.receipt_path,
            PathBuf::from("/tools/my-tool-x/receipt.toml")
        );
        assert_eq!(layout.launchers_dir, PathBuf::from("/bin"));
    }

    #[test]
    fn receipt_roundtrips_through_toml() {
        let mut r = receipt("Httpie", "3.2.0");
        r.name = "httpie".into();
        r.extras = vec!["socks".into()];
        r.requires_python = Some(">=3.8".into());
        r.closure.insert("requests".into(), "2.31.0".into());
        r.closure.insert("certifi".into(), "2024.2.2".into());
        r.launchers = vec!["http".into(), "https".into(), "httpie".into()];
        let body = render_receipt_toml(&r);
        let back = parse_receipt_toml(&body).unwrap();
        // Renderer normalizes list order alphabetically for stable diffs;
        // sort the original so equality holds across the round-trip.
        r.launchers.sort();
        r.extras.sort();
        assert_eq!(back, r);
    }

    #[test]
    fn receipt_render_is_deterministic_for_unordered_input() {
        let mut a = receipt("x", "1");
        a.launchers = vec!["b".into(), "a".into()];
        a.extras = vec!["socks".into(), "all".into()];
        a.closure.insert("zzz".into(), "1".into());
        a.closure.insert("aaa".into(), "1".into());
        let body1 = render_receipt_toml(&a);

        // Re-shuffle inputs; output must be identical.
        a.launchers.reverse();
        a.extras.reverse();
        let body2 = render_receipt_toml(&a);
        assert_eq!(body1, body2, "renderer must sort lists for stable diffs");
    }

    #[test]
    fn parse_rejects_future_schema() {
        let body = "\
schema_version = 99
name = \"x\"
version = \"1\"
python = \"/usr/bin/python3\"
extras = []
launchers = []
";
        let err = parse_receipt_toml(body).unwrap_err();
        assert!(format!("{err}").contains("newer than supported"));
    }

    #[test]
    fn parse_errors_when_required_field_missing() {
        let body = "\
schema_version = 1
version = \"1\"
python = \"/usr/bin/python3\"
extras = []
launchers = []
";
        let err = parse_receipt_toml(body).unwrap_err();
        assert!(format!("{err}").contains("name"));
    }

    #[test]
    fn upgrade_noop_when_receipts_match() {
        let a = receipt("x", "1.0");
        let b = receipt("x", "1.0");
        assert_eq!(plan_upgrade(&a, &b), UpgradeDecision::NoOp);
    }

    #[test]
    fn upgrade_reinstall_on_version_change() {
        let a = receipt("x", "1.0");
        let b = receipt("x", "1.1");
        match plan_upgrade(&a, &b) {
            UpgradeDecision::Reinstall { changes } => {
                assert!(matches!(
                    changes.as_slice(),
                    [UpgradeChange::Version { from, to }] if from == "1.0" && to == "1.1"
                ));
            }
            other => panic!("expected Reinstall, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_reinstall_on_closure_change() {
        let mut a = receipt("x", "1.0");
        a.closure.insert("requests".into(), "2.30.0".into());
        let mut b = receipt("x", "1.0");
        b.closure.insert("requests".into(), "2.31.0".into());
        match plan_upgrade(&a, &b) {
            UpgradeDecision::Reinstall { changes } => {
                assert!(changes.iter().any(|c| matches!(
                    c,
                    UpgradeChange::ClosureChanged { name, from, to }
                    if name == "requests" && from == "2.30.0" && to == "2.31.0"
                )));
            }
            other => panic!("expected Reinstall, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_reinstall_on_closure_added_and_removed() {
        let mut a = receipt("x", "1.0");
        a.closure.insert("oldlib".into(), "1".into());
        let mut b = receipt("x", "1.0");
        b.closure.insert("newlib".into(), "2".into());
        match plan_upgrade(&a, &b) {
            UpgradeDecision::Reinstall { changes } => {
                assert!(changes.iter().any(|c| matches!(
                    c,
                    UpgradeChange::ClosureAdded { name, .. } if name == "newlib"
                )));
                assert!(changes.iter().any(|c| matches!(
                    c,
                    UpgradeChange::ClosureRemoved { name, .. } if name == "oldlib"
                )));
            }
            other => panic!("expected Reinstall, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_launcher_delta_when_only_launchers_differ() {
        let mut a = receipt("x", "1.0");
        a.launchers = vec!["x".into(), "x-old".into()];
        let mut b = receipt("x", "1.0");
        b.launchers = vec!["x".into(), "x-new".into()];
        match plan_upgrade(&a, &b) {
            UpgradeDecision::LauncherDelta { added, removed } => {
                assert_eq!(added, vec!["x-new".to_string()]);
                assert_eq!(removed, vec!["x-old".to_string()]);
            }
            other => panic!("expected LauncherDelta, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_reinstall_takes_precedence_over_launcher_delta() {
        let mut a = receipt("x", "1.0");
        a.launchers = vec!["x".into()];
        let mut b = receipt("x", "1.1"); // version diff -> Reinstall
        b.launchers = vec!["x".into(), "x-extra".into()];
        match plan_upgrade(&a, &b) {
            UpgradeDecision::Reinstall { .. } => {}
            other => panic!("expected Reinstall, got {other:?}"),
        }
    }

    #[test]
    fn launcher_removal_spares_names_claimed_by_other_tools() {
        let mut target = receipt("httpie", "3.0");
        target.launchers = vec!["http".into(), "https".into(), "httpie".into()];
        let mut other = receipt("rival", "1.0");
        other.launchers = vec!["https".into()]; // collides intentionally

        let safe = plan_launcher_removal(&target, &[other]);
        // `https` is owned by both — must NOT be deleted.
        assert!(safe.contains(&"http".to_string()));
        assert!(safe.contains(&"httpie".to_string()));
        assert!(!safe.contains(&"https".to_string()));
    }

    #[test]
    fn launcher_removal_empty_when_target_has_no_launchers() {
        let target = receipt("x", "1");
        let safe = plan_launcher_removal(&target, &[]);
        assert!(safe.is_empty());
    }

    #[test]
    fn default_tool_root_uses_xdg_data_home_when_set() {
        let prev_xdg = std::env::var("XDG_DATA_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/x/data");
        }
        let root = default_tool_root();
        assert_eq!(root, PathBuf::from("/x/data/uv/tools"));
        // Restore.
        match prev_xdg {
            Some(v) => unsafe { std::env::set_var("XDG_DATA_HOME", v) },
            None => unsafe { std::env::remove_var("XDG_DATA_HOME") },
        }
        if let Some(v) = prev_home {
            unsafe { std::env::set_var("HOME", v) };
        }
    }

    #[test]
    fn toml_quote_escapes_special_chars() {
        assert_eq!(toml_quote("a\"b"), r#""a\"b""#);
        assert_eq!(toml_quote("a\\b"), r#""a\\b""#);
        assert_eq!(toml_quote("a\nb"), r#""a\nb""#);
        assert_eq!(toml_quote("hello"), r#""hello""#);
    }

    #[test]
    fn render_omits_optional_requires_python_when_none() {
        let r = receipt("x", "1.0");
        let body = render_receipt_toml(&r);
        assert!(!body.contains("requires_python"));
    }

    #[test]
    fn render_omits_closure_table_when_empty() {
        let r = receipt("x", "1.0");
        let body = render_receipt_toml(&r);
        assert!(!body.contains("[closure]"));
    }
}
