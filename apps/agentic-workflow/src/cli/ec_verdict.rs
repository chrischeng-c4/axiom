//! Target verification profiles and pure target verdict evaluation (#3349).

use crate::cli::change_lifecycle::{
    expected_parent_set, route_failure, ArtifactKind, ArtifactRevision, ChangeLifecycle,
    EvidenceBinding, FailureOwnership, LifecycleEventKind, NextObligation, OwnerVocabulary,
};

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
    pub contradictory_dimensions: Vec<String>,
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

    let active_tuple = lifecycle.active_digest_tuple();

    let mut green_dimensions = Vec::new();
    let mut failing_dimensions = Vec::new();
    let mut unpermitted_dimensions = Vec::new();
    let mut stale_dimensions = Vec::new();
    let mut contradictory_dimensions = Vec::new();

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

                let is_active = match target {
                    VerificationTarget::Td => ec_ok && td_ok,
                    VerificationTarget::Cb => binding.bound_tuple == active_tuple,
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
        let has_fail = active_fail_set.contains(dim_str);
        let has_green = active_green_set.contains(dim_str);

        if has_fail && has_green {
            contradictory_dimensions.push(dim_str.to_string());
        } else if has_fail {
            failing_dimensions.push(dim_str.to_string());
        } else if has_green {
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
        && stale_dimensions.is_empty()
        && contradictory_dimensions.is_empty();

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
        if !contradictory_dimensions.is_empty() {
            parts.push(format!(
                "evidence for dimension(s) {} contains contradictory observations under active digest tuple",
                contradictory_dimensions.join(", ")
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
        contradictory_dimensions,
    }
}

/// Pure routed target verdict outcome (#3349).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetVerdictRouting {
    pub obligation: Option<NextObligation>,
    pub failure_ownership: Option<FailureOwnership>,
    pub blocked: bool,
    pub retryable: bool,
    pub required_declaration: Option<String>,
}

fn failure_ownership_priority(ownership: FailureOwnership) -> u8 {
    match ownership {
        FailureOwnership::Contract => 1,
        FailureOwnership::Design => 2,
        FailureOwnership::Implementation => 3,
        FailureOwnership::Infrastructure => 4,
        FailureOwnership::WiDrift => 5,
    }
}

/// Route a red target verdict to exactly one failure owner and obligation.
pub fn route_target_verdict(
    lifecycle: &ChangeLifecycle,
    verdict: &TargetVerdict,
    declared_failures: &std::collections::BTreeMap<String, FailureOwnership>,
) -> Option<TargetVerdictRouting> {
    if verdict.is_green() {
        return None;
    }

    let mut ownerships = Vec::new();
    for dim in &verdict.failing_dimensions {
        if let Some(&ownership) = declared_failures.get(dim) {
            ownerships.push(ownership);
        }
    }

    if ownerships.is_empty() {
        return Some(TargetVerdictRouting {
            obligation: None,
            failure_ownership: None,
            blocked: false,
            retryable: false,
            required_declaration: Some(
                "declared ownership for failing dimension required to route verdict".to_string(),
            ),
        });
    }

    ownerships.sort_by_key(|&o| failure_ownership_priority(o));
    let winning_ownership = ownerships[0];

    let obligation = route_failure(winning_ownership, &lifecycle.slug, &lifecycle.next.command);
    let (blocked, retryable) = match winning_ownership {
        FailureOwnership::Infrastructure => (true, true),
        _ => (false, false),
    };

    Some(TargetVerdictRouting {
        obligation: Some(obligation),
        failure_ownership: Some(winning_ownership),
        blocked,
        retryable,
        required_declaration: None,
    })
}

/// Resolved TD impact route after an EC repair (#3349 R4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TdImpactRoute {
    /// Parent-only rebind: TD source is unchanged, routing to TD review.
    Rebind,
    /// Source change: TD source differs, routing to TD authoring.
    Change,
}

/// Pure evaluation result for pending TD repair impact on a lifecycle (#3349 R4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdImpactStatus {
    pub has_pending_impact: bool,
    pub pre_repair_td_digest: Option<String>,
    pub verifiers_owing_rerun: Vec<String>,
}

/// Pure decision result for a proposed TD candidate after an EC repair (#3349 R4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdImpactDecision {
    pub route: Option<TdImpactRoute>,
    pub claim_accepted: bool,
    pub refusal_reason: Option<String>,
    pub obligation: NextObligation,
}

/// Inspect a lifecycle for pending TD impact resulting from a contract-owned (EC) repair (#3349 R4).
pub fn inspect_td_impact(lifecycle: &ChangeLifecycle) -> TdImpactStatus {
    let ec_invalidation = lifecycle
        .invalidations
        .iter()
        .rev()
        .find(|rec| rec.trigger_kind == ArtifactKind::Ec);

    let active_td_missing = lifecycle
        .active_revisions
        .get(&ArtifactKind::Td)
        .and_then(|r| r.as_ref())
        .is_none();

    if let Some(rec) = ec_invalidation {
        if active_td_missing {
            let pre_repair_td_digest = rec
                .evicted_evidence
                .iter()
                .find_map(|b| b.bound_tuple.td_digest.clone());

            let verifiers_owing_rerun = rec.evicted_evidence_verifiers.clone();

            return TdImpactStatus {
                has_pending_impact: true,
                pre_repair_td_digest,
                verifiers_owing_rerun,
            };
        }
    }

    TdImpactStatus {
        has_pending_impact: false,
        pre_repair_td_digest: None,
        verifiers_owing_rerun: Vec::new(),
    }
}

/// Decide the impact route for a proposed TD candidate after a contract-owned (EC) repair (#3349 R4).
pub fn decide_td_impact(
    lifecycle: &ChangeLifecycle,
    candidate: &ArtifactRevision,
    claimed_event_kind: LifecycleEventKind,
) -> TdImpactDecision {
    let status = inspect_td_impact(lifecycle);

    let expected_parents = expected_parent_set(lifecycle, ArtifactKind::Td);
    let parents_valid = expected_parents
        .as_ref()
        .is_some_and(|expected| candidate.parents == *expected);

    if !parents_valid {
        return TdImpactDecision {
            route: None,
            claim_accepted: false,
            refusal_reason: Some(
                "candidate parent set names a stale causal parent and does not match active causal predecessor set"
                    .to_string(),
            ),
            obligation: NextObligation {
                command: format!("aw wi validate {}", lifecycle.slug),
                owner: OwnerVocabulary::Wi,
            },
        };
    }

    let pre_digest = status.pre_repair_td_digest.as_deref();
    let digest_same = pre_digest.is_some_and(|d| d == candidate.digest.as_str());

    let route = if digest_same {
        TdImpactRoute::Rebind
    } else {
        TdImpactRoute::Change
    };

    let claim_accepted = match (claimed_event_kind, route) {
        (LifecycleEventKind::TdReconcile, TdImpactRoute::Rebind) => true,
        (LifecycleEventKind::TdChange, TdImpactRoute::Change) => true,
        _ => false,
    };

    let refusal_reason = if !claim_accepted {
        match claimed_event_kind {
            LifecycleEventKind::TdReconcile => Some(
                "TdReconcile claim refused: candidate TD source digest differs from pre-repair TD digest"
                    .to_string(),
            ),
            LifecycleEventKind::TdChange => Some(
                "TdChange claim refused: candidate TD source digest is identical to pre-repair TD digest"
                    .to_string(),
            ),
            _ => Some(format!(
                "claimed event kind {:?} is not a valid TD repair transition",
                claimed_event_kind
            )),
        }
    } else {
        None
    };

    let obligation = match route {
        TdImpactRoute::Rebind => NextObligation {
            command: "aw td review".to_string(),
            owner: OwnerVocabulary::Td,
        },
        TdImpactRoute::Change => NextObligation {
            command: "aw td check".to_string(),
            owner: OwnerVocabulary::Td,
        },
    };

    TdImpactDecision {
        route: Some(route),
        claim_accepted,
        refusal_reason,
        obligation,
    }
}

/// Caller-supplied post-CB TD reconciliation claim (#3350).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TdReconcileClaim {
    NoChange,
    Amended,
}

/// Pure decision result for post-CB TD reconciliation (#3350).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdReconciliationDecision {
    pub available: bool,
    pub result: Option<TdReconcileClaim>,
    pub refusal_reason: Option<String>,
    pub obligation: Option<NextObligation>,
    pub evicted_evidence: Vec<EvidenceBinding>,
    pub obligation_chain: Vec<NextObligation>,
}

/// Pure decision function answering whether post-CB TD reconciliation is available,
/// which result it records, and the single next command that follows (#3350).
pub fn decide_td_reconciliation(
    lifecycle: &ChangeLifecycle,
    claim: TdReconcileClaim,
) -> TdReconciliationDecision {
    let active_tuple = lifecycle.active_digest_tuple();

    // 1. Check ordering: reconciliation is ordered before EC CB verification.
    let has_passing_ec_verify_cb = lifecycle.evidence.iter().any(|binding| {
        binding.verifier == "ec_verify_cb" && binding.passed && binding.bound_tuple == active_tuple
    });

    if has_passing_ec_verify_cb {
        return TdReconciliationDecision {
            available: false,
            result: None,
            refusal_reason: Some(
                "ordering violation: passing ec_verify_cb evidence already exists for current active digest tuple"
                    .to_string(),
            ),
            obligation: None,
            evicted_evidence: Vec::new(),
            obligation_chain: Vec::new(),
        };
    }

    // 2. Check for passing cb_test and cb_review evidence bound to active_tuple.
    let cb_test_active = lifecycle
        .evidence
        .iter()
        .find(|b| b.verifier == "cb_test" && b.bound_tuple == active_tuple);
    let cb_review_active = lifecycle
        .evidence
        .iter()
        .find(|b| b.verifier == "cb_review" && b.bound_tuple == active_tuple);

    if let Some(b) = cb_test_active {
        if !b.passed {
            return TdReconciliationDecision {
                available: false,
                result: None,
                refusal_reason: Some(
                    "unaccepted CB test: cb_test evidence binding on active tuple has passed set to false"
                        .to_string(),
                ),
                obligation: None,
                evicted_evidence: Vec::new(),
                obligation_chain: Vec::new(),
            };
        }
    }

    if let Some(b) = cb_review_active {
        if !b.passed {
            return TdReconciliationDecision {
                available: false,
                result: None,
                refusal_reason: Some(
                    "unaccepted CB review: cb_review evidence binding on active tuple has passed set to false"
                        .to_string(),
                ),
                obligation: None,
                evicted_evidence: Vec::new(),
                obligation_chain: Vec::new(),
            };
        }
    }

    let has_passing_cb_test = cb_test_active.is_some_and(|b| b.passed);
    let has_passing_cb_review = cb_review_active.is_some_and(|b| b.passed);

    if !has_passing_cb_test || !has_passing_cb_review {
        let has_stale_cb_evidence = lifecycle.evidence.iter().any(|b| {
            (b.verifier == "cb_test" || b.verifier == "cb_review") && b.bound_tuple != active_tuple
        });

        let refusal_reason = if has_stale_cb_evidence {
            "stale evidence binding: cb_test or cb_review evidence bound_tuple does not match current active digest tuple".to_string()
        } else {
            "missing required evidence: passing cb_test and cb_review evidence bound to active tuple required".to_string()
        };

        return TdReconciliationDecision {
            available: false,
            result: None,
            refusal_reason: Some(refusal_reason),
            obligation: None,
            evicted_evidence: Vec::new(),
            obligation_chain: Vec::new(),
        };
    }

    let obligation = match claim {
        TdReconcileClaim::NoChange => NextObligation {
            command: format!("aw ec verify cb {}", lifecycle.slug),
            owner: OwnerVocabulary::Cb,
        },
        TdReconcileClaim::Amended => NextObligation {
            command: format!("aw td change {}", lifecycle.slug),
            owner: OwnerVocabulary::Td,
        },
    };

    let (evicted_evidence, obligation_chain) = match claim {
        TdReconcileClaim::NoChange => (Vec::new(), Vec::new()),
        TdReconcileClaim::Amended => {
            let mut evicted: Vec<EvidenceBinding> = lifecycle
                .evidence
                .iter()
                .filter(|b| {
                    matches!(
                        b.verifier.as_str(),
                        "cb_test" | "cb_review" | "ec_verify_cb"
                    )
                })
                .cloned()
                .collect();

            fn verifier_rank(v: &str) -> usize {
                match v {
                    "cb_test" => 0,
                    "cb_review" => 1,
                    "ec_verify_cb" => 2,
                    _ => 3,
                }
            }

            evicted.sort_by(|a, b| {
                verifier_rank(&a.verifier)
                    .cmp(&verifier_rank(&b.verifier))
                    .then_with(|| a.verifier.cmp(&b.verifier))
                    .then_with(|| {
                        a.bound_tuple
                            .wi_digest
                            .cmp(&b.bound_tuple.wi_digest)
                            .then_with(|| a.bound_tuple.ec_digest.cmp(&b.bound_tuple.ec_digest))
                            .then_with(|| a.bound_tuple.td_digest.cmp(&b.bound_tuple.td_digest))
                            .then_with(|| a.bound_tuple.cb_digest.cmp(&b.bound_tuple.cb_digest))
                    })
                    .then_with(|| a.passed.cmp(&b.passed))
                    .then_with(|| a.summary.cmp(&b.summary))
            });

            let chain = vec![
                NextObligation {
                    command: format!("aw td check {}", lifecycle.slug),
                    owner: OwnerVocabulary::Td,
                },
                NextObligation {
                    command: format!("aw ec verify td {}", lifecycle.slug),
                    owner: OwnerVocabulary::Ec,
                },
                NextObligation {
                    command: format!("aw td review {}", lifecycle.slug),
                    owner: OwnerVocabulary::Td,
                },
                NextObligation {
                    command: format!("aw cb check {}", lifecycle.slug),
                    owner: OwnerVocabulary::Cb,
                },
            ];

            (evicted, chain)
        }
    };

    TdReconciliationDecision {
        available: true,
        result: Some(claim),
        refusal_reason: None,
        obligation: Some(obligation),
        evicted_evidence,
        obligation_chain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::change_lifecycle::{
        reduce_event, ActiveDigestTuple, ArtifactKind, ArtifactRevision, CausalParent,
        EvidenceBinding, InvalidationRecord, LifecycleEvent, LifecycleEventKind, OwnerVocabulary,
        ReducerResult,
    };
    use std::collections::BTreeMap;

    fn route_target_verdict_with_slice(
        lifecycle: &ChangeLifecycle,
        verdict: &TargetVerdict,
        declared_failures: &[(&str, FailureOwnership)],
    ) -> Option<TargetVerdictRouting> {
        let map: std::collections::BTreeMap<String, FailureOwnership> = declared_failures
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        route_target_verdict(lifecycle, verdict, &map)
    }

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

    #[test]
    fn revisioned_change_wi_verdict_routing() {
        // Row 1: CB verdict whose only red dimension is behavior, declared contract-owned -> owner ec, command aw ec check
        let mut lc_row1 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row1.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row1 = lc_row1.active_digest_tuple();
        add_evidence(&mut lc_row1, "behavior", false, tuple_row1.clone());
        add_evidence(&mut lc_row1, "efficiency", true, tuple_row1.clone());
        add_evidence(&mut lc_row1, "security", true, tuple_row1.clone());
        add_evidence(&mut lc_row1, "stability", true, tuple_row1.clone());

        let v1 = decide_target_verdict(&lc_row1, VerificationTarget::Cb);
        assert!(!v1.is_green());
        assert_eq!(v1.failing_dimensions, vec!["behavior"]);

        let r1 = route_target_verdict_with_slice(
            &lc_row1,
            &v1,
            &[("behavior", FailureOwnership::Contract)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r1.obligation.as_ref().unwrap().owner, OwnerVocabulary::Ec);
        assert_eq!(r1.obligation.as_ref().unwrap().command, "aw ec check");

        // Row 2: identical CB verdict with identical red behavior dimension, declared implementation-owned -> owner cb, command aw cb check
        let mut lc_row2 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row2.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row2 = lc_row2.active_digest_tuple();
        add_evidence(&mut lc_row2, "behavior", false, tuple_row2.clone());
        add_evidence(&mut lc_row2, "efficiency", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "security", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "stability", true, tuple_row2.clone());

        let v2 = decide_target_verdict(&lc_row2, VerificationTarget::Cb);
        assert_eq!(
            v1, v2,
            "lifecycles and verdicts for row 1 and row 2 must be identical"
        );

        let r2 = route_target_verdict_with_slice(
            &lc_row2,
            &v2,
            &[("behavior", FailureOwnership::Implementation)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r2.obligation.as_ref().unwrap().owner, OwnerVocabulary::Cb);
        assert_eq!(r2.obligation.as_ref().unwrap().command, "aw cb check");
        assert_ne!(
            r1, r2,
            "row 1 and row 2 outcomes must differ by declared failure owner"
        );

        // Row 3: CB verdict whose only red dimension is stability, declared contract-owned -> same outcome as row 1 (ec, aw ec check)
        let mut lc_row3 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row3.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row3 = lc_row3.active_digest_tuple();
        add_evidence(&mut lc_row3, "behavior", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "efficiency", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "security", true, tuple_row3.clone());
        add_evidence(&mut lc_row3, "stability", false, tuple_row3.clone());

        let v3 = decide_target_verdict(&lc_row3, VerificationTarget::Cb);
        assert!(!v3.is_green());
        assert_eq!(v3.failing_dimensions, vec!["stability"]);

        let r3 = route_target_verdict_with_slice(
            &lc_row3,
            &v3,
            &[("stability", FailureOwnership::Contract)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r3.obligation.as_ref().unwrap().owner, OwnerVocabulary::Ec);
        assert_eq!(r3.obligation.as_ref().unwrap().command, "aw ec check");
        assert_eq!(
            r1.obligation, r3.obligation,
            "row 3 must yield same obligation as row 1"
        );

        // Row 4: TD verdict whose only red dimension is security, declared design-owned -> owner td, command aw td check
        let mut lc_row4 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            None,
        );
        lc_row4.next = NextObligation {
            command: "aw td check".to_string(),
            owner: OwnerVocabulary::Td,
        };
        let tuple_row4 = lc_row4.active_digest_tuple();
        add_evidence(&mut lc_row4, "behavior", true, tuple_row4.clone());
        add_evidence(&mut lc_row4, "security", false, tuple_row4.clone());

        let v4 = decide_target_verdict(&lc_row4, VerificationTarget::Td);
        assert!(!v4.is_green());
        assert_eq!(v4.failing_dimensions, vec!["security"]);

        let r4 = route_target_verdict_with_slice(
            &lc_row4,
            &v4,
            &[("security", FailureOwnership::Design)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r4.obligation.as_ref().unwrap().owner, OwnerVocabulary::Td);
        assert_eq!(r4.obligation.as_ref().unwrap().command, "aw td check");

        // Row 5: (negative control) green CB verdict with all four dimensions green -> no owner and no failure obligation
        let mut lc_row5 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row5 = lc_row5.active_digest_tuple();
        add_evidence(&mut lc_row5, "behavior", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "efficiency", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "security", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "stability", true, tuple_row5.clone());

        let v5 = decide_target_verdict(&lc_row5, VerificationTarget::Cb);
        assert!(v5.is_green());

        let r5 = route_target_verdict_with_slice(
            &lc_row5,
            &v5,
            &[("behavior", FailureOwnership::Contract)],
        );
        assert!(
            r5.is_none(),
            "green verdict must produce no failure routing outcome"
        );

        // Row 6: CB verdict whose only red dimension is behavior, failure declared infrastructure-owned, current command aw cb check
        let mut lc_row6 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row6.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row6 = lc_row6.active_digest_tuple();
        add_evidence(&mut lc_row6, "behavior", false, tuple_row6.clone());
        add_evidence(&mut lc_row6, "efficiency", true, tuple_row6.clone());
        add_evidence(&mut lc_row6, "security", true, tuple_row6.clone());
        add_evidence(&mut lc_row6, "stability", true, tuple_row6.clone());

        let v6 = decide_target_verdict(&lc_row6, VerificationTarget::Cb);
        assert!(!v6.is_green());

        let r6 = route_target_verdict_with_slice(
            &lc_row6,
            &v6,
            &[("behavior", FailureOwnership::Infrastructure)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r6.obligation.as_ref().unwrap().command, "aw cb check");
        assert_eq!(r6.obligation.as_ref().unwrap().owner, OwnerVocabulary::Cb);
        assert!(r6.blocked, "infrastructure outcome must be marked blocked");
        assert!(
            r6.retryable,
            "infrastructure outcome must be marked retryable"
        );
        assert_ne!(
            r6, r2,
            "infrastructure outcome must not equal implementation outcome r2"
        );

        // Row 7: CB verdict with two red dimensions whose declared owners differ, routed in order then in reverse order
        let mut lc_row7_a = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row7_a.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row7_a = lc_row7_a.active_digest_tuple();
        add_evidence(&mut lc_row7_a, "behavior", false, tuple_row7_a.clone());
        add_evidence(&mut lc_row7_a, "efficiency", false, tuple_row7_a.clone());
        add_evidence(&mut lc_row7_a, "security", true, tuple_row7_a.clone());
        add_evidence(&mut lc_row7_a, "stability", true, tuple_row7_a.clone());

        let v7_a = decide_target_verdict(&lc_row7_a, VerificationTarget::Cb);
        assert!(!v7_a.is_green());

        let declared_7 = [
            ("behavior", FailureOwnership::Contract),
            ("efficiency", FailureOwnership::Implementation),
        ];
        let r7_a = route_target_verdict_with_slice(&lc_row7_a, &v7_a, &declared_7)
            .expect("red verdict must yield routed outcome");

        let mut lc_row7_b = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        lc_row7_b.next = NextObligation {
            command: "aw cb check".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        let tuple_row7_b = lc_row7_b.active_digest_tuple();
        add_evidence(&mut lc_row7_b, "efficiency", false, tuple_row7_b.clone());
        add_evidence(&mut lc_row7_b, "behavior", false, tuple_row7_b.clone());
        add_evidence(&mut lc_row7_b, "security", true, tuple_row7_b.clone());
        add_evidence(&mut lc_row7_b, "stability", true, tuple_row7_b.clone());

        let v7_b = decide_target_verdict(&lc_row7_b, VerificationTarget::Cb);
        assert!(!v7_b.is_green());

        let r7_b = route_target_verdict_with_slice(&lc_row7_b, &v7_b, &declared_7)
            .expect("red verdict must yield routed outcome");

        assert_eq!(r7_a.obligation.as_ref().unwrap().owner, OwnerVocabulary::Ec);
        assert_eq!(r7_b.obligation.as_ref().unwrap().owner, OwnerVocabulary::Ec);
        assert_eq!(
            r7_a, r7_b,
            "verdict routing must be invariant under evidence ordering"
        );

        // Row 7 tie-break: same CB verdict, but declared owners are swapped so the first dimension in required order (behavior) carries lower-priority owner
        let declared_7_swapped = [
            ("behavior", FailureOwnership::Implementation),
            ("efficiency", FailureOwnership::Contract),
        ];
        let r7_swapped_a = route_target_verdict_with_slice(&lc_row7_a, &v7_a, &declared_7_swapped)
            .expect("red verdict must yield routed outcome");
        let r7_swapped_b = route_target_verdict_with_slice(&lc_row7_b, &v7_b, &declared_7_swapped)
            .expect("red verdict must yield routed outcome");

        assert_eq!(
            r7_swapped_a.obligation.as_ref().unwrap().owner,
            OwnerVocabulary::Ec,
            "higher-priority owner (Contract) must win regardless of required dimension order"
        );
        assert_eq!(
            r7_swapped_a.failure_ownership,
            Some(FailureOwnership::Contract)
        );
        assert_eq!(
            r7_swapped_b.obligation.as_ref().unwrap().owner,
            OwnerVocabulary::Ec,
            "higher-priority owner (Contract) must win regardless of required dimension order"
        );
        assert_eq!(
            r7_swapped_b.failure_ownership,
            Some(FailureOwnership::Contract)
        );
        assert_eq!(
            r7_swapped_a, r7_swapped_b,
            "verdict routing with tie-break must be invariant under evidence ordering"
        );

        // Row 8: (negative control) row 1 lifecycle cloned before routing and compared after
        let lc_row8 = lc_row1.clone();
        let _ = route_target_verdict_with_slice(
            &lc_row1,
            &v1,
            &[("behavior", FailureOwnership::Contract)],
        );
        assert_eq!(lc_row1, lc_row8, "lifecycle must be unchanged by routing");

        // Row 9: red CB verdict identical to row 2's, routed with no declared ownership for failing behavior dimension
        let r9 = route_target_verdict_with_slice(&lc_row2, &v2, &[])
            .expect("red verdict must yield routed outcome");
        assert!(
            r9.obligation.is_none(),
            "unattributed outcome must carry no obligation/owner"
        );
        assert!(
            r9.failure_ownership.is_none(),
            "unattributed outcome must carry no failure ownership"
        );
        assert!(
            !r9.blocked,
            "unattributed outcome must not be marked blocked"
        );
        assert!(
            !r9.retryable,
            "unattributed outcome must not be marked retryable"
        );
        assert_ne!(
            r9, r2,
            "unattributed outcome r9 must not equal row 2 outcome r2"
        );
        assert!(
            r9.required_declaration.is_some(),
            "unattributed outcome must name required declaration"
        );

        // Row 10: CB verdict red only because required dimensions have no evidence at all (failing_dimensions empty, missing_dimensions naming them)
        let mut lc_row10 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row10 = lc_row10.active_digest_tuple();
        add_evidence(&mut lc_row10, "behavior", true, tuple_row10.clone());
        add_evidence(&mut lc_row10, "security", true, tuple_row10.clone());

        let v10 = decide_target_verdict(&lc_row10, VerificationTarget::Cb);
        assert!(!v10.is_green());
        assert!(v10.failing_dimensions.is_empty());
        assert_eq!(v10.missing_dimensions, vec!["efficiency", "stability"]);

        let r10 = route_target_verdict_with_slice(&lc_row10, &v10, &[])
            .expect("red verdict must yield routed outcome");
        assert_eq!(
            r9, r10,
            "row 10 missing-evidence unattributed outcome must equal row 9 unattributed outcome"
        );
        assert_ne!(
            r10, r2,
            "unattributed outcome r10 must not equal row 2 outcome r2"
        );

        // Row 11: row 9 verdict routed with declared ownership naming security (a green dimension in this verdict)
        let r11 = route_target_verdict_with_slice(
            &lc_row2,
            &v2,
            &[("security", FailureOwnership::Design)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(
            r9, r11,
            "row 11 declaration for green dimension must yield same unattributed outcome as row 9"
        );

        // Row 12: (negative control) row 2 input - one failing dimension with declared owner for that dimension
        let r12 = route_target_verdict_with_slice(
            &lc_row2,
            &v2,
            &[("behavior", FailureOwnership::Implementation)],
        )
        .expect("red verdict must yield routed outcome");
        assert_eq!(r12, r2, "row 12 outcome must be unchanged from row 2");
        assert_eq!(r12.obligation.as_ref().unwrap().owner, OwnerVocabulary::Cb);
        assert_eq!(r12.obligation.as_ref().unwrap().command, "aw cb check");
        assert_eq!(
            r12.failure_ownership,
            Some(FailureOwnership::Implementation)
        );
    }

    #[test]
    fn ec_target_evidence_binding() {
        // Row 1: CB lifecycle whose active WI revision is W1, with all four required dimensions
        // passing on evidence whose bound_tuple.wi_digest is W0 (superseded) and ec/td/cb current.
        let mut lc_row1 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let active_tuple_row1 = lc_row1.active_digest_tuple();
        let mut tuple_row1_w0 = active_tuple_row1.clone();
        tuple_row1_w0.wi_digest = Some("d-wi-0".to_string());

        add_evidence(&mut lc_row1, "behavior", true, tuple_row1_w0.clone());
        add_evidence(&mut lc_row1, "efficiency", true, tuple_row1_w0.clone());
        add_evidence(&mut lc_row1, "security", true, tuple_row1_w0.clone());
        add_evidence(&mut lc_row1, "stability", true, tuple_row1_w0.clone());

        let v1 = decide_target_verdict(&lc_row1, VerificationTarget::Cb);
        assert!(!v1.is_green());
        let r1 = v1.reason().expect("verdict 1 must have reason");
        assert!(
            r1.contains("bound to a revision that is no longer active"),
            "reason must name superseded binding: {r1}"
        );
        assert!(
            !r1.contains("failing dimension"),
            "reason must not name a failing dimension: {r1}"
        );
        assert!(v1.failing_dimensions.is_empty());
        assert!(!v1.stale_dimensions.is_empty());

        // Row 2: (negative control) row 1's lifecycle with evidence wi_digest set to active W1
        let mut lc_row2 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row2 = lc_row2.active_digest_tuple();
        add_evidence(&mut lc_row2, "behavior", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "efficiency", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "security", true, tuple_row2.clone());
        add_evidence(&mut lc_row2, "stability", true, tuple_row2.clone());

        let v2 = decide_target_verdict(&lc_row2, VerificationTarget::Cb);
        assert!(v2.is_green());
        assert!(v2.stale_dimensions.is_empty());
        assert!(v2.reason().is_none());

        // Row 3: CB lifecycle with all four dimensions passing on current tuple, plus a fifth binding
        // for behavior that fails on the identical current tuple.
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
        add_evidence(&mut lc_row3, "behavior", false, tuple_row3.clone());

        let v3 = decide_target_verdict(&lc_row3, VerificationTarget::Cb);
        assert!(!v3.is_green());

        // Row 4: (negative control) same lifecycle with only failing behavior binding and no contradicting green one
        let mut lc_row4 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row4 = lc_row4.active_digest_tuple();
        add_evidence(&mut lc_row4, "behavior", false, tuple_row4.clone());
        add_evidence(&mut lc_row4, "efficiency", true, tuple_row4.clone());
        add_evidence(&mut lc_row4, "security", true, tuple_row4.clone());
        add_evidence(&mut lc_row4, "stability", true, tuple_row4.clone());

        let v4 = decide_target_verdict(&lc_row4, VerificationTarget::Cb);
        assert!(!v4.is_green());
        assert_eq!(v4.failing_dimensions, vec!["behavior"]);
        assert_eq!(
            v4.reason().as_deref(),
            Some("failing dimension(s): behavior")
        );

        // Row 3 outcome must not equal Row 4 outcome, and must name contradiction
        assert_ne!(v3, v4, "Row 3 outcome must not equal Row 4 outcome");
        assert!(v3.failing_dimensions.is_empty());
        let r3 = v3.reason().expect("verdict 3 must have reason");
        assert!(
            r3.contains("contradictory"),
            "Row 3 reason must name contradiction: {r3}"
        );

        // Row 5: row 3's lifecycle with the contradicting binding inserted first instead of last
        let mut lc_row5 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_row5 = lc_row5.active_digest_tuple();
        add_evidence(&mut lc_row5, "behavior", false, tuple_row5.clone());
        add_evidence(&mut lc_row5, "behavior", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "efficiency", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "security", true, tuple_row5.clone());
        add_evidence(&mut lc_row5, "stability", true, tuple_row5.clone());

        let v5 = decide_target_verdict(&lc_row5, VerificationTarget::Cb);
        assert_eq!(v3, v5, "Row 5 outcome must be identical to Row 3 outcome");

        // Row 6: row 1's binding and row 3's pair, each also passed to ActiveDigestTuple::matches against active tuple
        assert_eq!(
            tuple_row1_w0 == active_tuple_row1,
            false,
            "Row 1 binding tuple must not match active tuple"
        );
        assert_eq!(
            tuple_row3 == lc_row3.active_digest_tuple(),
            true,
            "Row 3 binding tuple matches active tuple"
        );
        assert_eq!(
            v1.is_green(),
            tuple_row1_w0 == active_tuple_row1,
            "Verdict admissibility must agree with matches for Row 1"
        );

        // Row 7: row 1 and row 3 verdicts passed to route_target_verdict with declared implementation ownership for behavior
        let declared = [("behavior", FailureOwnership::Implementation)];
        let r_route1 = route_target_verdict_with_slice(&lc_row1, &v1, &declared)
            .expect("red verdict must yield routing result");
        assert!(
            r_route1.obligation.is_none(),
            "Row 1 verdict routing must yield no implementation obligation"
        );
        assert!(
            r_route1.failure_ownership.is_none(),
            "Row 1 verdict routing must yield no failure ownership"
        );

        let r_route3 = route_target_verdict_with_slice(&lc_row3, &v3, &declared)
            .expect("red verdict must yield routing result");
        assert!(
            r_route3.obligation.is_none(),
            "Row 3 verdict routing must yield no implementation obligation"
        );
        assert!(
            r_route3.failure_ownership.is_none(),
            "Row 3 verdict routing must yield no failure ownership"
        );
    }

    #[test]
    fn ec_repair_td_impact_routing() {
        let pre_repair_tuple = ActiveDigestTuple {
            wi_digest: Some("d-wi-1".to_string()),
            ec_digest: Some("d-ec-1".to_string()),
            td_digest: Some("d-td-1".to_string()),
            cb_digest: Some("d-cb-1".to_string()),
        };

        let evicted_bindings = vec![
            EvidenceBinding {
                verifier: "td_behavior".to_string(),
                bound_tuple: pre_repair_tuple.clone(),
                passed: true,
                summary: "td behavior green".to_string(),
            },
            EvidenceBinding {
                verifier: "td_security".to_string(),
                bound_tuple: pre_repair_tuple.clone(),
                passed: true,
                summary: "td security green".to_string(),
            },
            EvidenceBinding {
                verifier: "cb_test".to_string(),
                bound_tuple: pre_repair_tuple.clone(),
                passed: true,
                summary: "cb test green".to_string(),
            },
        ];

        let inv_record = InvalidationRecord {
            trigger_revision_id: "rev-ec-2".to_string(),
            trigger_kind: ArtifactKind::Ec,
            invalidated_kinds: vec![ArtifactKind::Td, ArtifactKind::Cb],
            invalidated_revision_ids: vec!["rev-td-1".to_string(), "rev-cb-1".to_string()],
            evicted_evidence: evicted_bindings,
            evicted_evidence_verifiers: vec![
                "cb_test".to_string(),
                "td_behavior".to_string(),
                "td_security".to_string(),
            ],
            reason: "Transitive invalidation triggered by ec revision rev-ec-2".to_string(),
        };

        let rev_wi = make_revision(ArtifactKind::Wi, "rev-wi-1", "d-wi-1");
        let rev_ec_post = make_revision(ArtifactKind::Ec, "rev-ec-2", "d-ec-2");

        let mut active_revisions = BTreeMap::new();
        active_revisions.insert(ArtifactKind::Wi, Some(rev_wi));
        active_revisions.insert(ArtifactKind::Ec, Some(rev_ec_post));
        active_revisions.insert(ArtifactKind::Td, None);
        active_revisions.insert(ArtifactKind::Cb, None);

        let repaired_lc = ChangeLifecycle {
            schema: "aw.change-lifecycle.v1".to_string(),
            slug: "test-repair-slug".to_string(),
            epoch: 2,
            head_event_id: Some("evt-002".to_string()),
            active_revisions,
            events: Vec::new(),
            evidence: Vec::new(),
            invalidations: vec![inv_record],
            iteration: 2,
            terminal: false,
            next: NextObligation {
                command: "aw td check".to_string(),
                owner: OwnerVocabulary::Td,
            },
        };

        let expected_td_parents = expected_parent_set(&repaired_lc, ArtifactKind::Td)
            .expect("repaired lifecycle must yield expected parent set for TD");

        // Row 1: candidate claiming TdReconcile whose digest equals pre-repair TD digest ("d-td-1") and valid parents
        let cand_row1 = ArtifactRevision {
            id: "rev-td-new1".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-1".to_string(),
            parents: expected_td_parents.clone(),
            iteration: 3,
            superseded_by: None,
            invalidation_reason: None,
        };

        let dec_row1 = decide_td_impact(&repaired_lc, &cand_row1, LifecycleEventKind::TdReconcile);
        assert_eq!(dec_row1.route, Some(TdImpactRoute::Rebind));
        assert!(dec_row1.claim_accepted);
        assert!(dec_row1.refusal_reason.is_none());
        assert_eq!(dec_row1.obligation.owner, OwnerVocabulary::Td);
        assert_eq!(dec_row1.obligation.command, "aw td review");

        // Row 2: candidate claiming TdReconcile whose digest differs ("d-td-v9") from pre-repair TD digest
        let cand_row2 = ArtifactRevision {
            id: "rev-td-new2".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-v9".to_string(),
            parents: expected_td_parents.clone(),
            iteration: 3,
            superseded_by: None,
            invalidation_reason: None,
        };

        let dec_row2 = decide_td_impact(&repaired_lc, &cand_row2, LifecycleEventKind::TdReconcile);
        assert_eq!(dec_row2.route, Some(TdImpactRoute::Change));
        assert_ne!(
            dec_row1.route, dec_row2.route,
            "Row 1 (rebind) and Row 2 (change) routes must differ"
        );
        assert!(!dec_row2.claim_accepted);
        assert!(dec_row2.refusal_reason.is_some());
        assert_eq!(dec_row2.obligation.owner, OwnerVocabulary::Td);
        assert_eq!(dec_row2.obligation.command, "aw td check");

        // Row 3: candidate claiming TdChange whose digest equals pre-repair TD digest ("d-td-1")
        let cand_row3 = ArtifactRevision {
            id: "rev-td-new3".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-1".to_string(),
            parents: expected_td_parents.clone(),
            iteration: 3,
            superseded_by: None,
            invalidation_reason: None,
        };

        let dec_row3 = decide_td_impact(&repaired_lc, &cand_row3, LifecycleEventKind::TdChange);
        assert_eq!(
            dec_row3.route,
            Some(TdImpactRoute::Rebind),
            "Row 3 must resolve to rebind route because digest is unchanged"
        );
        assert_eq!(
            dec_row3.route, dec_row1.route,
            "Row 3 must return the Row 1 route"
        );
        assert_eq!(
            dec_row3.obligation, dec_row1.obligation,
            "Row 3 obligation must match Row 1 obligation (TD review)"
        );

        // Row 4: repaired lifecycle before any candidate
        let status_row4 = inspect_td_impact(&repaired_lc);
        assert!(status_row4.has_pending_impact);
        assert_eq!(status_row4.pre_repair_td_digest.as_deref(), Some("d-td-1"));
        assert_eq!(
            status_row4.verifiers_owing_rerun,
            vec!["cb_test", "td_behavior", "td_security"]
        );
        assert!(repaired_lc.evidence.is_empty());
        let verdict_td = decide_target_verdict(&repaired_lc, VerificationTarget::Td);
        assert!(
            !verdict_td.is_green(),
            "Evicted evidence must not be admitted as active observation"
        );

        // Row 5: (negative control) lifecycle with all four revisions active, current evidence, invalidations empty
        let clean_lc = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let status_row5 = inspect_td_impact(&clean_lc);
        assert!(
            !status_row5.has_pending_impact,
            "Clean lifecycle must have no pending TD impact"
        );
        assert!(status_row5.pre_repair_td_digest.is_none());
        assert!(status_row5.verifiers_owing_rerun.is_empty());

        // Row 6: candidate whose parents name the superseded EC revision instead
        let stale_parents = vec![
            CausalParent {
                revision_id: "rev-wi-1".to_string(),
                digest: "d-wi-1".to_string(),
            },
            CausalParent {
                revision_id: "rev-ec-1".to_string(),
                digest: "d-ec-1".to_string(),
            },
        ];
        let cand_row6 = ArtifactRevision {
            id: "rev-td-stale".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-1".to_string(),
            parents: stale_parents,
            iteration: 3,
            superseded_by: None,
            invalidation_reason: None,
        };

        let dec_row6 = decide_td_impact(&repaired_lc, &cand_row6, LifecycleEventKind::TdReconcile);
        assert!(!dec_row6.claim_accepted);
        assert!(
            dec_row6.route.is_none(),
            "Stale parent candidate must route as neither rebind nor change"
        );
        let r6_reason = dec_row6
            .refusal_reason
            .expect("Stale candidate must have refusal reason");
        assert!(
            r6_reason.contains("stale") || r6_reason.contains("parent"),
            "Refusal reason must name stale causal parent: {r6_reason}"
        );

        // Row 7: purity check - repaired lifecycle compared before and after calls, and twice on each input
        let lc_snapshot = repaired_lc.clone();

        let d1_first = decide_td_impact(&repaired_lc, &cand_row1, LifecycleEventKind::TdReconcile);
        assert_eq!(
            repaired_lc, lc_snapshot,
            "Lifecycle must remain unchanged after Row 1 decision"
        );
        let d1_second = decide_td_impact(&repaired_lc, &cand_row1, LifecycleEventKind::TdReconcile);
        assert_eq!(
            repaired_lc, lc_snapshot,
            "Lifecycle must remain unchanged after second Row 1 call"
        );
        assert_eq!(
            d1_first, d1_second,
            "Two calls on Row 1 input must return equal answers"
        );

        let d2_first = decide_td_impact(&repaired_lc, &cand_row2, LifecycleEventKind::TdReconcile);
        assert_eq!(
            repaired_lc, lc_snapshot,
            "Lifecycle must remain unchanged after Row 2 decision"
        );
        let d2_second = decide_td_impact(&repaired_lc, &cand_row2, LifecycleEventKind::TdReconcile);
        assert_eq!(
            repaired_lc, lc_snapshot,
            "Lifecycle must remain unchanged after second Row 2 call"
        );
        assert_eq!(
            d2_first, d2_second,
            "Two calls on Row 2 input must return equal answers"
        );

        // Row 8: candidate claiming TdChange whose digest differs from pre-repair TD digest and valid parents
        let cand_row8 = ArtifactRevision {
            id: "rev-td-new8".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-v9".to_string(),
            parents: expected_td_parents.clone(),
            iteration: 3,
            superseded_by: None,
            invalidation_reason: None,
        };

        let dec_row8 = decide_td_impact(&repaired_lc, &cand_row8, LifecycleEventKind::TdChange);
        assert_eq!(dec_row8.route, Some(TdImpactRoute::Change));
        assert!(dec_row8.claim_accepted);
        assert!(dec_row8.refusal_reason.is_none());
        assert_eq!(dec_row8.obligation.owner, OwnerVocabulary::Td);
        assert_eq!(dec_row8.obligation.command, "aw td check");

        // Row 9: lifecycle built from repaired_lc by making Td active again, with invalidations left unchanged
        let mut lc_row9 = repaired_lc.clone();
        let rev_td_active = make_revision(ArtifactKind::Td, "rev-td-2", "d-td-2");
        lc_row9
            .active_revisions
            .insert(ArtifactKind::Td, Some(rev_td_active));

        let status_row9 = inspect_td_impact(&lc_row9);
        assert!(!status_row9.has_pending_impact);
        assert!(status_row9.pre_repair_td_digest.is_none());
        assert!(status_row9.verifiers_owing_rerun.is_empty());

        // Row 10: lifecycle whose only invalidation record has trigger_kind Wi and invalidated_kinds [Ec, Td, Cb], with active Td missing
        let inv_record_wi = InvalidationRecord {
            trigger_revision_id: "rev-wi-2".to_string(),
            trigger_kind: ArtifactKind::Wi,
            invalidated_kinds: vec![ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb],
            invalidated_revision_ids: vec![
                "rev-ec-1".to_string(),
                "rev-td-1".to_string(),
                "rev-cb-1".to_string(),
            ],
            evicted_evidence: Vec::new(),
            evicted_evidence_verifiers: Vec::new(),
            reason: "Invalidation triggered by wi revision rev-wi-2".to_string(),
        };

        let mut lc_row10 = make_lifecycle(
            Some(("wi-2", "d-wi-2")),
            Some(("ec-1", "d-ec-1")),
            None,
            None,
        );
        lc_row10.invalidations = vec![inv_record_wi];

        let status_row10 = inspect_td_impact(&lc_row10);
        assert!(!status_row10.has_pending_impact);
        assert!(status_row10.pre_repair_td_digest.is_none());
        assert!(status_row10.verifiers_owing_rerun.is_empty());
    }

    #[test]
    fn revisioned_change_wi_td_reconciliation() {
        // Row 1: lifecycle with a complete WI/EC/TD/CB active tuple and passing cb_test + cb_review bound to that tuple; claim no_change
        let mut lc_row1 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let active_tuple = lc_row1.active_digest_tuple();
        add_evidence(&mut lc_row1, "cb_test", true, active_tuple.clone());
        add_evidence(&mut lc_row1, "cb_review", true, active_tuple.clone());

        let active_td_before = lc_row1.active_revisions.get(&ArtifactKind::Td).cloned();

        let dec1 = decide_td_reconciliation(&lc_row1, TdReconcileClaim::NoChange);
        assert!(dec1.available);
        assert_eq!(dec1.result, Some(TdReconcileClaim::NoChange));
        assert!(dec1.refusal_reason.is_none());
        let ob1 = dec1
            .obligation
            .as_ref()
            .expect("obligation must be present");
        assert_eq!(ob1.owner, OwnerVocabulary::Cb);
        assert_eq!(ob1.command, format!("aw ec verify cb {}", lc_row1.slug));

        let active_td_after = lc_row1.active_revisions.get(&ArtifactKind::Td).cloned();
        assert_eq!(active_td_before, active_td_after);

        // Row 2: the same lifecycle; claim amended
        let dec2 = decide_td_reconciliation(&lc_row1, TdReconcileClaim::Amended);
        assert!(dec2.available);
        assert_eq!(dec2.result, Some(TdReconcileClaim::Amended));
        assert!(dec2.refusal_reason.is_none());
        let ob2 = dec2
            .obligation
            .as_ref()
            .expect("obligation must be present");
        assert_eq!(ob2.owner, OwnerVocabulary::Td);
        assert_eq!(ob2.command, format!("aw td change {}", lc_row1.slug));

        // Row 3: the same lifecycle with the cb_review binding's passed set to false (negative control)
        let mut lc_row3 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        add_evidence(&mut lc_row3, "cb_test", true, active_tuple.clone());
        add_evidence(&mut lc_row3, "cb_review", false, active_tuple.clone());

        let dec3 = decide_td_reconciliation(&lc_row3, TdReconcileClaim::NoChange);
        assert!(!dec3.available);
        assert!(dec3.result.is_none());
        assert!(dec3.obligation.is_none());
        let r3 = dec3
            .refusal_reason
            .as_ref()
            .expect("refusal reason must be set for row 3");
        assert!(
            r3.contains("unaccepted CB review"),
            "refusal reason must name unaccepted CB review: {r3}"
        );

        // Row 4: the same lifecycle whose cb_test and cb_review bindings carry a bound_tuple whose td_digest differs from the active tuple
        let mut lc_row4 = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let mut stale_tuple = active_tuple.clone();
        stale_tuple.td_digest = Some("d-td-stale".to_string());
        add_evidence(&mut lc_row4, "cb_test", true, stale_tuple.clone());
        add_evidence(&mut lc_row4, "cb_review", true, stale_tuple.clone());

        let dec4 = decide_td_reconciliation(&lc_row4, TdReconcileClaim::NoChange);
        assert!(!dec4.available);
        assert!(dec4.result.is_none());
        assert!(dec4.obligation.is_none());
        let r4 = dec4
            .refusal_reason
            .as_ref()
            .expect("refusal reason must be set for row 4");
        assert!(
            r4.contains("stale evidence"),
            "refusal reason must name stale binding: {r4}"
        );
        assert_ne!(r3, r4, "row 3 and row 4 refusal reasons must differ");

        // Row 5: the same lifecycle as row 1 plus a passing ec_verify_cb binding on the current tuple
        let mut lc_row5 = lc_row1.clone();
        add_evidence(&mut lc_row5, "ec_verify_cb", true, active_tuple.clone());

        let dec5 = decide_td_reconciliation(&lc_row5, TdReconcileClaim::NoChange);
        assert!(!dec5.available);
        assert!(dec5.result.is_none());
        assert!(dec5.obligation.is_none());
        let r5 = dec5
            .refusal_reason
            .as_ref()
            .expect("refusal reason must be set for row 5");
        assert!(
            r5.contains("ordering violation"),
            "refusal reason must name ordering violation: {r5}"
        );
        assert_ne!(r3, r5, "row 3 and row 5 refusal reasons must differ");
        assert_ne!(r4, r5, "row 4 and row 5 refusal reasons must differ");
    }

    #[test]
    fn td_reconciliation_invalidation() {
        // Fixture F: cb_test passed, cb_review passed, ec_verify_cb failed, td_behavior passed
        let mut lc_f = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let active_tuple = lc_f.active_digest_tuple();
        add_evidence(&mut lc_f, "cb_test", true, active_tuple.clone());
        add_evidence(&mut lc_f, "cb_review", true, active_tuple.clone());
        add_evidence(&mut lc_f, "ec_verify_cb", false, active_tuple.clone());
        add_evidence(&mut lc_f, "td_behavior", true, active_tuple.clone());

        // Row 1: F, claim Amended
        let dec1 = decide_td_reconciliation(&lc_f, TdReconcileClaim::Amended);
        assert!(dec1.available);
        assert_eq!(dec1.result, Some(TdReconcileClaim::Amended));
        assert!(dec1.refusal_reason.is_none());
        let ob1 = dec1
            .obligation
            .as_ref()
            .expect("obligation must be present");
        assert_eq!(ob1.owner, OwnerVocabulary::Td);
        assert_eq!(ob1.command, format!("aw td change {}", lc_f.slug));

        // Eviction set: exactly cb_test, cb_review, ec_verify_cb; td_behavior absent
        assert_eq!(dec1.evicted_evidence.len(), 3);
        assert_eq!(dec1.evicted_evidence[0], lc_f.evidence[0]);
        assert_eq!(dec1.evicted_evidence[1], lc_f.evidence[1]);
        assert_eq!(dec1.evicted_evidence[2], lc_f.evidence[2]);
        assert!(
            dec1.evicted_evidence
                .iter()
                .all(|b| b.verifier != "td_behavior"),
            "td_behavior must be absent from eviction set"
        );

        // Obligation chain: 4 entries in exact order
        assert_eq!(dec1.obligation_chain.len(), 4);
        assert_eq!(dec1.obligation_chain[0].owner, OwnerVocabulary::Td);
        assert_eq!(
            dec1.obligation_chain[0].command,
            format!("aw td check {}", lc_f.slug)
        );
        assert_eq!(dec1.obligation_chain[1].owner, OwnerVocabulary::Ec);
        assert_eq!(
            dec1.obligation_chain[1].command,
            format!("aw ec verify td {}", lc_f.slug)
        );
        assert_eq!(dec1.obligation_chain[2].owner, OwnerVocabulary::Td);
        assert_eq!(
            dec1.obligation_chain[2].command,
            format!("aw td review {}", lc_f.slug)
        );
        assert_eq!(dec1.obligation_chain[3].owner, OwnerVocabulary::Cb);
        assert_eq!(
            dec1.obligation_chain[3].command,
            format!("aw cb check {}", lc_f.slug)
        );

        // Row 2: F with the four bindings pushed onto lifecycle.evidence in reverse
        let mut lc_f_rev = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        add_evidence(&mut lc_f_rev, "td_behavior", true, active_tuple.clone());
        add_evidence(&mut lc_f_rev, "ec_verify_cb", false, active_tuple.clone());
        add_evidence(&mut lc_f_rev, "cb_review", true, active_tuple.clone());
        add_evidence(&mut lc_f_rev, "cb_test", true, active_tuple.clone());

        let dec2 = decide_td_reconciliation(&lc_f_rev, TdReconcileClaim::Amended);
        assert!(dec2.available);
        assert_eq!(dec2.result, Some(TdReconcileClaim::Amended));
        assert_eq!(dec2.evicted_evidence, dec1.evicted_evidence);
        assert_eq!(dec2.obligation_chain, dec1.obligation_chain);

        // Row 3: F, claim NoChange
        let dec3 = decide_td_reconciliation(&lc_f, TdReconcileClaim::NoChange);
        assert!(dec3.available);
        assert_eq!(dec3.result, Some(TdReconcileClaim::NoChange));
        assert!(dec3.refusal_reason.is_none());
        let ob3 = dec3
            .obligation
            .as_ref()
            .expect("obligation must be present");
        assert_eq!(ob3.owner, OwnerVocabulary::Cb);
        assert_eq!(ob3.command, format!("aw ec verify cb {}", lc_f.slug));
        assert!(dec3.evicted_evidence.is_empty());
        assert!(dec3.obligation_chain.is_empty());

        // Row 4: F with ec_verify_cb flipped to passed (negative control)
        let mut lc_f_neg = lc_f.clone();
        if let Some(b) = lc_f_neg
            .evidence
            .iter_mut()
            .find(|b| b.verifier == "ec_verify_cb")
        {
            b.passed = true;
        }
        let dec4 = decide_td_reconciliation(&lc_f_neg, TdReconcileClaim::Amended);
        assert!(!dec4.available);
        assert!(dec4.result.is_none());
        assert!(dec4.obligation.is_none());
        let r4 = dec4
            .refusal_reason
            .as_ref()
            .expect("refusal reason must be set for row 4");
        assert!(
            r4.contains("ordering violation"),
            "refusal reason must name ordering violation: {r4}"
        );
        assert_eq!(
            r4,
            "ordering violation: passing ec_verify_cb evidence already exists for current active digest tuple"
        );
        assert!(dec4.evicted_evidence.is_empty());
        assert!(dec4.obligation_chain.is_empty());
    }

    #[test]
    fn td_reconciliation_side_effect_boundary() {
        // Row 1: production half of ec_verdict.rs contains no Command::new, std::process, std::fs, or std::env, and contains fn decide_td_reconciliation
        let source = include_str!("ec_verdict.rs");
        let non_test = source
            .split("#[cfg(test)]")
            .next()
            .expect("ec_verdict.rs must contain #[cfg(test)]");
        assert!(
            !non_test.contains("Command::new"),
            "production code must contain no Command::new"
        );
        assert!(
            !non_test.contains("std::process"),
            "production code must contain no std::process"
        );
        assert!(
            !non_test.contains("std::fs"),
            "production code must contain no std::fs"
        );
        assert!(
            !non_test.contains("std::env"),
            "production code must contain no std::env"
        );
        assert!(
            non_test.contains("fn decide_td_reconciliation"),
            "production code scan must contain fn decide_td_reconciliation"
        );

        // Fixture L: lifecycle at a reviewed, tested, current CB candidate carrying two evidence bindings (cb_test and cb_review)
        let mut l = make_lifecycle(
            Some(("wi-1", "d-wi-1")),
            Some(("ec-1", "d-ec-1")),
            Some(("td-1", "d-td-1")),
            Some(("cb-1", "d-cb-1")),
        );
        let tuple_l = l.active_digest_tuple();
        add_evidence(&mut l, "cb_test", true, tuple_l.clone());
        add_evidence(&mut l, "cb_review", true, tuple_l.clone());

        let expected_td_parents =
            expected_parent_set(&l, ArtifactKind::Td).expect("L must have expected parents for TD");

        // Row 2: L and a TdReconcile event e whose candidate carries expected parent set
        let cand_row2 = ArtifactRevision {
            id: "rev-td-reconciled".to_string(),
            kind: ArtifactKind::Td,
            digest: "d-td-reconciled".to_string(),
            parents: expected_td_parents.clone(),
            iteration: 2,
            superseded_by: None,
            invalidation_reason: None,
        };
        let e_row2 = LifecycleEvent {
            event_id: "evt-001".to_string(),
            predecessor_id: l.head_event_id.clone(),
            kind: LifecycleEventKind::TdReconcile,
            candidate_revision: cand_row2,
            bound_tuple: ActiveDigestTuple {
                td_digest: Some("d-td-reconciled".to_string()),
                ..l.active_digest_tuple()
            },
            next_command: format!("aw td review {}", l.slug),
            next_owner: OwnerVocabulary::Td,
        };

        let res2 = reduce_event(&l, e_row2.clone());
        assert!(
            res2.accepted,
            "Row 2: expected accepted == true, got {:?}",
            res2
        );
        assert_eq!(
            res2.lifecycle.epoch,
            l.epoch + 1,
            "Row 2: expected epoch L.epoch + 1 ({}), got {}",
            l.epoch + 1,
            res2.lifecycle.epoch
        );
        assert_eq!(
            res2.lifecycle.events.len(),
            l.events.len() + 1,
            "Row 2: expected events.len() L.events.len() + 1 ({}), got {}",
            l.events.len() + 1,
            res2.lifecycle.events.len()
        );
        assert_eq!(
            res2.lifecycle.events.last(),
            Some(&e_row2),
            "Row 2: expected last event equal to e"
        );

        // Row 3: L and a TdReconcile whose candidate is L's active TD revision replayed verbatim (parent set is empty, not active causal predecessor set)
        let active_td_verbatim = l
            .active_revisions
            .get(&ArtifactKind::Td)
            .and_then(|r| r.as_ref())
            .expect("L must have active TD revision")
            .clone();
        let e_stale = LifecycleEvent {
            event_id: "evt-001".to_string(),
            predecessor_id: l.head_event_id.clone(),
            kind: LifecycleEventKind::TdReconcile,
            candidate_revision: active_td_verbatim,
            bound_tuple: l.active_digest_tuple(),
            next_command: format!("aw wi validate {}", l.slug),
            next_owner: OwnerVocabulary::Wi,
        };

        let res3 = reduce_event(&l, e_stale);
        assert!(
            !res3.accepted,
            "Row 3: expected accepted == false, got accepted == true"
        );
        let r3_reason = res3
            .rejection_reason
            .as_ref()
            .expect("Row 3 must have rejection_reason");
        assert!(
            r3_reason.contains("causal predecessor set"),
            "Row 3 rejection reason must name causal predecessor set: {r3_reason}"
        );
        assert_eq!(
            res3.lifecycle.epoch, l.epoch,
            "Row 3: epoch must equal L.epoch ({}), got {}",
            l.epoch, res3.lifecycle.epoch
        );
        assert_eq!(
            res3.lifecycle.events.len(),
            l.events.len(),
            "Row 3: events.len() must equal L.events.len() ({}), got {}",
            l.events.len(),
            res3.lifecycle.events.len()
        );
        assert_eq!(
            res3.lifecycle.evidence.len(),
            l.evidence.len(),
            "Row 3: evidence.len() must equal L.evidence.len() ({}), got {}",
            l.evidence.len(),
            res3.lifecycle.evidence.len()
        );
        assert_eq!(
            res3.lifecycle.invalidations.len(),
            l.invalidations.len(),
            "Row 3: invalidations.len() must equal L.invalidations.len() ({}), got {}",
            l.invalidations.len(),
            res3.lifecycle.invalidations.len()
        );
        assert_eq!(
            res3.lifecycle.head_event_id, l.head_event_id,
            "Row 3: head_event_id must equal L.head_event_id ({:?}), got {:?}",
            l.head_event_id, res3.lifecycle.head_event_id
        );
        assert_eq!(
            res3.lifecycle.active_revisions.get(&ArtifactKind::Cb),
            l.active_revisions.get(&ArtifactKind::Cb),
            "Row 3: active CB revision must equal L's active CB revision"
        );

        // Row 4: L and a TdReconcile carrying a changed TD digest with expected parent set
        let res4 = res2;
        assert_eq!(
            res4.lifecycle.active_revisions.get(&ArtifactKind::Cb),
            Some(&None),
            "Row 4: active CB revision must be cleared to None"
        );
        assert_eq!(
            res4.lifecycle.invalidations.len(),
            1,
            "Row 4: expected exactly 1 invalidation record appended, got {}",
            res4.lifecycle.invalidations.len()
        );
        assert_eq!(
            res4.lifecycle.invalidations[0].trigger_kind,
            ArtifactKind::Td,
            "Row 4: invalidation trigger_kind must be ArtifactKind::Td"
        );
    }
}
