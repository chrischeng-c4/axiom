// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Per-case browser storage reset primitive (#2880).
//!
//! Two e2e cases that share a browser context can leak state through
//! `localStorage` / `sessionStorage` / cookies: case A logs in, case B
//! inherits the session and silently skips the login flow. This
//! module owns the reset policy and the evidence record. The actual
//! CDP call that runs `localStorage.clear()` etc. lives in the
//! driver; this layer is the policy + payload + audit trail.
//!
//! Database / backend fixture orchestration is out of scope.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`StorageResetEvidence`].
pub const STORAGE_RESET_SCHEMA_VERSION: &str = "jet.e2e.storage-reset.v1";

/// Which storage surfaces the runner should clear before the next
/// case starts. `Cookies` covers document.cookie + the CDP cookie jar
/// for the controlled origin.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageSurface {
    LocalStorage,
    SessionStorage,
    Cookies,
}

/// Reset policy attached to a case. The runner reads this between
/// cases and emits one reset event per surface so an inspector can
/// tell whether a leak was prevented or never possible.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageResetPolicy {
    pub surfaces: Vec<StorageSurface>,
    /// When true, also clear IndexedDB / Cache Storage via CDP's
    /// `Storage.clearDataForOrigin`. Off by default — the smoke
    /// fixture only needs the three core surfaces.
    #[serde(default)]
    pub include_origin_data: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StorageResetPolicy {
    /// Reset everything the smoke fixture exercises: local + session
    /// + cookies, no origin-data wipe.
    pub fn default_between_cases() -> Self {
        Self {
            surfaces: vec![
                StorageSurface::LocalStorage,
                StorageSurface::SessionStorage,
                StorageSurface::Cookies,
            ],
            include_origin_data: false,
        }
    }

    /// Skip the reset entirely — the next case wants whatever state
    /// the previous case left behind (e.g. session-reuse perf bench).
    pub fn none() -> Self {
        Self {
            surfaces: Vec::new(),
            include_origin_data: false,
        }
    }

    pub fn is_noop(&self) -> bool {
        self.surfaces.is_empty() && !self.include_origin_data
    }

    /// JS payload the CDP driver runs against the controlled page. We
    /// emit one statement per surface so the inspector can see which
    /// surfaces were touched directly in the call log. Returns `None`
    /// when the policy is a no-op.
    pub fn to_browser_script(&self) -> Option<String> {
        if self.is_noop() {
            return None;
        }
        let mut stmts: Vec<&'static str> = Vec::new();
        for s in &self.surfaces {
            match s {
                StorageSurface::LocalStorage => {
                    stmts.push("try{localStorage.clear();}catch(e){}");
                }
                StorageSurface::SessionStorage => {
                    stmts.push("try{sessionStorage.clear();}catch(e){}");
                }
                StorageSurface::Cookies => {
                    // Clear all cookies on the document by setting
                    // them to expire in the past. CDP-level cookie
                    // jar reset is the driver's job; this is the
                    // best-effort document-side complement.
                    stmts.push(
                        "try{document.cookie.split(';').forEach(c=>{\
                         const eq=c.indexOf('=');\
                         const n=eq>-1?c.substring(0,eq).trim():c.trim();\
                         document.cookie=n+'=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';\
                         });}catch(e){}",
                    );
                }
            }
        }
        Some(format!("(()=>{{{}}})();", stmts.join("")))
    }
}

/// One reset action the runner actually performed. Recorded into
/// evidence so a reviewer can confirm the leak window between cases
/// was closed.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageResetEvent {
    pub surface: StorageSurface,
    /// Best-effort count of entries cleared, when the driver can
    /// report it. `None` means "cleared, count unknown" (cookies
    /// often land here).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_cleared: Option<u32>,
}

/// Evidence wrapper. Pairs the requested policy with the events that
/// actually ran, so a no-op policy and a no-state browser look
/// different in evidence (the former has an empty events list, the
/// latter has events with `entries_cleared = Some(0)`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageResetEvidence {
    pub schema_version: String,
    pub policy: StorageResetPolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<StorageResetEvent>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StorageResetEvidence {
    pub fn from_policy(policy: StorageResetPolicy) -> Self {
        Self {
            schema_version: STORAGE_RESET_SCHEMA_VERSION.to_string(),
            policy,
            events: Vec::new(),
        }
    }

    pub fn record(&mut self, surface: StorageSurface, entries_cleared: Option<u32>) {
        self.events.push(StorageResetEvent {
            surface,
            entries_cleared,
        });
    }

    /// True when every surface in the policy has a matching event —
    /// the assertion the runner uses to confirm the reset ran fully.
    pub fn covers_policy(&self) -> bool {
        if self.policy.is_noop() {
            return self.events.is_empty();
        }
        self.policy
            .surfaces
            .iter()
            .all(|s| self.events.iter().any(|e| e.surface == *s))
    }
}

/// Tiny in-memory browser-storage stand-in. Used by the fixture test
/// to demonstrate that without a reset between cases, case B sees the
/// state case A wrote — and with a reset, it doesn't.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Default)]
pub struct FakeBrowserStorage {
    pub local: Vec<(String, String)>,
    pub session: Vec<(String, String)>,
    pub cookies: Vec<(String, String)>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl FakeBrowserStorage {
    pub fn set_local(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.local.push((k.into(), v.into()));
    }

    pub fn set_session(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.session.push((k.into(), v.into()));
    }

    pub fn set_cookie(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.cookies.push((k.into(), v.into()));
    }

    /// Apply a reset policy and return the evidence describing what
    /// was cleared. Mirrors the driver shape so the policy can be
    /// covered by a unit test without spinning up a browser.
    pub fn apply_reset(&mut self, policy: &StorageResetPolicy) -> StorageResetEvidence {
        let mut evidence = StorageResetEvidence::from_policy(policy.clone());
        for surface in &policy.surfaces {
            let cleared = match surface {
                StorageSurface::LocalStorage => {
                    let n = self.local.len() as u32;
                    self.local.clear();
                    n
                }
                StorageSurface::SessionStorage => {
                    let n = self.session.len() as u32;
                    self.session.clear();
                    n
                }
                StorageSurface::Cookies => {
                    let n = self.cookies.len() as u32;
                    self.cookies.clear();
                    n
                }
            };
            evidence.record(*surface, Some(cleared));
        }
        evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_covers_three_core_surfaces() {
        let policy = StorageResetPolicy::default_between_cases();
        assert_eq!(policy.surfaces.len(), 3);
        assert!(policy.surfaces.contains(&StorageSurface::LocalStorage));
        assert!(policy.surfaces.contains(&StorageSurface::SessionStorage));
        assert!(policy.surfaces.contains(&StorageSurface::Cookies));
        assert!(!policy.include_origin_data);
    }

    #[test]
    fn none_policy_is_noop() {
        let policy = StorageResetPolicy::none();
        assert!(policy.is_noop());
        assert!(policy.to_browser_script().is_none());
    }

    #[test]
    fn second_case_starts_with_clean_storage() {
        // Stop condition (#2880): two cases do not leak local/session
        // storage state.
        let mut browser = FakeBrowserStorage::default();

        // Case A writes some state, mimicking a login flow.
        browser.set_local("auth.token", "session-abc");
        browser.set_session("flow.step", "review");
        browser.set_cookie("sid", "xyz");

        // Runner resets between cases.
        let policy = StorageResetPolicy::default_between_cases();
        let evidence = browser.apply_reset(&policy);

        // Case B starts — storage must be empty.
        assert!(browser.local.is_empty(), "localStorage leaked across cases");
        assert!(
            browser.session.is_empty(),
            "sessionStorage leaked across cases"
        );
        assert!(browser.cookies.is_empty(), "cookies leaked across cases");

        // Stop condition (#2880): evidence records that storage reset
        // ran.
        assert!(evidence.covers_policy());
        assert_eq!(evidence.events.len(), 3);
    }

    #[test]
    fn evidence_records_per_surface_counts() {
        let mut browser = FakeBrowserStorage::default();
        browser.set_local("a", "1");
        browser.set_local("b", "2");
        browser.set_session("s", "v");

        let evidence = browser.apply_reset(&StorageResetPolicy::default_between_cases());
        let local = evidence
            .events
            .iter()
            .find(|e| e.surface == StorageSurface::LocalStorage)
            .unwrap();
        let session = evidence
            .events
            .iter()
            .find(|e| e.surface == StorageSurface::SessionStorage)
            .unwrap();
        let cookies = evidence
            .events
            .iter()
            .find(|e| e.surface == StorageSurface::Cookies)
            .unwrap();
        assert_eq!(local.entries_cleared, Some(2));
        assert_eq!(session.entries_cleared, Some(1));
        assert_eq!(cookies.entries_cleared, Some(0));
    }

    #[test]
    fn noop_policy_records_no_events_and_emits_no_script() {
        let mut browser = FakeBrowserStorage::default();
        browser.set_local("a", "1");
        let evidence = browser.apply_reset(&StorageResetPolicy::none());
        assert!(evidence.events.is_empty());
        assert!(evidence.covers_policy());
        assert_eq!(browser.local.len(), 1, "noop policy must not clear state");
    }

    #[test]
    fn browser_script_includes_targeted_surfaces_only() {
        let policy = StorageResetPolicy {
            surfaces: vec![StorageSurface::LocalStorage],
            include_origin_data: false,
        };
        let script = policy.to_browser_script().unwrap();
        assert!(script.contains("localStorage.clear"), "{script}");
        assert!(!script.contains("sessionStorage.clear"), "{script}");
    }

    #[test]
    fn covers_policy_fails_when_an_event_is_missing() {
        let policy = StorageResetPolicy::default_between_cases();
        let mut evidence = StorageResetEvidence::from_policy(policy);
        evidence.record(StorageSurface::LocalStorage, Some(0));
        evidence.record(StorageSurface::SessionStorage, Some(0));
        assert!(!evidence.covers_policy(), "missing cookies surface");
    }

    #[test]
    fn evidence_round_trips_through_json() {
        let mut browser = FakeBrowserStorage::default();
        browser.set_local("k", "v");
        let evidence = browser.apply_reset(&StorageResetPolicy::default_between_cases());
        let json = serde_json::to_string(&evidence).unwrap();
        let back: StorageResetEvidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, evidence);
        assert!(json.contains("\"surface\":\"local-storage\""), "{json}");
    }
}
// CODEGEN-END
