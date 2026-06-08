// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Actionability checks for E2E selectors (#2877).
//!
//! Before driving a click/type/select against a resolved selector,
//! the runner verifies three actionability conditions:
//!
//! 1. **Visibility** — the element renders with a non-zero box and is
//!    not display:none / visibility:hidden.
//! 2. **Enabled** — the element does not carry `disabled` /
//!    `aria-disabled="true"` / `readonly` for interactive controls.
//! 3. **Hit-target available** — the point under the action is the
//!    element itself, not something covering it.
//!
//! Each failing check reports a distinct [`ActionabilityReason`] so
//! the runner can pin the failure to the current product step.
//! Retry/layout-stability are out of scope here (split into #2878).

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`ActionabilityResult`].
pub const ACTIONABILITY_SCHEMA_VERSION: &str = "jet.e2e.actionability.v1";

/// Why an actionability check failed. Each variant maps to one of the
/// three required conditions plus a catch-all for unexpected DOM
/// state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum ActionabilityReason {
    NotVisible { detail: String },
    NotEnabled { detail: String },
    CoveredByOtherElement { covering_selector: String },
    UnexpectedDomState { detail: String },
}

/// DOM probe the runner samples from the controlled browser before
/// the action. Filled in via the CDP driver — this module is the
/// policy layer.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementProbe {
    pub width_px: f64,
    pub height_px: f64,
    pub display_none: bool,
    pub visibility_hidden: bool,
    pub opacity_zero: bool,
    pub disabled_attr: bool,
    pub aria_disabled: bool,
    pub readonly_attr: bool,
    pub is_interactive: bool,
    /// `Some(selector)` when a different element occupies the hit
    /// target; `None` when the probe target wins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub covering_selector: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for ElementProbe {
    fn default() -> Self {
        Self {
            width_px: 100.0,
            height_px: 30.0,
            display_none: false,
            visibility_hidden: false,
            opacity_zero: false,
            disabled_attr: false,
            aria_disabled: false,
            readonly_attr: false,
            is_interactive: true,
            covering_selector: None,
        }
    }
}

/// Outcome of the three checks against one probe.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionabilityResult {
    Actionable,
    Blocked {
        schema_version: String,
        reason: ActionabilityReason,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ActionabilityResult {
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Actionable)
    }
}

/// Run the three checks in order: visibility, enabled, hit target.
/// Returns the *first* failing reason so the step inspector pins one
/// root cause; chained failures should re-check after a fix.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn check_actionability(probe: &ElementProbe) -> ActionabilityResult {
    if let Some(reason) = check_visibility(probe) {
        return ActionabilityResult::Blocked {
            schema_version: ACTIONABILITY_SCHEMA_VERSION.to_string(),
            reason,
        };
    }
    if let Some(reason) = check_enabled(probe) {
        return ActionabilityResult::Blocked {
            schema_version: ACTIONABILITY_SCHEMA_VERSION.to_string(),
            reason,
        };
    }
    if let Some(reason) = check_hit_target(probe) {
        return ActionabilityResult::Blocked {
            schema_version: ACTIONABILITY_SCHEMA_VERSION.to_string(),
            reason,
        };
    }
    ActionabilityResult::Actionable
}

fn check_visibility(probe: &ElementProbe) -> Option<ActionabilityReason> {
    if probe.display_none {
        return Some(ActionabilityReason::NotVisible {
            detail: "computed display is 'none'".into(),
        });
    }
    if probe.visibility_hidden {
        return Some(ActionabilityReason::NotVisible {
            detail: "computed visibility is 'hidden'".into(),
        });
    }
    if probe.opacity_zero {
        return Some(ActionabilityReason::NotVisible {
            detail: "computed opacity is 0".into(),
        });
    }
    if probe.width_px <= 0.0 || probe.height_px <= 0.0 {
        return Some(ActionabilityReason::NotVisible {
            detail: format!(
                "element box is {}x{}",
                probe.width_px as i64, probe.height_px as i64
            ),
        });
    }
    None
}

fn check_enabled(probe: &ElementProbe) -> Option<ActionabilityReason> {
    if !probe.is_interactive {
        return None;
    }
    if probe.disabled_attr {
        return Some(ActionabilityReason::NotEnabled {
            detail: "element carries `disabled` attribute".into(),
        });
    }
    if probe.aria_disabled {
        return Some(ActionabilityReason::NotEnabled {
            detail: "element carries `aria-disabled=\"true\"`".into(),
        });
    }
    if probe.readonly_attr {
        return Some(ActionabilityReason::NotEnabled {
            detail: "element carries `readonly` attribute".into(),
        });
    }
    None
}

fn check_hit_target(probe: &ElementProbe) -> Option<ActionabilityReason> {
    probe
        .covering_selector
        .as_ref()
        .map(|sel| ActionabilityReason::CoveredByOtherElement {
            covering_selector: sel.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok() -> ElementProbe {
        ElementProbe::default()
    }

    #[test]
    fn fully_visible_enabled_uncovered_element_is_actionable() {
        let result = check_actionability(&ok());
        assert!(matches!(result, ActionabilityResult::Actionable));
        assert!(result.is_actionable());
    }

    #[test]
    fn hidden_element_reports_not_visible() {
        // Stop condition (#2877): hidden element produces a distinct
        // failure reason.
        let mut probe = ok();
        probe.display_none = true;
        let result = check_actionability(&probe);
        let ActionabilityResult::Blocked { reason, .. } = result else {
            panic!("expected blocked");
        };
        assert!(matches!(
            reason,
            ActionabilityReason::NotVisible { ref detail }
                if detail.contains("display")
        ));
    }

    #[test]
    fn zero_box_reports_not_visible() {
        let mut probe = ok();
        probe.width_px = 0.0;
        let result = check_actionability(&probe);
        assert!(matches!(
            result,
            ActionabilityResult::Blocked {
                reason: ActionabilityReason::NotVisible { .. },
                ..
            }
        ));
    }

    #[test]
    fn disabled_element_reports_not_enabled() {
        // Stop condition (#2877): disabled element produces a
        // distinct failure reason.
        let mut probe = ok();
        probe.disabled_attr = true;
        let result = check_actionability(&probe);
        let ActionabilityResult::Blocked { reason, .. } = result else {
            panic!("expected blocked");
        };
        assert!(matches!(
            reason,
            ActionabilityReason::NotEnabled { ref detail }
                if detail.contains("disabled")
        ));
    }

    #[test]
    fn aria_disabled_reports_not_enabled() {
        let mut probe = ok();
        probe.aria_disabled = true;
        let result = check_actionability(&probe);
        assert!(matches!(
            result,
            ActionabilityResult::Blocked {
                reason: ActionabilityReason::NotEnabled { .. },
                ..
            }
        ));
    }

    #[test]
    fn covered_element_reports_hit_target_failure() {
        // Stop condition (#2877): covered element produces a distinct
        // failure reason.
        let mut probe = ok();
        probe.covering_selector = Some("div.modal-overlay".into());
        let result = check_actionability(&probe);
        let ActionabilityResult::Blocked { reason, .. } = result else {
            panic!("expected blocked");
        };
        assert_eq!(
            reason,
            ActionabilityReason::CoveredByOtherElement {
                covering_selector: "div.modal-overlay".into()
            }
        );
    }

    #[test]
    fn non_interactive_element_skips_enabled_check() {
        let mut probe = ok();
        probe.is_interactive = false;
        probe.disabled_attr = true;
        let result = check_actionability(&probe);
        assert!(result.is_actionable());
    }

    #[test]
    fn visibility_failure_short_circuits_before_enabled_check() {
        // Pin one root cause per failure so the inspector doesn't
        // chain unrelated reasons.
        let mut probe = ok();
        probe.display_none = true;
        probe.disabled_attr = true;
        let result = check_actionability(&probe);
        assert!(matches!(
            result,
            ActionabilityResult::Blocked {
                reason: ActionabilityReason::NotVisible { .. },
                ..
            }
        ));
    }

    #[test]
    fn result_round_trips_through_json() {
        let mut probe = ok();
        probe.covering_selector = Some("a".into());
        let result = check_actionability(&probe);
        let json = serde_json::to_string(&result).unwrap();
        let back: ActionabilityResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
        assert!(json.contains("covered_by_other_element"), "{json}");
    }
}
// CODEGEN-END
