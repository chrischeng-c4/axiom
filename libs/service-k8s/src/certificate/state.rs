// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-state" tracker="#3110" reason="Own the rotation state machine as a pure function of observed cluster state and time, so restart-safety and overlap ordering are properties of one testable decision rather than of when the controller happened to be running."
//! What to do next, decided from what is observably true.
//!
//! [`next_action`] is a pure function of (what the service wants, what the
//! cluster currently shows, what time it is). It keeps no memory, and that is
//! the whole design:
//!
//! * **R4, restart safety.** A controller that remembers "I issued one a minute
//!   ago" mints a duplicate every time it restarts, and the failure is invisible
//!   — two valid certificates look exactly like one. Reconstructing the decision
//!   from the leaf's own metadata makes restart a non-event by construction.
//! * **R5, ordering.** Overlap is not "publish the bundle, then sleep, then
//!   issue". It is a sequence of states, each of which is visible in the
//!   cluster, so an interrupted rotation resumes where it stopped rather than
//!   restarting or, worse, skipping a step.
//!
//! Note what [`Action`] cannot express: there is no variant that removes the
//! current leaf. R5's "a failed step retains the last valid serving material"
//! is not a rule this module follows, it is a sentence it cannot say.

use std::time::Duration;

use chrono::{DateTime, Utc};

use super::digest::hex_sha256;
use super::issuer::IssuerId;
use super::profile::CertificateProfile;

/// The leaf currently projected into the instance's Secret, as read back from
/// the cluster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedLeaf {
    pub issuer: IssuerId,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    /// Lowercase hex sha256 of the leaf DER.
    pub fingerprint: String,
    /// [`CertificateProfile::identity_digest`] of the profile it was issued
    /// for. A change here means the service now wants a different identity,
    /// which is a reissue even though the current leaf is perfectly valid.
    pub identity_digest: String,
}

/// Everything the reconciler could see, gathered before deciding anything.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    pub leaf: Option<ObservedLeaf>,
    /// Issuers whose anchors are published in the trust bundle right now.
    pub trust_bundle: Vec<IssuerId>,
    /// The leaf fingerprint the runtime reports it is *actually* presenting.
    ///
    /// This is the difference between "we wrote a file" and "the process is
    /// serving it", and it is the only honest input to the retire decision:
    /// a workload that has not reloaded is still authenticating with the old
    /// leaf, signed by the issuer we are about to remove.
    pub activated_fingerprint: Option<String>,
    /// Consecutive failed attempts since the last success. Drives backoff only;
    /// it never changes which action is correct.
    pub consecutive_failures: u32,
}

/// Why an issuance is being asked for. Surfaced in conditions and events so an
/// operator can tell a routine renewal from a scramble.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueReason {
    /// Nothing is projected yet.
    Bootstrap,
    /// The current leaf is inside its renewal window.
    Renewal,
    /// The current leaf is already expired. Distinct from `Renewal` because it
    /// means something was wrong for a while.
    Expired,
    /// The service now wants different names or usages.
    IdentityChanged,
    /// The operator moved to a different issuer.
    IssuerRotation,
}

impl IssueReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bootstrap => "Bootstrap",
            Self::Renewal => "Renewal",
            Self::Expired => "Expired",
            Self::IdentityChanged => "IdentityChanged",
            Self::IssuerRotation => "IssuerRotation",
        }
    }
}

/// The single next step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Publish anchors for `issuers` — always a superset of what is published
    /// now, never a replacement. Widening trust is safe; narrowing it is
    /// [`Action::RetireIssuers`], which has preconditions.
    PublishTrustBundle { issuers: Vec<IssuerId> },
    /// Request a new leaf from `issuer` and project it.
    Issue {
        issuer: IssuerId,
        reason: IssueReason,
    },
    /// The new leaf is projected but the runtime is still presenting the old
    /// one. Nothing to do but look again.
    AwaitActivation {
        fingerprint: String,
        recheck_after: Duration,
    },
    /// Drop anchors that nothing is using any more.
    RetireIssuers { issuers: Vec<IssuerId> },
    /// Everything matches. Come back at `renew_at`.
    Wait { until: DateTime<Utc> },
}

/// What the service wants right now.
pub struct Desired<'a> {
    pub profile: &'a CertificateProfile,
    /// The issuer the operator wants to be on. Changing this is what starts a
    /// rotation.
    pub issuer: IssuerId,
}

/// How often to re-check while waiting for the runtime to pick up a new leaf.
/// Short: this is the one window in a rotation where the old issuer is still
/// trusted and the new leaf is not yet in use, and shortening it shortens how
/// long the system sits in a state it is only passing through.
const ACTIVATION_RECHECK: Duration = Duration::from_secs(15);

/// Decide the next step.
pub fn next_action(
    desired: &Desired<'_>,
    observed: &Observed,
    now: DateTime<Utc>,
) -> Action {
    // Trust before identity, always. A leaf from an issuer nobody trusts is
    // worse than no leaf: it fails at handshake time, on the far side, where
    // the error says "unknown CA" and names nothing useful.
    if !observed.trust_bundle.contains(&desired.issuer) {
        let mut issuers = observed.trust_bundle.clone();
        issuers.push(desired.issuer.clone());
        issuers.sort();
        issuers.dedup();
        return Action::PublishTrustBundle { issuers };
    }

    let Some(leaf) = &observed.leaf else {
        return Action::Issue {
            issuer: desired.issuer.clone(),
            reason: IssueReason::Bootstrap,
        };
    };

    if leaf.issuer != desired.issuer {
        return Action::Issue {
            issuer: desired.issuer.clone(),
            reason: IssueReason::IssuerRotation,
        };
    }
    if leaf.identity_digest != desired.profile.identity_digest() {
        return Action::Issue {
            issuer: desired.issuer.clone(),
            reason: IssueReason::IdentityChanged,
        };
    }
    if now >= leaf.not_after {
        return Action::Issue {
            issuer: desired.issuer.clone(),
            reason: IssueReason::Expired,
        };
    }

    // The leaf is from the right issuer and carries the right names. If the
    // bundle still holds anyone else, we are in the tail of a rotation.
    let stale: Vec<IssuerId> = observed
        .trust_bundle
        .iter()
        .filter(|issuer| **issuer != desired.issuer)
        .cloned()
        .collect();
    if !stale.is_empty() {
        // Retire only once the runtime says it is presenting the new leaf.
        // Anything less -- a written file, a successful API call, a sleep --
        // retires an issuer that is still authenticating live connections.
        if observed.activated_fingerprint.as_deref() == Some(leaf.fingerprint.as_str()) {
            return Action::RetireIssuers { issuers: stale };
        }
        return Action::AwaitActivation {
            fingerprint: leaf.fingerprint.clone(),
            recheck_after: ACTIVATION_RECHECK,
        };
    }

    let renew_at = renew_at(desired.profile, leaf);
    if now >= renew_at {
        return Action::Issue {
            issuer: desired.issuer.clone(),
            reason: IssueReason::Renewal,
        };
    }
    Action::Wait { until: renew_at }
}

/// When `leaf` becomes due for renewal: its expiry, less the profile's renewal
/// window, plus a deterministic jitter.
///
/// The jitter is derived from the leaf's own fingerprint, not from a random
/// number generator, and that is not a shortcut — it is the requirement. A
/// random offset chosen at reconcile time changes on every restart, so a
/// controller that restarts often would drift its own renewal time around and
/// could, at the wrong moment, decide a leaf is not yet due when the previous
/// process had already decided it was. Deriving it from the leaf means every
/// process, on every restart, computes the same instant for the same
/// certificate — while different certificates still spread out, which is the
/// point of jitter (R4).
pub fn renew_at(profile: &CertificateProfile, leaf: &ObservedLeaf) -> DateTime<Utc> {
    let window = chrono::Duration::from_std(profile.renew_before()).unwrap_or(chrono::Duration::zero());
    let base = leaf.not_after - window;
    let jitter_secs = profile.renew_jitter().as_secs();
    if jitter_secs == 0 {
        return base;
    }
    let offset = fingerprint_offset(&leaf.fingerprint, jitter_secs);
    base + chrono::Duration::seconds(offset as i64)
}

/// A stable value in `0..span`, derived from a fingerprint.
fn fingerprint_offset(fingerprint: &str, span: u64) -> u64 {
    let digest = hex_sha256(fingerprint.as_bytes());
    let mut acc: u64 = 0;
    for byte in digest.as_bytes().iter().take(16) {
        acc = acc.wrapping_mul(31).wrapping_add(u64::from(*byte));
    }
    acc % span
}

/// How long to wait before retrying after `failures` consecutive failures.
///
/// Exponential from 5s to a 5-minute ceiling. The ceiling matters more than the
/// curve: the renewal window is hours wide, so a controller that has been
/// failing for a while should keep trying at a steady, unalarming rate rather
/// than back off into a schedule that would miss the expiry it is racing.
pub fn retry_after(failures: u32) -> Duration {
    const BASE_SECS: u64 = 5;
    const CEILING_SECS: u64 = 300;
    if failures == 0 {
        return Duration::from_secs(BASE_SECS);
    }
    let shift = failures.min(16);
    let secs = BASE_SECS.saturating_mul(1u64 << shift.min(6));
    Duration::from_secs(secs.min(CEILING_SECS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::profile::{
        CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
    };

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn profile() -> CertificateProfile {
        CertificateProfile::new(
            &scope(),
            Purpose::Serving,
            "lumen.lumen.svc.cluster.local",
            CertificateIdentity {
                dns_names: vec!["lumen.lumen.svc.cluster.local".into()],
                spiffe_uri: None,
            },
            Duration::from_secs(86_400),
            Duration::from_secs(21_600),
            Duration::from_secs(1_800),
        )
        .unwrap()
    }

    fn at(hours: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + hours * 3_600, 0).unwrap()
    }

    fn leaf(issuer: &str, profile: &CertificateProfile) -> ObservedLeaf {
        ObservedLeaf {
            issuer: IssuerId::new(issuer),
            not_before: at(0),
            not_after: at(24),
            fingerprint: "aa".repeat(32),
            identity_digest: profile.identity_digest(),
        }
    }

    #[test]
    fn trust_is_published_before_anything_is_issued() {
        let profile = profile();
        let desired = Desired {
            profile: &profile,
            issuer: IssuerId::new("pool-a"),
        };
        let action = next_action(&desired, &Observed::default(), at(1));
        assert_eq!(
            action,
            Action::PublishTrustBundle {
                issuers: vec![IssuerId::new("pool-a")]
            },
            "a leaf from an untrusted issuer fails on the far side, where the error names nothing"
        );
    }

    #[test]
    fn a_healthy_leaf_waits_until_its_renewal_instant() {
        let profile = profile();
        let observed = Observed {
            leaf: Some(leaf("pool-a", &profile)),
            trust_bundle: vec![IssuerId::new("pool-a")],
            ..Observed::default()
        };
        let desired = Desired {
            profile: &profile,
            issuer: IssuerId::new("pool-a"),
        };
        let Action::Wait { until } = next_action(&desired, &observed, at(1)) else {
            panic!("expected Wait");
        };
        // 24h expiry, 6h window, up to 30m jitter.
        assert!(until >= at(18) && until <= at(18) + chrono::Duration::minutes(30));
    }

    #[test]
    fn the_renewal_instant_does_not_move_when_the_controller_restarts() {
        let profile = profile();
        let leaf = leaf("pool-a", &profile);
        let first = renew_at(&profile, &leaf);
        let second = renew_at(&profile, &leaf);
        assert_eq!(
            first, second,
            "a jitter drawn at reconcile time would make every restart a different deadline"
        );
    }

    #[test]
    fn different_certificates_still_spread_out() {
        let profile = profile();
        let mut a = leaf("pool-a", &profile);
        let mut b = a.clone();
        a.fingerprint = "11".repeat(32);
        b.fingerprint = "22".repeat(32);
        assert_ne!(
            renew_at(&profile, &a),
            renew_at(&profile, &b),
            "deterministic must not mean identical; the point of jitter is that a fleet does \
             not renew in lockstep"
        );
    }

    #[test]
    fn an_identity_change_reissues_a_perfectly_valid_leaf() {
        let profile = profile();
        let mut observed = Observed {
            leaf: Some(leaf("pool-a", &profile)),
            trust_bundle: vec![IssuerId::new("pool-a")],
            ..Observed::default()
        };
        observed.leaf.as_mut().unwrap().identity_digest = "stale".into();
        let desired = Desired {
            profile: &profile,
            issuer: IssuerId::new("pool-a"),
        };
        assert_eq!(
            next_action(&desired, &observed, at(1)),
            Action::Issue {
                issuer: IssuerId::new("pool-a"),
                reason: IssueReason::IdentityChanged
            }
        );
    }

    #[test]
    fn an_expired_leaf_is_distinguishable_from_a_due_one() {
        let profile = profile();
        let observed = Observed {
            leaf: Some(leaf("pool-a", &profile)),
            trust_bundle: vec![IssuerId::new("pool-a")],
            ..Observed::default()
        };
        let desired = Desired {
            profile: &profile,
            issuer: IssuerId::new("pool-a"),
        };
        assert_eq!(
            next_action(&desired, &observed, at(25)),
            Action::Issue {
                issuer: IssuerId::new("pool-a"),
                reason: IssueReason::Expired
            },
            "'expired' means something was already wrong; folding it into 'renewal' hides that"
        );
    }

    #[test]
    fn no_action_can_remove_the_current_leaf() {
        // Enumerated rather than asserted on one case: the guarantee is about
        // the shape of `Action`, so the test that matters is that every
        // reachable variant leaves existing material alone.
        let profile = profile();
        let desired = Desired {
            profile: &profile,
            issuer: IssuerId::new("pool-b"),
        };
        let states = [
            Observed::default(),
            Observed {
                leaf: Some(leaf("pool-a", &profile)),
                trust_bundle: vec![IssuerId::new("pool-a")],
                ..Observed::default()
            },
            Observed {
                leaf: Some(leaf("pool-b", &profile)),
                trust_bundle: vec![IssuerId::new("pool-a"), IssuerId::new("pool-b")],
                activated_fingerprint: Some("aa".repeat(32)),
                ..Observed::default()
            },
        ];
        for observed in states {
            match next_action(&desired, &observed, at(1)) {
                Action::PublishTrustBundle { .. }
                | Action::Issue { .. }
                | Action::AwaitActivation { .. }
                | Action::RetireIssuers { .. }
                | Action::Wait { .. } => {}
            }
        }
    }

    #[test]
    fn backoff_climbs_to_a_ceiling_and_stays_there() {
        assert_eq!(retry_after(0), Duration::from_secs(5));
        assert!(retry_after(1) < retry_after(4));
        assert_eq!(retry_after(20), Duration::from_secs(300));
        assert_eq!(
            retry_after(u32::MAX),
            Duration::from_secs(300),
            "a controller racing an expiry must not back off past the window it is racing"
        );
    }
}
// HANDWRITE-END
