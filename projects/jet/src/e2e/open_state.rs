// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Open-mode keyboard shortcuts and local state persistence (#2729).
//!
//! The reusable open-mode review UI exposes a small set of Cypress-like
//! actions (run, pause, next-step, replay, stop) bound to keyboard
//! shortcuts, plus a per-session local-state record that captures
//! the reviewer's last selected case and panel layout. Both surfaces
//! live in this module so the desktop shell and the in-browser
//! component agree on the contract.
//!
//! Shortcuts are documented in the dev-facing help (not noisy UI
//! copy) so this module owns the canonical map. Persistence
//! round-trips through JSON; the caller chooses where to write
//! (e.g. `~/.jet/open-state.json`).
//!
//! Remote/collaborative review is out of scope.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Stable schema tag for [`OpenLocalState`].
pub const OPEN_STATE_SCHEMA_VERSION: &str = "jet.e2e.open-state.v1";

/// Reviewer-facing actions bound to keyboard shortcuts. Mirrors
/// the controls already exposed by the live step API; this enum
/// is the canonical reference for help text.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShortcutAction {
    Run,
    Pause,
    NextStep,
    ReplayCase,
    Stop,
    FocusSearch,
}

/// One keyboard binding. `keys` is the platform-neutral combo
/// using `Ctrl`/`Shift`/`Alt`/`Meta` modifiers and a single key
/// (e.g. `"Ctrl+Enter"`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub action: ShortcutAction,
    pub keys: String,
    pub description: String,
}

/// Default shortcut map. UI surfaces should consume this through
/// [`default_shortcuts`] so renaming a binding only needs one edit.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn default_shortcuts() -> Vec<ShortcutBinding> {
    vec![
        ShortcutBinding {
            action: ShortcutAction::Run,
            keys: "Ctrl+Enter".into(),
            description: "Run the selected case".into(),
        },
        ShortcutBinding {
            action: ShortcutAction::Pause,
            keys: "Space".into(),
            description: "Pause the current run".into(),
        },
        ShortcutBinding {
            action: ShortcutAction::NextStep,
            keys: "ArrowRight".into(),
            description: "Advance to next step".into(),
        },
        ShortcutBinding {
            action: ShortcutAction::ReplayCase,
            keys: "R".into(),
            description: "Replay the selected case from the first step".into(),
        },
        ShortcutBinding {
            action: ShortcutAction::Stop,
            keys: "Escape".into(),
            description: "Stop the current run".into(),
        },
        ShortcutBinding {
            action: ShortcutAction::FocusSearch,
            keys: "/".into(),
            description: "Focus the case explorer search box".into(),
        },
    ]
}

/// Resolve a shortcut binding from the default map.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn lookup_shortcut(action: ShortcutAction) -> Option<ShortcutBinding> {
    default_shortcuts().into_iter().find(|b| b.action == action)
}

/// Persistent layout selection. `panel_layout` is an opaque token
/// (e.g. "wide" / "stacked") chosen by the UI; this module does not
/// enumerate possible layouts so new layouts roll out without a
/// schema bump.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelLayout {
    pub name: String,
    /// Whether the inspector panel is visible.
    pub inspector_open: bool,
    /// Whether the explorer panel is visible.
    pub explorer_open: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for PanelLayout {
    fn default() -> Self {
        Self {
            name: "wide".into(),
            inspector_open: true,
            explorer_open: true,
        }
    }
}

/// Per-session local state restored on next launch.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenLocalState {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_case_id: Option<String>,
    pub panel_layout: PanelLayout,
    /// Text filter persisted from the case explorer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explorer_text_filter: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for OpenLocalState {
    fn default() -> Self {
        Self {
            schema_version: OPEN_STATE_SCHEMA_VERSION.to_string(),
            selected_case_id: None,
            panel_layout: PanelLayout::default(),
            explorer_text_filter: None,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl OpenLocalState {
    /// Read the state from `path`. Returns [`Self::default`] when
    /// the file does not exist, so first-launch is not an error.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let body =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let state: Self =
            serde_json::from_str(&body).with_context(|| format!("decoding {}", path.display()))?;
        Ok(state)
    }

    /// Write the state at `path`. Creates parent directories.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_vec_pretty(self).context("serialising open state")?;
        std::fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shortcuts_cover_every_action() {
        let map = default_shortcuts();
        let actions: Vec<ShortcutAction> = map.iter().map(|b| b.action).collect();
        for a in [
            ShortcutAction::Run,
            ShortcutAction::Pause,
            ShortcutAction::NextStep,
            ShortcutAction::ReplayCase,
            ShortcutAction::Stop,
            ShortcutAction::FocusSearch,
        ] {
            assert!(actions.contains(&a), "missing action: {a:?}");
        }
    }

    #[test]
    fn lookup_returns_default_binding() {
        let b = lookup_shortcut(ShortcutAction::Run).unwrap();
        assert_eq!(b.keys, "Ctrl+Enter");
        assert!(b.description.contains("Run"));
    }

    #[test]
    fn state_round_trips_through_disk() {
        // Stop condition (#2729): selected case + panel restore after
        // reload in local mode.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("open-state.json");
        let mut state = OpenLocalState::default();
        state.selected_case_id = Some("flows/buy.case.ts::buyer".into());
        state.panel_layout = PanelLayout {
            name: "stacked".into(),
            inspector_open: true,
            explorer_open: false,
        };
        state.explorer_text_filter = Some("buyer".into());
        state.save(&path).unwrap();

        let back = OpenLocalState::load(&path).unwrap();
        assert_eq!(back, state);
        assert_eq!(
            back.selected_case_id.as_deref(),
            Some("flows/buy.case.ts::buyer"),
        );
        assert_eq!(back.panel_layout.name, "stacked");
        assert!(!back.panel_layout.explorer_open);
    }

    #[test]
    fn missing_state_file_returns_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("absent.json");
        let state = OpenLocalState::load(&path).unwrap();
        assert_eq!(state, OpenLocalState::default());
    }

    #[test]
    fn shortcut_binding_serialises_kebab_case_actions() {
        let b = ShortcutBinding {
            action: ShortcutAction::ReplayCase,
            keys: "R".into(),
            description: "Replay".into(),
        };
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.contains("\"action\":\"replay-case\""), "{json}");
        assert!(json.contains("\"keys\":\"R\""), "{json}");
    }

    #[test]
    fn unset_filter_skip_serialises() {
        let state = OpenLocalState::default();
        let json = serde_json::to_string(&state).unwrap();
        assert!(!json.contains("\"selected_case_id\""), "{json}");
        assert!(!json.contains("\"explorer_text_filter\""), "{json}");
    }
}
// CODEGEN-END
