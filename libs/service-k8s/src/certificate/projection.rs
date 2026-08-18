// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-projection" tracker="#3110" reason="Own how leaf material and a widening trust bundle are laid out in one owner-scoped Secret, including which facts are read back from the certificate itself rather than from an annotation that could disagree with it."
//! Where the material lives, and how it is read back.
//!
//! One Secret per purpose per instance, carrying the same three keys #2890
//! already projects — `tls.crt`, `tls.key`, `ca.crt`. Keeping that layout is
//! deliberate: the consumer contract for peer material is already deployed, and
//! a lifecycle that changed it would have to change the pod spec too, which is
//! the one thing R9 says renewal must never require.
//!
//! `ca.crt` is the trust *bundle*, not a single anchor. During a rotation it
//! holds the outgoing and incoming issuers at once, which is what makes the
//! overlap in [`super::state`] mean anything: a verifier reading this file
//! accepts both while the fleet crosses over. `rustls`' root store — and so
//! `peer_tls::PeerTlsConfig` — reads every PEM block in the file, so this needs
//! no consumer change either.
//!
//! ### Which facts are read from where
//!
//! Expiry and fingerprint are parsed from the certificate itself. Issuer ids
//! and the identity digest come from annotations, because they are not
//! derivable from the DER — an issuer id is our name for a pool, not the
//! subject on the chain.
//!
//! That split is the honest one. An annotation is a claim; a certificate is
//! evidence. Anywhere both could answer, the certificate answers, so a
//! hand-edited annotation cannot talk the controller into believing a leaf
//! expires later than it does.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};

use super::digest::hex_sha256;
use super::issuer::{IssuedMaterial, IssuerId};
use super::profile::{InstanceScope, Purpose};
use super::state::ObservedLeaf;

/// The three keys, in the order an operator would look for them.
pub const CERT_KEY: &str = "tls.crt";
pub const PRIVATE_KEY_KEY: &str = "tls.key";
pub const TRUST_BUNDLE_KEY: &str = "ca.crt";

/// Annotation carrying the ordered issuer ids whose anchors are in `ca.crt`.
pub const TRUST_BUNDLE_ANNOTATION: &str = "service-k8s.axiom.dev/trust-bundle";
/// Annotation naming the issuer that signed the leaf in `tls.crt`.
pub const LEAF_ISSUER_ANNOTATION: &str = "service-k8s.axiom.dev/leaf-issuer";
/// Annotation carrying the profile identity digest the leaf was issued for.
pub const IDENTITY_DIGEST_ANNOTATION: &str = "service-k8s.axiom.dev/identity-digest";

/// Labels every object this lifecycle writes carries, so a sweep can find them
/// and an operator can tell at a glance what created them.
fn labels(scope: &InstanceScope, purpose: Purpose) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_string(), scope.instance.clone()),
        (
            "app.kubernetes.io/managed-by".to_string(),
            "service-k8s".to_string(),
        ),
        (
            "app.kubernetes.io/component".to_string(),
            format!("{}-tls", purpose.as_str()),
        ),
    ])
}

/// The owning custom resource. Every Secret this lifecycle writes is garbage
/// collected with it — R7's "scoped to one instance" applies to cleanup too,
/// and an orphaned Secret full of key material is exactly the kind of residue
/// nobody notices until an audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Owner {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub uid: String,
}

impl Owner {
    fn reference(&self) -> Value {
        json!({
            "apiVersion": self.api_version,
            "kind": self.kind,
            "name": self.name,
            "uid": self.uid,
            "controller": true,
            "blockOwnerDeletion": true,
        })
    }
}

/// An ordered set of issuer anchors.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustBundle {
    entries: Vec<(IssuerId, String)>,
}

impl TrustBundle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an anchor, replacing any previous PEM for the same issuer.
    pub fn insert(&mut self, issuer: IssuerId, anchor_pem: impl Into<String>) {
        let pem = anchor_pem.into();
        match self.entries.iter_mut().find(|(id, _)| *id == issuer) {
            Some(entry) => entry.1 = pem,
            None => self.entries.push((issuer, pem)),
        }
        self.entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    /// Keep only `issuers`. Used by the retire step, never by publish.
    pub fn retain(&mut self, issuers: &[IssuerId]) {
        self.entries.retain(|(id, _)| issuers.contains(id));
    }

    pub fn issuers(&self) -> Vec<IssuerId> {
        self.entries.iter().map(|(id, _)| id.clone()).collect()
    }

    pub fn contains(&self, issuer: &IssuerId) -> bool {
        self.entries.iter().any(|(id, _)| id == issuer)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Concatenated PEM, one anchor after another — what lands in `ca.crt`.
    pub fn to_pem(&self) -> String {
        let mut out = String::new();
        for (_, pem) in &self.entries {
            out.push_str(pem.trim_end());
            out.push('\n');
        }
        out
    }

    /// Rebuild from a Secret's `ca.crt` plus the issuer-id annotation.
    ///
    /// A count mismatch between the two is not repaired silently — it returns
    /// an empty bundle, which the state machine reads as "trust is not
    /// published", so the next reconcile republishes from the issuers
    /// themselves. Guessing which block belongs to which id would be a guess
    /// about what the fleet currently trusts.
    pub fn parse(pem: &str, annotation: Option<&str>) -> Self {
        let blocks = split_pem_blocks(pem);
        let ids: Vec<IssuerId> = annotation
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(IssuerId::new)
            .collect();
        if blocks.len() != ids.len() {
            return Self::default();
        }
        let mut bundle = Self::default();
        for (id, block) in ids.into_iter().zip(blocks) {
            bundle.insert(id, block);
        }
        bundle
    }

    fn annotation(&self) -> String {
        self.issuers()
            .iter()
            .map(IssuerId::as_str)
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Split concatenated PEM into its individual blocks, preserving each one's
/// text exactly.
fn split_pem_blocks(pem: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&str>> = None;
    for line in pem.lines() {
        if line.starts_with("-----BEGIN") {
            current = Some(vec![line]);
        } else if line.starts_with("-----END") {
            if let Some(mut lines) = current.take() {
                lines.push(line);
                blocks.push(lines.join("\n"));
            }
        } else if let Some(lines) = current.as_mut() {
            lines.push(line);
        }
    }
    blocks
}

/// What a reconcile read out of the cluster.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectedState {
    pub leaf: Option<ObservedLeaf>,
    pub bundle: TrustBundle,
}

/// Read a Secret's data back into the facts the state machine reasons about.
///
/// `data` is the decoded Secret data (the `kube` client hands out base64; the
/// caller decodes, because this module has no opinion about transport).
pub fn read_state(
    data: &BTreeMap<String, Vec<u8>>,
    annotations: &BTreeMap<String, String>,
) -> ProjectedState {
    let bundle = data
        .get(TRUST_BUNDLE_KEY)
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .map(|pem| {
            TrustBundle::parse(pem, annotations.get(TRUST_BUNDLE_ANNOTATION).map(String::as_str))
        })
        .unwrap_or_default();

    let leaf = data
        .get(CERT_KEY)
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|pem| {
            let issuer = annotations.get(LEAF_ISSUER_ANNOTATION)?;
            let identity_digest = annotations.get(IDENTITY_DIGEST_ANNOTATION)?;
            let facts = parse_leaf(pem).ok()?;
            Some(ObservedLeaf {
                issuer: IssuerId::new(issuer.clone()),
                not_before: facts.not_before,
                not_after: facts.not_after,
                fingerprint: facts.fingerprint,
                identity_digest: identity_digest.clone(),
            })
        });

    ProjectedState { leaf, bundle }
}

/// Facts read from the leaf itself.
pub struct LeafFacts {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub fingerprint: String,
}

/// Parse validity and fingerprint out of a PEM leaf.
pub fn parse_leaf(pem: &str) -> Result<LeafFacts, String> {
    let block = split_pem_blocks(pem)
        .into_iter()
        .next()
        .ok_or_else(|| "no PEM block".to_string())?;
    let der = pem_body_to_der(&block)?;
    let (_, cert) = x509_parser::parse_x509_certificate(&der)
        .map_err(|err| format!("parse certificate: {err}"))?;
    let not_before = Utc
        .timestamp_opt(cert.validity().not_before.timestamp(), 0)
        .single()
        .ok_or_else(|| "notBefore is not a representable instant".to_string())?;
    let not_after = Utc
        .timestamp_opt(cert.validity().not_after.timestamp(), 0)
        .single()
        .ok_or_else(|| "notAfter is not a representable instant".to_string())?;
    Ok(LeafFacts {
        not_before,
        not_after,
        fingerprint: hex_sha256(&der),
    })
}

fn pem_body_to_der(block: &str) -> Result<Vec<u8>, String> {
    use base64::Engine as _;
    let body: String = block
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(body.trim())
        .map_err(|err| format!("decode PEM body: {err}"))
}

/// The Secret carrying a full set of material: leaf, key, and trust bundle.
///
/// `type` is `Opaque` rather than `kubernetes.io/tls` on purpose. The TLS type
/// requires both `tls.crt` and `tls.key` to be present at all times, which
/// would make [`trust_bundle_secret`] — the bootstrap step that publishes trust
/// *before* any leaf exists — unrepresentable. The keys are what consumers read;
/// the type is what would stop the sequence from having a first step.
pub fn material_secret(
    scope: &InstanceScope,
    purpose: Purpose,
    owner: &Owner,
    material: &IssuedMaterial,
    private_key_pem: &str,
    bundle: &TrustBundle,
    identity_digest: &str,
) -> Value {
    let mut secret = base_secret(scope, purpose, owner);
    secret["metadata"]["annotations"] = json!({
        TRUST_BUNDLE_ANNOTATION: bundle.annotation(),
        LEAF_ISSUER_ANNOTATION: material.issuer.as_str(),
        IDENTITY_DIGEST_ANNOTATION: identity_digest,
    });
    secret["stringData"] = json!({
        CERT_KEY: material.certificate_pem,
        PRIVATE_KEY_KEY: private_key_pem,
        TRUST_BUNDLE_KEY: bundle.to_pem(),
    });
    secret
}

/// The Secret carrying only a trust bundle.
///
/// Applied with a merge patch so it widens `ca.crt` without touching
/// `tls.crt`/`tls.key`. That is R5's "a failed step retains the last valid
/// serving material" at the point where it is easiest to get wrong: publishing
/// the next issuer's anchor must never be able to blank the leaf that is
/// currently serving traffic.
pub fn trust_bundle_secret(
    scope: &InstanceScope,
    purpose: Purpose,
    owner: &Owner,
    bundle: &TrustBundle,
) -> Value {
    let mut secret = base_secret(scope, purpose, owner);
    secret["metadata"]["annotations"] = json!({
        TRUST_BUNDLE_ANNOTATION: bundle.annotation(),
    });
    secret["stringData"] = json!({
        TRUST_BUNDLE_KEY: bundle.to_pem(),
    });
    secret
}

fn base_secret(scope: &InstanceScope, purpose: Purpose, owner: &Owner) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": {
            "name": scope.secret_name(purpose),
            "namespace": scope.namespace,
            "labels": labels(scope, purpose),
            "ownerReferences": [owner.reference()],
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn owner() -> Owner {
        Owner {
            api_version: "lumen.dev/v1".into(),
            kind: "Lumen".into(),
            name: "lumen".into(),
            uid: "0f7d1f4e-0000-4000-8000-000000000000".into(),
        }
    }

    fn anchor(tag: &str) -> String {
        format!("-----BEGIN CERTIFICATE-----\n{tag}\n-----END CERTIFICATE-----")
    }

    #[test]
    fn a_bundle_round_trips_through_pem_and_its_annotation() {
        let mut bundle = TrustBundle::new();
        bundle.insert(IssuerId::new("pool-a"), anchor("QUFB"));
        bundle.insert(IssuerId::new("pool-b"), anchor("QkJC"));
        let parsed = TrustBundle::parse(&bundle.to_pem(), Some(&bundle.annotation()));
        assert_eq!(parsed, bundle);
        assert_eq!(
            parsed.issuers(),
            vec![IssuerId::new("pool-a"), IssuerId::new("pool-b")]
        );
    }

    #[test]
    fn a_bundle_whose_annotation_disagrees_with_its_contents_is_not_guessed_at() {
        let mut bundle = TrustBundle::new();
        bundle.insert(IssuerId::new("pool-a"), anchor("QUFB"));
        bundle.insert(IssuerId::new("pool-b"), anchor("QkJC"));
        let parsed = TrustBundle::parse(&bundle.to_pem(), Some("pool-a"));
        assert!(
            parsed.is_empty(),
            "pairing two anchors with one id would be a guess about what the fleet trusts"
        );
    }

    #[test]
    fn publishing_trust_writes_no_leaf_keys() {
        let mut bundle = TrustBundle::new();
        bundle.insert(IssuerId::new("pool-a"), anchor("QUFB"));
        let secret = trust_bundle_secret(&scope(), Purpose::Peer, &owner(), &bundle);
        let data = secret["stringData"].as_object().unwrap();
        assert_eq!(data.len(), 1);
        assert!(data.contains_key(TRUST_BUNDLE_KEY));
        assert!(
            !data.contains_key(CERT_KEY) && !data.contains_key(PRIVATE_KEY_KEY),
            "widening trust must not be able to blank the leaf that is serving traffic"
        );
    }

    #[test]
    fn secrets_are_garbage_collected_with_their_instance() {
        let secret = trust_bundle_secret(&scope(), Purpose::Peer, &owner(), &TrustBundle::new());
        let reference = &secret["metadata"]["ownerReferences"][0];
        assert_eq!(reference["controller"], json!(true));
        assert_eq!(reference["blockOwnerDeletion"], json!(true));
        assert_eq!(reference["uid"], json!("0f7d1f4e-0000-4000-8000-000000000000"));
    }

    #[test]
    fn the_secret_lands_in_the_instances_own_namespace() {
        let secret = trust_bundle_secret(&scope(), Purpose::Serving, &owner(), &TrustBundle::new());
        assert_eq!(secret["metadata"]["namespace"], json!("lumen"));
        assert_eq!(secret["metadata"]["name"], json!("lumen-serving-tls"));
    }
}
// HANDWRITE-END
