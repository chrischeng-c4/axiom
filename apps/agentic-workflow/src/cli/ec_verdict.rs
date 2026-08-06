//! Target verification profiles and pure target verdict evaluation (#3349).

use crate::cli::change_lifecycle::{ArtifactKind, ChangeLifecycle};

/// Known EC verification dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EcDimension {
    Behavior,
    Efficiency,
    Security,
    Stability,
}

impl EcDimension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Behavior => "behavior",
            Self::Efficiency => "efficiency",
            Self::Security => "security",
            Self::Stability => "stability",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "behavior" => Some(Self::Behavior),
            "efficiency" => Some(Self::Efficiency),
            "security" => Some(Self::Security),
            "stability" => Some(Self::Stability),
            _ => None,
        }
    }
}

/// Target verification stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerificationTarget {
    Td,
    Cb,
}

impl VerificationTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Td => "td",
            Self::Cb => "cb",
        }
    }

    pub fn required_dimensions(self) -> &'static [&'static str] {
        match self {
            Self::Td => &["behavior", "security"],
            Self::Cb => &["behavior", "efficiency", "security", "stability"],
        }
    }
}

/// Pure target verdict evaluation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetVerdict {
    pub target: VerificationTarget,
    pub green: bool,
    pub reason: Option<String>,
    pub required_dimensions: Vec<String>,
    pub green_dimensions: Vec<String>,
    pub failing_dimensions: Vec<String>,
    pub missing_dimensions: Vec<String>,
    pub unpermitted_dimensions: Vec<String>,
    pub stale_dimensions: Vec<String>,
}

impl TargetVerdict {
    pub fn is_green(&self) -> bool {
        self.green
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Evaluate a target verdict as a pure function of the committed lifecycle
/// and target verification profile requirements.
pub fn decide_target_verdict(
    lifecycle: &ChangeLifecycle,
    target: VerificationTarget,
) -> TargetVerdict {
    let req_dims_strs = target.required_dimensions();
    let required_dimensions: Vec<String> = req_dims_strs.iter().map(|s| s.to_string()).collect();

    let active_ec = lifecycle
        .active_revisions
        .get(&ArtifactKind::Ec)
        .and_then(|r| r.as_ref())
        .map(|r| r.digest.clone());
    let active_td = lifecycle
        .active_revisions
        .get(&ArtifactKind::Td)
        .and_then(|r| r.as_ref())
        .map(|r| r.digest.clone());
    let active_cb = lifecycle
        .active_revisions
        .get(&ArtifactKind::Cb)
        .and_then(|r| r.as_ref())
        .map(|r| r.digest.clone());

    let mut green_dimensions = Vec::new();
    let mut failing_dimensions = Vec::new();
    let mut unpermitted_dimensions = Vec::new();
    let mut stale_dimensions = Vec::new();

    let mut active_green_set = std::collections::BTreeSet::new();
    let mut active_fail_set = std::collections::BTreeSet::new();
    let mut stale_set = std::collections::BTreeSet::new();
    let mut unpermitted_set = std::collections::BTreeSet::new();

    for binding in &lifecycle.evidence {
        if let Some(dim) = EcDimension::from_str(&binding.verifier) {
            let dim_str = dim.as_str();

            if req_dims_strs.contains(&dim_str) {
                let ec_ok = active_ec.is_some() && binding.bound_tuple.ec_digest == active_ec;
                let td_ok = active_td.is_some() && binding.bound_tuple.td_digest == active_td;
                let cb_ok = active_cb.is_some() && binding.bound_tuple.cb_digest == active_cb;

                let is_active = match target {
                    VerificationTarget::Td => ec_ok && td_ok,
                    VerificationTarget::Cb => ec_ok && td_ok && cb_ok,
                };

                if is_active {
                    if binding.passed {
                        active_green_set.insert(dim_str);
                    } else {
                        active_fail_set.insert(dim_str);
                    }
                } else {
                    stale_set.insert(dim_str);
                }
            } else {
                unpermitted_set.insert(dim_str);
            }
        }
    }

    let mut missing_dimensions = Vec::new();
    for dim_str in req_dims_strs {
        if active_fail_set.contains(dim_str) {
            failing_dimensions.push(dim_str.to_string());
        } else if active_green_set.contains(dim_str) {
            green_dimensions.push(dim_str.to_string());
        } else if stale_set.contains(dim_str) {
            stale_dimensions.push(dim_str.to_string());
        } else {
            missing_dimensions.push(dim_str.to_string());
        }
    }

    for dim_str in &unpermitted_set {
        unpermitted_dimensions.push(dim_str.to_string());
    }

    let is_green = failing_dimensions.is_empty()
        && missing_dimensions.is_empty()
        && unpermitted_dimensions.is_empty()
        && stale_dimensions.is_empty();

    let reason = if is_green {
        None
    } else {
        let mut parts = Vec::new();

        if !stale_dimensions.is_empty() {
            parts.push(format!(
                "evidence for dimension(s) {} is bound to a revision that is no longer active",
                stale_dimensions.join(", ")
            ));
        }
        if !unpermitted_dimensions.is_empty() {
            for dim in &unpermitted_dimensions {
                parts.push(format!(
                    "dimension '{dim}' is not a {} dimension",
                    target.as_str()
                ));
            }
        }
        if !failing_dimensions.is_empty() {
            parts.push(format!(
                "failing dimension(s): {}",
                failing_dimensions.join(", ")
            ));
        }
        if !missing_dimensions.is_empty() {
            parts.push(format!(
                "missing required dimension(s): {}",
                missing_dimensions.join(", ")
            ));
        }

        Some(parts.join("; "))
    };

    TargetVerdict {
        target,
        green: is_green,
        reason,
        required_dimensions,
        green_dimensions,
        failing_dimensions,
        missing_dimensions,
        unpermitted_dimensions,
        stale_dimensions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::change_lifecycle::{
        ActiveDigestTuple, ArtifactKind, ArtifactRevision, EvidenceBinding,
    };
    use std::collections::BTreeMap;

    fn make_revision(kind: ArtifactKind, id: &str, digest: &str) -> ArtifactRevision {
        ArtifactRevision {
            id: id.to_string(),
            kind,
            digest: digest.to_string(),
            parents: Vec::new(),
            iteration: 1,
            superseded_by: None,
            invalidation_reason: None,
        }
    }

    fn make_lifecycle(
        wi: Option<(&str, &str)>,
        ec: Option<(&str, &str)>,
        td: Option<(&str, &str)>,
        cb: Option<(&str, &str)>,
    ) -> ChangeLifecycle {
        let mut active_revisions = BTreeMap::new();
        if let Some((id, digest)) = wi {
            active_revisions.insert(
                ArtifactKind::Wi,
                Some(make_revision(ArtifactKind::Wi, id, digest)),
            );
        }
        if let Some((id, digest)) = ec {
            active_revisions.insert(
                ArtifactKind::Ec,
                Some(make_revision(ArtifactKind::Ec, id, digest)),
            );
        }
        if let Some((id, digest)) = td {
            active_revisions.insert(
                ArtifactKind::Td,
                Some(make_revision(ArtifactKind::Td, id, digest)),
            );
        }
        if let Some((id, digest)) = cb {
            active_revisions.insert(
                ArtifactKind::Cb,
                Some(make_revision(ArtifactKind::Cb, id, digest)),
            );
        }

        ChangeLifecycle {
            schema: "aw.change-lifecycle.v1".to_string(),
            slug: "test-change".to_string(),
            epoch: 1,
            head_event_id: None,
            active_revisions,
            events: Vec::new(),
            evidence: Vec::new(),
            invalidations: Vec::new(),
            iteration: 1,
            terminal: false,
            next: crate::cli::change_lifecycle::NextObligation {
                command: "aw ec verify".to_string(),
                owner: crate::cli::change_lifecycle::OwnerVocabulary::Ec,
            },
        }
    }

    fn add_evidence(
        lc: &mut ChangeLifecycle,
        verifier: &str,
        passed: bool,
        tuple: ActiveDigestTuple,
    ) {
        lc.evidence.push(EvidenceBinding {
            verifier: verifier.to_string(),
            bound_tuple: tuple,
            passed,
            summary: format!("evidence for {verifier}"),
        });
    }

    #[test]
    fn ec_target_verification_profiles() {
        // Row 1: td target required dimension set is exactly behavior + security; efficiency and stability absent
        let req_td = VerificationTarget::Td.required_dimensions();
        assert_eq!(req_td, &["behavior", "security"]);
        assert!(!req_td.contains(&"efficiency"));
        assert!(!req_td.contains(&"stability"));

        let req_cb = VerificationTarget::Cb.required_dimensions();
        assert_eq!(req_cb, &["behavior", "efficiency", "security", "stability"]);

        // Row 2: lifecycle with active WI, EC, TD, CB. Evidence showing behavior and security green, nothing for efficiency/stability.
        let mut lc_row2 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row2 = lc_row2.active_digest_tuple();
        add_evidence(&mut lc_row2, "behavior", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "security", true, tuple_row2.clone());

        let v2 = decide_target_verdict(&lc_row2, VerificationTarget::Cb);
        assert!(!v2.is_green());
        let r2 = v2.reason().unwrap();
        assert!(
            r2.contains("efficiency"),
            "reason must name efficiency: {r2}"
        );
        assert!(r2.contains("stability"), "reason must name stability: {r2}");

        // Row 3: (negative control) CB lifecycle with all four dimensions green at current tuple
        let mut lc_row3 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row3 = lc_row3.active_digest_tuple();
        add_evidence(&mut lc_row3, "behavior", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "efficiency", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "security", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "stability", true, tuple_row3.clone());

        let v3 = decide_target_verdict(&lc_row3, VerificationTarget::Cb);
        assert!(v3.is_green());
        assert!(v3.reason().is_none());

        // Row 4: CB lifecycle, efficiency failed, behavior/security/stability green
        let mut lc_row4 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row4 = lc_row4.active_digest_tuple();
        add_evidence(&mut lc_row4, "behavior", true, tuple_row4.clone());
        add_evidence(&mut lc_row4, "efficiency", false, tuple_row4.clone());
        add_evidence(&mut lc_row4, "security", true, tuple_row4.clone());
        add_evidence(&mut lc_row4, "stability", true, tuple_row4.clone());

        let v4 = decide_target_verdict(&lc_row4, VerificationTarget::Cb);
        assert!(!v4.is_green());
        let r4 = v4.reason().unwrap();
        assert!(
            r4.contains("efficiency"),
            "reason must name efficiency: {r4}"
        );
        assert!(
            !r4.contains("behavior"),
            "reason must not name behavior: {r4}"
        );
        assert!(
            !r4.contains("security"),
            "reason must not name security: {r4}"
        );
        assert!(
            !r4.contains("stability"),
            "reason must not name stability: {r4}"
        );

        // Row 5: two lifecycles differing only in active CB revision, each carrying same behavior+security evidence bound to first lifecycle's tuple
        let mut lc_row5_a = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            None,
        );
        let tuple_row5_a = lc_row5_a.active_digest_tuple();
        add_evidence(&mut lc_row5_a, "behavior", true, tuple_row5_a.clone());
        add_evidence(&mut lc_row5_a, "security", true, tuple_row5_a.clone());

        let mut lc_row5_b = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        add_evidence(&mut lc_row5_b, "behavior", true, tuple_row5_a.clone());
        add_evidence(&mut lc_row5_b, "security", true, tuple_row5_a.clone());

        let v5_a = decide_target_verdict(&lc_row5_a, VerificationTarget::Td);
        let v5_b = decide_target_verdict(&lc_row5_b, VerificationTarget::Td);
        assert!(v5_a.is_green());
        assert!(v5_b.is_green());

        // Row 6: (negative control) TD lifecycle after active WI digest changes, with EC/TD unchanged
        let mut lc_row6 = make_lifecycle(
            Some(("wi-2", "d-wi-2")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            None,
        );
        add_evidence(&mut lc_row6, "behavior", true, tuple_row5_a.clone());
        add_evidence(&mut lc_row6, "security", true, tuple_row5_a.clone());

        let v6 = decide_target_verdict(&lc_row6, VerificationTarget::Td);
        assert!(v6.is_green());

        // Row 7: lifecycle whose active EC revision has been replaced by a new one
        let mut lc_row7_td = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-2", "d-ec-2")),
            Some(("td-1", "d-td-1")),
            None,
        );
        add_evidence(&mut lc_row7_td, "behavior", true, tuple_row5_a.clone());
        add_evidence(&mut lc_row7_td, "security", true, tuple_row5_a.clone());

        let v7_td = decide_target_verdict(&lc_row7_td, VerificationTarget::Td);
        assert!(!v7_td.is_green());
        let r7_td = v7_td.reason().unwrap();
        assert!(
            r7_td.contains("bound to a revision that is no longer active"),
            "td reason must name stale revision binding: {r7_td}"
        );

        let mut lc_row7_cb = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-2", "d-ec-2")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        add_evidence(&mut lc_row7_cb, "behavior", true, tuple_row3.clone());
        add_evidence(&mut lc_row7_cb, "efficiency", true, tuple_row3.clone());
        add_evidence(&mut lc_row7_cb, "security", true, tuple_row3.clone());
        add_evidence(&mut lc_row7_cb, "stability", true, tuple_row3.clone());

        let v7_cb = decide_target_verdict(&lc_row7_cb, VerificationTarget::Cb);
        assert!(!v7_cb.is_green());
        let r7_cb = v7_cb.reason().unwrap();
        assert!(
            r7_cb.contains("bound to a revision that is no longer active"),
            "cb reason must name stale revision binding: {r7_cb}"
        );

        // Row 8: CB lifecycle carrying green evidence for old verifiers cb_test, cb_review, td_reconcile, ec_verify_cb, covering only behavior
        let mut lc_row8 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row8 = lc_row8.active_digest_tuple();
        add_evidence(&mut lc_row8, "cb_test", true, tuple_row8.clone());
        add_evidence(&mut lc_row8, "cb_review", true, tuple_row8.clone());
        add_evidence(&mut lc_row8, "td_reconcile", true, tuple_row8.clone());
        add_evidence(&mut lc_row8, "ec_verify_cb", true, tuple_row8.clone());
        add_evidence(&mut lc_row8, "behavior", true, tuple_row8.clone());

        let v8 = decide_target_verdict(&lc_row8, VerificationTarget::Cb);
        assert!(!v8.is_green());
        let r8 = v8.reason().unwrap();
        assert!(r8.contains("security"), "reason must name security: {r8}");
        assert!(
            r8.contains("efficiency"),
            "reason must name efficiency: {r8}"
        );
        assert!(r8.contains("stability"), "reason must name stability: {r8}");

        // Row 9: TD lifecycle carrying green behavior, green security, AND green efficiency
        let mut lc_row9 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            None,
        );
        let tuple_row9 = lc_row9.active_digest_tuple();
        add_evidence(&mut lc_row9, "behavior", true, tuple_row9.clone());
        add_evidence(&mut lc_row9, "security", true, tuple_row9.clone());
        add_evidence(&mut lc_row9, "efficiency", true, tuple_row9.clone());

        let v9 = decide_target_verdict(&lc_row9, VerificationTarget::Td);
        assert!(!v9.is_green());
        let r9 = v9.reason().unwrap();
        assert!(
            r9.contains("efficiency"),
            "reason must name efficiency: {r9}"
        );
        assert!(r9.contains("td"), "reason must mention td target: {r9}");

        // Row 10: TD lifecycle carrying green behavior and security at current tuple, green efficiency bound to superseded EC digest
        let mut lc_row10 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-2", "d-ec-2")),
            Some(("td-1", "d-td-1")),
            None,
        );
        let tuple_row10 = lc_row10.active_digest_tuple();
        add_evidence(&mut lc_row10, "behavior", true, tuple_row10.clone());
        add_evidence(&mut lc_row10, "security", true, tuple_row10.clone());
        add_evidence(&mut lc_row10, "efficiency", true, tuple_row5_a.clone());

        let v10 = decide_target_verdict(&lc_row10, VerificationTarget::Td);
        assert!(!v10.is_green());
        let r10 = v10.reason().unwrap();
        assert!(
            r10.contains("efficiency"),
            "reason must name efficiency: {r10}"
        );
        assert!(
            r10.contains("is not a td dimension"),
            "reason must state efficiency is not a td dimension: {r10}"
        );
        assert!(
            !r10.contains("bound to a revision that is no longer active"),
            "reason must not report stale binding for unpermitted dimension: {r10}"
        );
    }
}
