// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-status" tracker="#3110" reason="Own what a certificate lifecycle is allowed to say about itself, and make the redaction a property of the only path that produces status rather than a rule each call site is asked to remember."
//! What the lifecycle is allowed to say about itself.
//!
//! Certificate state is genuinely useful to publish: an operator debugging a
//! handshake failure wants to know which issuer signed the leaf, when it
//! expires, and whether a rotation is in flight. Every one of those facts sits
//! next to something that must never leave the process — the private key, the
//! projected KSA token used to reach the CA.
//!
//! So this module has exactly one way to produce status, and that way redacts.
//! [`redact`] is not a helper callers are asked to remember; [`CertificateFacts`]
//! carries no secret-bearing field in the first place, and every string it emits
//! passes through the scrubber on the way out. The rule is enforced by what the
//! type can hold, not by discipline at the call site.
//!
//! ### On expiry as a public fact
//!
//! `notAfter` is on the certificate any TLS client already receives during a
//! handshake. Publishing it discloses nothing and is what makes an expiry alert
//! possible without granting anyone read access to the Secret.

use chrono::{DateTime, Utc};

use crate::service::{ConditionFact, ConditionStatus};

use super::issuer::IssuerId;
use super::profile::Purpose;
use super::state::{Action, IssueReason};

/// Condition types this lifecycle owns.
pub const READY_CONDITION: &str = "CertificateReady";
pub const ROTATING_CONDITION: &str = "CertificateRotating";

/// The publishable summary of one purpose's certificate state.
///
/// Note the absent fields: no PEM, no key, no token, no CA request body. There
/// is no `message` a caller can stuff arbitrary bytes into either — messages are
/// composed here from these fields alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateFacts {
    pub purpose: Purpose,
    /// Issuer that signed the leaf currently projected, if there is one.
    pub issuer: Option<IssuerId>,
    /// Expiry of the projected leaf. Already public via any handshake.
    pub not_after: Option<DateTime<Utc>>,
    /// Short fingerprint prefix — enough to correlate two observations, not
    /// enough to be mistaken for material.
    pub fingerprint: Option<String>,
    /// Issuers currently trusted, in bundle order.
    pub trust_bundle: Vec<IssuerId>,
    /// Whether the next action moves state rather than waiting.
    pub rotating: Option<IssueReason>,
    /// Consecutive failed attempts, for the message on a stuck lifecycle.
    pub consecutive_failures: u32,
}

impl CertificateFacts {
    /// Build facts from what the projector observed and what the state machine
    /// decided to do next. Taking the decision as input rather than recomputing
    /// it is deliberate: status then cannot describe a different reconcile than
    /// the one that ran.
    pub fn from_action(
        purpose: Purpose,
        issuer: Option<IssuerId>,
        not_after: Option<DateTime<Utc>>,
        fingerprint: Option<&str>,
        trust_bundle: Vec<IssuerId>,
        consecutive_failures: u32,
        action: &Action,
    ) -> Self {
        Self {
            purpose,
            issuer,
            not_after,
            fingerprint: fingerprint.map(short_fingerprint),
            trust_bundle,
            rotating: match action {
                Action::Issue { reason, .. } => Some(*reason),
                _ => None,
            },
            consecutive_failures,
        }
    }

    /// The conditions to merge into the owning resource's status.
    ///
    /// Two conditions rather than one because they answer different questions.
    /// `CertificateReady` is "can this instance serve TLS right now" — a
    /// readiness gate. `CertificateRotating` is "is material changing" — an
    /// informational signal that must not itself make an instance unready, or
    /// every routine renewal would look like an outage.
    pub fn conditions(&self) -> Vec<ConditionFact> {
        let prefix = condition_prefix(self.purpose);
        let ready = self.issuer.is_some() && self.not_after.is_some();

        let ready_fact = if ready {
            ConditionFact::new(
                format!("{prefix}{READY_CONDITION}"),
                ConditionStatus::True,
                "Issued",
                redact(&self.ready_message()),
            )
        } else if self.consecutive_failures > 0 {
            ConditionFact::new(
                format!("{prefix}{READY_CONDITION}"),
                ConditionStatus::False,
                "IssuanceFailing",
                redact(&format!(
                    "no {} certificate projected after {} consecutive attempts",
                    self.purpose.as_str(),
                    self.consecutive_failures
                )),
            )
        } else {
            ConditionFact::new(
                format!("{prefix}{READY_CONDITION}"),
                ConditionStatus::False,
                "Pending",
                redact(&format!(
                    "no {} certificate projected yet",
                    self.purpose.as_str()
                )),
            )
        };

        let rotating_fact = match self.rotating {
            Some(reason) => ConditionFact::new(
                format!("{prefix}{ROTATING_CONDITION}"),
                ConditionStatus::True,
                rotation_reason(reason),
                redact(&format!(
                    "issuing a new {} certificate: {}",
                    self.purpose.as_str(),
                    rotation_detail(reason)
                )),
            ),
            None => ConditionFact::new(
                format!("{prefix}{ROTATING_CONDITION}"),
                ConditionStatus::False,
                "Stable",
                String::new(),
            ),
        };

        vec![ready_fact, rotating_fact]
    }

    fn ready_message(&self) -> String {
        let mut parts = Vec::new();
        if let Some(issuer) = &self.issuer {
            parts.push(format!("issuer {}", issuer.as_str()));
        }
        if let Some(fingerprint) = &self.fingerprint {
            parts.push(format!("leaf {fingerprint}"));
        }
        if let Some(not_after) = self.not_after {
            parts.push(format!("expires {}", not_after.to_rfc3339()));
        }
        if !self.trust_bundle.is_empty() {
            parts.push(format!(
                "trusting {}",
                self.trust_bundle
                    .iter()
                    .map(IssuerId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        parts.join("; ")
    }
}

/// Conditions are namespaced by purpose so serving and peer material are
/// separately diagnosable — the whole point of #2890's identity split is that
/// one can be broken while the other is fine.
fn condition_prefix(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::Serving => "Serving",
        Purpose::Peer => "Peer",
    }
}

fn rotation_reason(reason: IssueReason) -> &'static str {
    match reason {
        IssueReason::Bootstrap => "Bootstrap",
        IssueReason::Renewal => "Renewal",
        IssueReason::Expired => "Expired",
        IssueReason::IdentityChanged => "IdentityChanged",
        IssueReason::IssuerRotation => "IssuerRotation",
    }
}

fn rotation_detail(reason: IssueReason) -> &'static str {
    match reason {
        IssueReason::Bootstrap => "no material has been issued yet",
        IssueReason::Renewal => "the renewal window has opened",
        IssueReason::Expired => "the projected leaf is past its notAfter",
        IssueReason::IdentityChanged => "the requested names no longer match the leaf",
        IssueReason::IssuerRotation => "the configured issuer changed",
    }
}

/// First 16 hex characters of a fingerprint.
///
/// Long enough that two different leaves will not collide in practice, short
/// enough that nobody mistakes a status field for a copyable artifact.
fn short_fingerprint(fingerprint: &str) -> String {
    fingerprint.chars().take(16).collect()
}

/// Strip anything that must never reach status, a log line, or an event.
///
/// Three shapes, all of which have escaped into status in real systems: PEM
/// blocks (a key pasted into an error message), bearer tokens (an upstream 401
/// echoing the Authorization header back), and JWT-shaped strings (a projected
/// KSA token in a request dump).
///
/// This is a backstop, not the primary defence — the primary defence is that
/// [`CertificateFacts`] has nowhere to put those bytes. A backstop that is never
/// exercised in production is exactly the one worth having.
pub fn redact(text: &str) -> String {
    let without_pem = strip_pem(text);
    let without_bearer = strip_bearer(&without_pem);
    strip_jwt(&without_bearer)
}

fn strip_pem(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("-----BEGIN") {
        out.push_str(&rest[..start]);
        out.push_str("[redacted pem]");
        match rest[start..].find("-----END") {
            Some(end_offset) => {
                let after_end = start + end_offset;
                // Drop through the closing dashes as well, if they are present.
                let tail = &rest[after_end..];
                match tail.find("-----\n").or_else(|| tail.find("-----")) {
                    Some(_) => {
                        let close = tail
                            .match_indices("-----")
                            .nth(1)
                            .map(|(idx, _)| idx + 5)
                            .unwrap_or(tail.len());
                        rest = &tail[close.min(tail.len())..];
                    }
                    None => rest = "",
                }
            }
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn strip_bearer(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (index, segment) in text.split("Bearer ").enumerate() {
        if index == 0 {
            out.push_str(segment);
            continue;
        }
        out.push_str("Bearer [redacted]");
        match segment.find(char::is_whitespace) {
            Some(idx) => out.push_str(&segment[idx..]),
            None => {}
        }
    }
    out
}

/// A JWT is three base64url segments joined by dots. Anything matching that
/// shape is treated as a token regardless of where it came from.
fn strip_jwt(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let parts: Vec<&str> = word.split('.').collect();
            let looks_like_jwt = parts.len() == 3
                && parts.iter().all(|part| {
                    part.len() >= 8
                        && part
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                });
            if looks_like_jwt {
                "[redacted token]".to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn facts() -> CertificateFacts {
        CertificateFacts {
            purpose: Purpose::Peer,
            issuer: Some(IssuerId::new("pool-a")),
            not_after: Some(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap()),
            fingerprint: Some("0123456789abcdef".into()),
            trust_bundle: vec![IssuerId::new("pool-a")],
            rotating: None,
            consecutive_failures: 0,
        }
    }

    #[test]
    fn a_pem_block_never_survives_into_status() {
        let leaked = format!(
            "issued {}",
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBg\n-----END PRIVATE KEY-----"
        );
        let cleaned = redact(&leaked);
        assert!(!cleaned.contains("MIIEvQIBADANBg"));
        assert!(!cleaned.contains("BEGIN PRIVATE KEY"));
        assert!(cleaned.contains("[redacted pem]"));
    }

    #[test]
    fn a_bearer_token_never_survives_into_status() {
        let cleaned = redact("upstream said 401 for Bearer ya29.a0AfB_secretvalue here");
        assert!(!cleaned.contains("ya29.a0AfB_secretvalue"));
        assert!(cleaned.contains("[redacted]"));
    }

    #[test]
    fn a_projected_token_never_survives_into_status() {
        let cleaned = redact("request carried eyJhbGciOiJSUzI1NiJ9.eyJhdWQiOlsibHVtZW4iXX0.c2lnbmF0dXJl as audience proof");
        assert!(!cleaned.contains("eyJhbGciOiJSUzI1NiJ9"));
        assert!(cleaned.contains("[redacted token]"));
    }

    #[test]
    fn ordinary_text_passes_through_unharmed() {
        let text = "issuer pool-a; expires 2026-08-01T00:00:00+00:00";
        assert_eq!(redact(text), text);
    }

    #[test]
    fn a_projected_certificate_reports_ready() {
        let conditions = facts().conditions();
        let ready = &conditions[0];
        assert_eq!(ready.type_, "PeerCertificateReady");
        assert_eq!(ready.status, ConditionStatus::True);
        assert!(ready.message.contains("pool-a"));
        assert!(ready.message.contains("expires"));
    }

    #[test]
    fn a_rotation_does_not_make_the_instance_unready() {
        let mut facts = facts();
        facts.rotating = Some(IssueReason::Renewal);
        let conditions = facts.conditions();
        assert_eq!(conditions[0].status, ConditionStatus::True);
        assert_eq!(conditions[1].type_, "PeerCertificateRotating");
        assert_eq!(conditions[1].status, ConditionStatus::True);
        assert_eq!(conditions[1].reason, "Renewal");
    }

    #[test]
    fn a_failing_lifecycle_says_so_rather_than_staying_pending() {
        let facts = CertificateFacts {
            issuer: None,
            not_after: None,
            fingerprint: None,
            consecutive_failures: 4,
            ..facts()
        };
        let conditions = facts.conditions();
        assert_eq!(conditions[0].status, ConditionStatus::False);
        assert_eq!(conditions[0].reason, "IssuanceFailing");
        assert!(conditions[0].message.contains('4'));
    }

    #[test]
    fn serving_and_peer_conditions_do_not_collide() {
        let serving = CertificateFacts {
            purpose: Purpose::Serving,
            ..facts()
        };
        assert_eq!(serving.conditions()[0].type_, "ServingCertificateReady");
        assert_eq!(facts().conditions()[0].type_, "PeerCertificateReady");
    }

    #[test]
    fn the_published_fingerprint_is_a_correlator_not_an_artifact() {
        let facts = CertificateFacts::from_action(
            Purpose::Serving,
            Some(IssuerId::new("pool-a")),
            None,
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Vec::new(),
            0,
            &Action::Wait {
                until: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            },
        );
        assert_eq!(facts.fingerprint.as_deref(), Some("0123456789abcdef"));
    }
}
// HANDWRITE-END
