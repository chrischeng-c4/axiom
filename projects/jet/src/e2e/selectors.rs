// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Selector resolution contract and step-evidence shape (#2724).
//!
//! Product-flow E2E steps interact with the AUT through a small set
//! of selector kinds (CSS, role, text, test-id, XPath). When a step
//! runs, the controlled browser resolves the selector and reports
//! one of two outcomes: a `Hit` carrying the matched element handle,
//! or a `Miss` carrying the reason the resolver could not match.
//! This module defines the on-disk shape both outcomes serialise
//! into so step evidence is uniform across hit and miss.
//!
//! Actionability (visibility, enabled-state, hit-target, retry loop)
//! is intentionally out of scope here — that work lives in #2877
//! and #2878. This slice is the bare resolution contract plus the
//! evidence record that wraps it.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`SelectorEvidence`].
pub const SELECTOR_EVIDENCE_SCHEMA_VERSION: &str = "jet.e2e.selector.v1";

/// Supported selector kinds. Mode-specific runners map their own
/// short-hands onto this lexicon so evidence stays vocabulary-stable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectorKind {
    Css,
    Role,
    Text,
    TestId,
    Xpath,
}

/// Author-facing selector descriptor. `kind` + `query` together
/// uniquely identify the resolver request; `description` is an
/// optional human-readable label for review-mode rendering.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    pub kind: SelectorKind,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Selector {
    pub fn css(query: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Css,
            query: query.into(),
            description: None,
        }
    }

    pub fn role(query: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Role,
            query: query.into(),
            description: None,
        }
    }

    pub fn text(query: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Text,
            query: query.into(),
            description: None,
        }
    }

    pub fn test_id(query: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::TestId,
            query: query.into(),
            description: None,
        }
    }

    /// Builder for cases that want to attach a human-readable label.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Reason a selector did not resolve. Mirrors the failure lexicon
/// used by browser drivers so evidence and exit-code mapping stay
/// aligned. Actionability-related misses (e.g. element matched but
/// invisible) are routed through #2877 and are NOT modelled here.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectorMissReason {
    /// No element in the DOM matched the selector.
    NoMatch,
    /// More than one element matched; the resolver requires a
    /// single-element selector for this step.
    MultipleMatches,
    /// Selector kind not supported by the active driver (e.g. xpath
    /// disabled in a CDP-only driver).
    Unsupported,
    /// Selector resolution timed out before the DOM settled.
    Timeout,
}

/// Outcome of one selector resolution. `Hit` carries the opaque
/// element handle reported by the driver so a follow-up action
/// step can reuse it without re-resolving. `Miss` carries the
/// reason + the candidate count observed so failure renderers can
/// say "0 matches" vs "3 matches".
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SelectorResolution {
    Hit {
        /// Driver-specific element handle (e.g. CDP node id rendered
        /// as a string). Opaque to consumers.
        element_handle: String,
        /// Number of matches the resolver observed; always >= 1.
        match_count: u32,
    },
    Miss {
        reason: SelectorMissReason,
        /// Number of matches the resolver observed (0 for `NoMatch`).
        match_count: u32,
        /// Short human-readable explanation lifted from the driver.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl SelectorResolution {
    pub fn hit(element_handle: impl Into<String>) -> Self {
        Self::Hit {
            element_handle: element_handle.into(),
            match_count: 1,
        }
    }

    pub fn miss(reason: SelectorMissReason, match_count: u32) -> Self {
        Self::Miss {
            reason,
            match_count,
            detail: None,
        }
    }

    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        match self {
            Self::Hit { .. } => self,
            Self::Miss {
                reason,
                match_count,
                ..
            } => Self::Miss {
                reason,
                match_count,
                detail: Some(detail.into()),
            },
        }
    }

    pub fn is_hit(&self) -> bool {
        matches!(self, Self::Hit { .. })
    }
}

/// Per-step selector evidence record. One record is emitted for
/// every selector resolution the step performs (including the
/// preflight resolve before an action). Records are appended into
/// the step's evidence stream in resolution order.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectorEvidence {
    pub schema_version: String,
    pub selector: Selector,
    pub resolution: SelectorResolution,
    pub duration_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl SelectorEvidence {
    pub fn new(selector: Selector, resolution: SelectorResolution, duration_ms: u64) -> Self {
        Self {
            schema_version: SELECTOR_EVIDENCE_SCHEMA_VERSION.to_string(),
            selector,
            resolution,
            duration_ms,
        }
    }

    pub fn is_hit(&self) -> bool {
        self.resolution.is_hit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_resolution_serialises_with_element_handle() {
        // Stop condition (#2724): one selector hit produces step evidence.
        let ev = SelectorEvidence::new(
            Selector::test_id("submit").with_description("submit-order button"),
            SelectorResolution::hit("node-42"),
            7,
        );
        assert!(ev.is_hit());
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"kind\":\"test-id\""), "{json}");
        assert!(json.contains("\"query\":\"submit\""), "{json}");
        assert!(json.contains("\"outcome\":\"hit\""), "{json}");
        assert!(json.contains("\"element_handle\":\"node-42\""), "{json}");
        let back: SelectorEvidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
    }

    #[test]
    fn miss_resolution_carries_reason_and_match_count() {
        // Stop condition (#2724): one selector miss produces step evidence.
        let ev = SelectorEvidence::new(
            Selector::css("button.checkout"),
            SelectorResolution::miss(SelectorMissReason::NoMatch, 0)
                .with_detail("0 elements matched .button.checkout"),
            120,
        );
        assert!(!ev.is_hit());
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"outcome\":\"miss\""), "{json}");
        assert!(json.contains("\"reason\":\"no_match\""), "{json}");
        assert!(json.contains("\"match_count\":0"), "{json}");
        assert!(json.contains("0 elements matched"), "{json}");
    }

    #[test]
    fn multiple_matches_miss_records_the_observed_count() {
        let ev = SelectorEvidence::new(
            Selector::text("Submit"),
            SelectorResolution::miss(SelectorMissReason::MultipleMatches, 3),
            5,
        );
        if let SelectorResolution::Miss {
            reason,
            match_count,
            ..
        } = &ev.resolution
        {
            assert_eq!(*reason, SelectorMissReason::MultipleMatches);
            assert_eq!(*match_count, 3);
        } else {
            panic!("expected miss");
        }
    }

    #[test]
    fn selector_description_skip_serialises_when_absent() {
        let ev = SelectorEvidence::new(Selector::css("body"), SelectorResolution::hit("node-1"), 1);
        let json = serde_json::to_string(&ev).unwrap();
        assert!(!json.contains("\"description\""), "{json}");
    }

    #[test]
    fn detail_skip_serialises_when_absent() {
        let ev = SelectorEvidence::new(
            Selector::css("missing"),
            SelectorResolution::miss(SelectorMissReason::Unsupported, 0),
            0,
        );
        let json = serde_json::to_string(&ev).unwrap();
        assert!(!json.contains("\"detail\""), "{json}");
    }

    #[test]
    fn timeout_miss_is_distinct_from_no_match() {
        // Failure renderers need to distinguish "DOM never settled"
        // from "selector resolved to nothing" — both encode as Miss
        // but carry different reasons.
        let timeout = SelectorResolution::miss(SelectorMissReason::Timeout, 0);
        let no_match = SelectorResolution::miss(SelectorMissReason::NoMatch, 0);
        let timeout_json = serde_json::to_string(&timeout).unwrap();
        let no_match_json = serde_json::to_string(&no_match).unwrap();
        assert_ne!(timeout_json, no_match_json);
        assert!(timeout_json.contains("\"reason\":\"timeout\""));
        assert!(no_match_json.contains("\"reason\":\"no_match\""));
    }
}
// CODEGEN-END
