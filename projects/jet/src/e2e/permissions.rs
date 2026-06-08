// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Per-case browser permission control primitive (#2881).
//!
//! Some product flows ride on a browser permission grant: a "share my
//! location" tour, a paste-from-clipboard shortcut, a push-notification
//! opt-in. Without a deterministic permission state the case races
//! against the browser's permission UI, which mutates per profile.
//! This module owns the permission config policy and the evidence
//! record. The actual CDP `Browser.grantPermissions` call lives in
//! the driver; this layer is the policy + audit trail.
//!
//! Full cross-browser permission compatibility is out of scope.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`PermissionEvidence`].
pub const PERMISSION_SCHEMA_VERSION: &str = "jet.e2e.permission.v1";

/// One permission the case might depend on. Names match Chromium's
/// CDP `Browser.PermissionType` so the driver layer can map directly.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserPermission {
    Geolocation,
    Notifications,
    ClipboardReadWrite,
    Camera,
    Microphone,
    MidiSysex,
    BackgroundSync,
}

/// Configured permission state. `Grant` and `Deny` short-circuit the
/// browser prompt; `Prompt` leaves the default behaviour so the case
/// can exercise the consent UI itself.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionState {
    Grant,
    Deny,
    Prompt,
}

/// A single permission ↔ state pair the case asks for.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub permission: BrowserPermission,
    pub state: PermissionState,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl PermissionGrant {
    pub fn grant(permission: BrowserPermission) -> Self {
        Self {
            permission,
            state: PermissionState::Grant,
        }
    }

    pub fn deny(permission: BrowserPermission) -> Self {
        Self {
            permission,
            state: PermissionState::Deny,
        }
    }
}

/// Per-case permission policy. Scoped to a single origin so two cases
/// targeting different sandboxes don't bleed permissions across.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub origin: String,
    pub grants: Vec<PermissionGrant>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl PermissionPolicy {
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            grants: Vec::new(),
        }
    }

    pub fn with(mut self, grant: PermissionGrant) -> Self {
        self.grants.push(grant);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// Look up the configured state for one permission. Returns
    /// [`PermissionState::Prompt`] when the policy doesn't mention it
    /// — equivalent to "let the browser decide".
    pub fn state_for(&self, permission: BrowserPermission) -> PermissionState {
        self.grants
            .iter()
            .find(|g| g.permission == permission)
            .map(|g| g.state)
            .unwrap_or(PermissionState::Prompt)
    }
}

/// Outcome of asking the controlled permission for a value. Used by
/// the fixture to prove the case behaves deterministically — a granted
/// permission returns `Allowed`, a denied permission returns `Denied`
/// with the policy reason, never a prompt that hangs the test.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum PermissionQueryOutcome {
    Allowed,
    Denied { reason: String },
    Prompted,
}

/// Evidence row the runner attaches to the case. Captures both the
/// requested policy and the queries the fixture made, so a reviewer
/// can confirm the case really did see the configured permission
/// state (not just that the policy was set on the side).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionEvidence {
    pub schema_version: String,
    pub policy: PermissionPolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<PermissionQueryRecord>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionQueryRecord {
    pub permission: BrowserPermission,
    pub outcome: PermissionQueryOutcome,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl PermissionEvidence {
    pub fn from_policy(policy: PermissionPolicy) -> Self {
        Self {
            schema_version: PERMISSION_SCHEMA_VERSION.to_string(),
            policy,
            queries: Vec::new(),
        }
    }

    /// Apply the policy to one permission ask. The fixture calls this
    /// once per `navigator.permissions.query`-style request.
    pub fn observe(&mut self, permission: BrowserPermission) -> PermissionQueryOutcome {
        let outcome = match self.policy.state_for(permission) {
            PermissionState::Grant => PermissionQueryOutcome::Allowed,
            PermissionState::Deny => PermissionQueryOutcome::Denied {
                reason: format!("denied by case policy for {}", self.policy.origin),
            },
            PermissionState::Prompt => PermissionQueryOutcome::Prompted,
        };
        self.queries.push(PermissionQueryRecord {
            permission,
            outcome: outcome.clone(),
        });
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_with_grant_observes_allowed() {
        // Stop condition (#2881): a case can run with a deterministic
        // permission grant state.
        let policy = PermissionPolicy::new("https://example.test")
            .with(PermissionGrant::grant(BrowserPermission::Geolocation));
        let mut ev = PermissionEvidence::from_policy(policy);
        let outcome = ev.observe(BrowserPermission::Geolocation);
        assert_eq!(outcome, PermissionQueryOutcome::Allowed);
    }

    #[test]
    fn case_with_deny_observes_denied_with_reason() {
        // Stop condition (#2881): deterministic deny state.
        let policy = PermissionPolicy::new("https://example.test")
            .with(PermissionGrant::deny(BrowserPermission::Notifications));
        let mut ev = PermissionEvidence::from_policy(policy);
        match ev.observe(BrowserPermission::Notifications) {
            PermissionQueryOutcome::Denied { reason } => {
                assert!(reason.contains("example.test"), "{reason}");
            }
            other => panic!("expected Denied, got {other:?}"),
        }
    }

    #[test]
    fn unconfigured_permission_defaults_to_prompt() {
        let policy = PermissionPolicy::new("https://example.test");
        let mut ev = PermissionEvidence::from_policy(policy);
        assert_eq!(
            ev.observe(BrowserPermission::Camera),
            PermissionQueryOutcome::Prompted
        );
    }

    #[test]
    fn evidence_captures_configured_permission_state() {
        // Stop condition (#2881): evidence captures the configured
        // permission state.
        let policy = PermissionPolicy::new("https://example.test")
            .with(PermissionGrant::grant(BrowserPermission::Geolocation))
            .with(PermissionGrant::deny(BrowserPermission::Notifications));
        let mut ev = PermissionEvidence::from_policy(policy.clone());
        ev.observe(BrowserPermission::Geolocation);
        ev.observe(BrowserPermission::Notifications);

        assert_eq!(ev.policy, policy);
        assert_eq!(ev.queries.len(), 2);
        assert_eq!(ev.queries[0].outcome, PermissionQueryOutcome::Allowed);
        match &ev.queries[1].outcome {
            PermissionQueryOutcome::Denied { .. } => {}
            other => panic!("expected denied for notifications, got {other:?}"),
        }
    }

    #[test]
    fn permission_dependent_fixture_behaves_deterministically_across_runs() {
        // Stop condition (#2881): a permission-dependent fixture
        // behaves deterministically — same policy twice = same record.
        let policy = PermissionPolicy::new("https://example.test").with(PermissionGrant::grant(
            BrowserPermission::ClipboardReadWrite,
        ));

        let run_once = || {
            let mut ev = PermissionEvidence::from_policy(policy.clone());
            for _ in 0..3 {
                ev.observe(BrowserPermission::ClipboardReadWrite);
            }
            ev
        };
        let a = run_once();
        let b = run_once();
        assert_eq!(a, b);
        assert!(a
            .queries
            .iter()
            .all(|q| q.outcome == PermissionQueryOutcome::Allowed));
    }

    #[test]
    fn evidence_round_trips_through_json() {
        let policy = PermissionPolicy::new("https://example.test")
            .with(PermissionGrant::grant(BrowserPermission::MidiSysex));
        let mut ev = PermissionEvidence::from_policy(policy);
        ev.observe(BrowserPermission::MidiSysex);
        let json = serde_json::to_string(&ev).unwrap();
        let back: PermissionEvidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
        assert!(json.contains("\"permission\":\"midi-sysex\""), "{json}");
        assert!(json.contains("\"state\":\"grant\""), "{json}");
        assert!(json.contains("\"outcome\":\"allowed\""), "{json}");
    }
}
// CODEGEN-END
