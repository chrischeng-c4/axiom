// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-profile" tracker="#3110" reason="Own what a service is allowed to ask a certificate for -- purpose, identity, and lifetime bounds -- as one validated value, so a profile that names another namespace's identity cannot be constructed rather than merely being rejected later."
//! What a service asks a certificate *for*.
//!
//! A profile is the service-specific half of the lifecycle: which names the
//! leaf must carry, which direction of TLS it is for, and how long it may
//! live. Everything else — issuing it, projecting it, renewing it, rotating
//! the issuer under it — is generic and lives in the sibling modules.
//!
//! The reason profiles are validated against an [`InstanceScope`] rather than
//! trusted is R7. A certificate is an authorization artifact: whoever can name
//! the identity on it can obtain that identity. If a profile could name
//! `lumen.other-tenant.svc.cluster.local`, then a bug in one instance's
//! reconcile loop is a cross-tenant impersonation, not a misconfiguration.
//! Checking here — before a key is generated and before any Secret is read —
//! is what makes AC4 a property of construction rather than of code review.

use std::fmt;
use std::time::Duration;

/// Which direction of TLS a leaf is for.
///
/// This is not cosmetic. It selects the extended key usages, and the two
/// answers are genuinely different: a serving leaf that also carried
/// `clientAuth` could be replayed by whoever holds it to authenticate *as*
/// the service to its own peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Purpose {
    /// Client-facing TLS. Answers to Service DNS names; never authenticates
    /// outward.
    Serving,
    /// Peer mTLS. Both ends of a Raft link present one of these to each other,
    /// so it is server and client at once.
    Peer,
}

impl Purpose {
    /// Stable lowercase token used in Secret names, metrics, and conditions.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serving => "serving",
            Self::Peer => "peer",
        }
    }

    /// The extended key usages a leaf of this purpose may carry — exactly, not
    /// at least. `Serving` deliberately omits `clientAuth`.
    pub fn extended_key_usages(self) -> &'static [ExtendedUsage] {
        match self {
            Self::Serving => &[ExtendedUsage::ServerAuth],
            Self::Peer => &[ExtendedUsage::ServerAuth, ExtendedUsage::ClientAuth],
        }
    }
}

impl fmt::Display for Purpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The extended key usages this lifecycle can express. There is no `Any`, and
/// no code-signing or OCSP variant: a leaf minted here is for one Kubernetes
/// service talking to another, and a wider enum would be a wider blast radius
/// for a typo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExtendedUsage {
    ServerAuth,
    ClientAuth,
}

impl ExtendedUsage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ServerAuth => "serverAuth",
            Self::ClientAuth => "clientAuth",
        }
    }
}

/// One Lumen instance, in one namespace. Every read, write, owner reference,
/// and issuance request in this lifecycle is scoped by one of these.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceScope {
    pub namespace: String,
    pub instance: String,
    /// The environment's single SPIFFE trust domain (#3109 R5). Identities are
    /// scoped inside it; it is not per-instance.
    pub trust_domain: String,
}

impl InstanceScope {
    pub fn new(
        namespace: impl Into<String>,
        instance: impl Into<String>,
        trust_domain: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            instance: instance.into(),
            trust_domain: trust_domain.into(),
        }
    }

    /// Where this instance's material for `purpose` lives. Derived, never
    /// supplied: a caller-chosen Secret name is a caller-chosen place to write,
    /// and this lifecycle owns what it writes.
    pub fn secret_name(&self, purpose: Purpose) -> String {
        format!("{}-{}-tls", self.instance, purpose.as_str())
    }

    /// The SPIFFE identity prefix every leaf in this scope must sit under.
    pub fn spiffe_prefix(&self) -> String {
        format!("spiffe://{}/ns/{}/", self.trust_domain, self.namespace)
    }

    /// True when `other` is the same instance in the same namespace of the same
    /// trust domain.
    pub fn covers(&self, other: &InstanceScope) -> bool {
        self == other
    }
}

/// Names a leaf must carry.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CertificateIdentity {
    /// Kubernetes-internal DNS names. Public names are not expressible: the
    /// issuing pool would refuse them (#3109 AC3), and this refuses them first.
    pub dns_names: Vec<String>,
    /// SPIFFE URI SAN. Required for peer leaves — it is what a peer actually
    /// authorizes against, since DNS alone cannot distinguish two members of
    /// the same headless Service from each other's point of view.
    pub spiffe_uri: Option<String>,
}

/// A validated request shape: purpose, names, and lifetime bounds.
///
/// Construct with [`CertificateProfile::new`], which is fallible. There is no
/// public field-literal path on purpose — an unvalidated profile is exactly the
/// value this module exists to prevent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateProfile {
    /// The scope this profile was validated against.
    ///
    /// Retained rather than merely consulted at construction: "validated" has to
    /// stay checkable downstream, and a reconciler that cannot ask which
    /// instance a profile belongs to has no way to refuse one that belongs to
    /// another (R7).
    scope: InstanceScope,
    purpose: Purpose,
    common_name: String,
    identity: CertificateIdentity,
    lifetime: Duration,
    renew_before: Duration,
    renew_jitter: Duration,
}

/// Why a profile was refused. Every variant names the offending value: an
/// operator reading this in a condition should not have to diff two lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    NoNames,
    ForeignDnsName {
        name: String,
        namespace: String,
    },
    PublicDnsName {
        name: String,
    },
    ForeignSpiffeUri {
        uri: String,
        expected_prefix: String,
    },
    PeerNeedsSpiffeUri,
    LifetimeOutOfBounds {
        seconds: u64,
    },
    RenewWindowTooWide {
        renew_before_secs: u64,
        lifetime_secs: u64,
    },
    RenewWindowTooNarrow {
        renew_before_secs: u64,
    },
    JitterExceedsWindow {
        jitter_secs: u64,
        renew_before_secs: u64,
    },
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoNames => write!(f, "a certificate profile must request at least one DNS name"),
            Self::ForeignDnsName { name, namespace } => write!(
                f,
                "DNS name {name} is not inside namespace {namespace}; one instance may not \
                 request an identity belonging to another"
            ),
            Self::PublicDnsName { name } => write!(
                f,
                "DNS name {name} is not a Kubernetes-internal name; this trust domain does not \
                 issue publicly resolvable identities"
            ),
            Self::ForeignSpiffeUri {
                uri,
                expected_prefix,
            } => write!(
                f,
                "SPIFFE URI {uri} is outside this instance's scope; it must begin with \
                 {expected_prefix}"
            ),
            Self::PeerNeedsSpiffeUri => write!(
                f,
                "a peer profile must carry a SPIFFE URI: DNS alone cannot distinguish two \
                 members of the same headless Service"
            ),
            Self::LifetimeOutOfBounds { seconds } => write!(
                f,
                "leaf lifetime {seconds}s is outside {MIN_LIFETIME_SECS}s..{MAX_LIFETIME_SECS}s; \
                 shorter cannot survive a controller outage, longer stops being short-lived"
            ),
            Self::RenewWindowTooWide {
                renew_before_secs,
                lifetime_secs,
            } => write!(
                f,
                "renew_before {renew_before_secs}s is not shorter than the {lifetime_secs}s \
                 lifetime; a leaf due for renewal the moment it is issued renews forever"
            ),
            Self::RenewWindowTooNarrow { renew_before_secs } => write!(
                f,
                "renew_before {renew_before_secs}s leaves no room to retry a failed issuance \
                 before the current leaf expires; the floor is {MIN_RENEW_BEFORE_SECS}s"
            ),
            Self::JitterExceedsWindow {
                jitter_secs,
                renew_before_secs,
            } => write!(
                f,
                "renew jitter {jitter_secs}s exceeds the {renew_before_secs}s renewal window; \
                 spreading renewals must not push one past expiry"
            ),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Floor and ceiling on a leaf's lifetime, mirroring the issuing pool's own
/// bounds (#3109 `max_leaf_lifetime_seconds`). Stated here too because the
/// controller should refuse an impossible profile locally rather than learn it
/// from a rejected CSR.
pub const MIN_LIFETIME_SECS: u64 = 300;
pub const MAX_LIFETIME_SECS: u64 = 604_800;
/// The renewal window must leave room for several failed attempts. Ten minutes
/// is roughly a dozen retries at the backoff ceiling.
pub const MIN_RENEW_BEFORE_SECS: u64 = 600;

/// Kubernetes-internal DNS suffixes. Same list as the issuing pool's default
/// `allowed_dns_suffixes` (#3109) — kept in sync by intent, checked here so a
/// profile fails before a CSR is submitted rather than after.
const CLUSTER_SUFFIXES: [&str; 2] = [".svc.cluster.local", ".svc"];

impl CertificateProfile {
    /// Validate a profile against the scope that will own it.
    pub fn new(
        scope: &InstanceScope,
        purpose: Purpose,
        common_name: impl Into<String>,
        identity: CertificateIdentity,
        lifetime: Duration,
        renew_before: Duration,
        renew_jitter: Duration,
    ) -> Result<Self, ProfileError> {
        if identity.dns_names.is_empty() {
            return Err(ProfileError::NoNames);
        }
        for name in &identity.dns_names {
            if !CLUSTER_SUFFIXES
                .iter()
                .any(|suffix| name.ends_with(suffix))
            {
                return Err(ProfileError::PublicDnsName { name: name.clone() });
            }
            // `.<namespace>.svc` is the segment Kubernetes itself uses to
            // separate tenants in DNS, so it is the segment worth checking. A
            // suffix match on the namespace alone would accept
            // `evil.lumen-prod.svc` for namespace `prod`.
            let namespaced = format!(".{}.svc", scope.namespace);
            if !name.contains(&namespaced) {
                return Err(ProfileError::ForeignDnsName {
                    name: name.clone(),
                    namespace: scope.namespace.clone(),
                });
            }
        }
        match (&identity.spiffe_uri, purpose) {
            (None, Purpose::Peer) => return Err(ProfileError::PeerNeedsSpiffeUri),
            (Some(uri), _) => {
                let prefix = scope.spiffe_prefix();
                if !uri.starts_with(&prefix) {
                    return Err(ProfileError::ForeignSpiffeUri {
                        uri: uri.clone(),
                        expected_prefix: prefix,
                    });
                }
            }
            (None, Purpose::Serving) => {}
        }
        let lifetime_secs = lifetime.as_secs();
        if !(MIN_LIFETIME_SECS..=MAX_LIFETIME_SECS).contains(&lifetime_secs) {
            return Err(ProfileError::LifetimeOutOfBounds {
                seconds: lifetime_secs,
            });
        }
        let renew_secs = renew_before.as_secs();
        if renew_secs < MIN_RENEW_BEFORE_SECS {
            return Err(ProfileError::RenewWindowTooNarrow {
                renew_before_secs: renew_secs,
            });
        }
        if renew_secs >= lifetime_secs {
            return Err(ProfileError::RenewWindowTooWide {
                renew_before_secs: renew_secs,
                lifetime_secs,
            });
        }
        if renew_jitter > renew_before {
            return Err(ProfileError::JitterExceedsWindow {
                jitter_secs: renew_jitter.as_secs(),
                renew_before_secs: renew_secs,
            });
        }
        Ok(Self {
            scope: scope.clone(),
            purpose,
            common_name: common_name.into(),
            identity,
            lifetime,
            renew_before,
            renew_jitter,
        })
    }

    /// The scope this profile was validated against.
    pub fn scope(&self) -> &InstanceScope {
        &self.scope
    }

    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    pub fn common_name(&self) -> &str {
        &self.common_name
    }

    pub fn identity(&self) -> &CertificateIdentity {
        &self.identity
    }

    pub fn lifetime(&self) -> Duration {
        self.lifetime
    }

    pub fn renew_before(&self) -> Duration {
        self.renew_before
    }

    pub fn renew_jitter(&self) -> Duration {
        self.renew_jitter
    }

    pub fn extended_key_usages(&self) -> &'static [ExtendedUsage] {
        self.purpose.extended_key_usages()
    }

    /// A stable digest of everything a reissue would change: purpose, names,
    /// and usages.
    ///
    /// This is what lets a restarted controller decide "the leaf on disk is
    /// still the one this profile asks for" without keeping any memory of
    /// having issued it (R4). It covers the certified content only — lifetime
    /// is not in it, because changing the renewal cadence is not a reason to
    /// throw away a valid identity.
    pub fn identity_digest(&self) -> String {
        let mut parts = vec![format!("purpose={}", self.purpose.as_str())];
        parts.push(format!("cn={}", self.common_name));
        let mut dns = self.identity.dns_names.clone();
        dns.sort();
        parts.push(format!("dns={}", dns.join(",")));
        parts.push(format!(
            "uri={}",
            self.identity.spiffe_uri.as_deref().unwrap_or("")
        ));
        let usages: Vec<&str> = self
            .extended_key_usages()
            .iter()
            .map(|u| u.as_str())
            .collect();
        parts.push(format!("eku={}", usages.join(",")));
        crate::certificate::digest::hex_sha256(parts.join("|").as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn serving_identity() -> CertificateIdentity {
        CertificateIdentity {
            dns_names: vec!["lumen.lumen.svc.cluster.local".into(), "lumen.lumen.svc".into()],
            spiffe_uri: None,
        }
    }

    fn build(
        purpose: Purpose,
        identity: CertificateIdentity,
    ) -> Result<CertificateProfile, ProfileError> {
        CertificateProfile::new(
            &scope(),
            purpose,
            "lumen.lumen.svc.cluster.local",
            identity,
            Duration::from_secs(86_400),
            Duration::from_secs(21_600),
            Duration::from_secs(1_800),
        )
    }

    #[test]
    fn serving_leaves_do_not_carry_client_auth() {
        let profile = build(Purpose::Serving, serving_identity()).unwrap();
        assert_eq!(
            profile.extended_key_usages(),
            &[ExtendedUsage::ServerAuth],
            "a serving leaf that could also authenticate outward is a credential its holder \
             can replay against the service's own peers"
        );
    }

    #[test]
    fn peer_leaves_carry_both_directions() {
        let identity = CertificateIdentity {
            dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
            spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
        };
        let profile = build(Purpose::Peer, identity).unwrap();
        assert_eq!(
            profile.extended_key_usages(),
            &[ExtendedUsage::ServerAuth, ExtendedUsage::ClientAuth]
        );
    }

    #[test]
    fn a_peer_profile_without_a_spiffe_uri_is_refused() {
        let identity = CertificateIdentity {
            dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
            spiffe_uri: None,
        };
        assert_eq!(
            build(Purpose::Peer, identity),
            Err(ProfileError::PeerNeedsSpiffeUri)
        );
    }

    #[test]
    fn another_namespaces_dns_name_is_refused() {
        let identity = CertificateIdentity {
            dns_names: vec!["lumen.other-tenant.svc.cluster.local".into()],
            spiffe_uri: None,
        };
        assert!(matches!(
            build(Purpose::Serving, identity),
            Err(ProfileError::ForeignDnsName { .. })
        ));
    }

    #[test]
    fn a_namespace_prefix_is_not_a_namespace_match() {
        // `lumen-prod` starts with `lumen`, and a suffix or prefix check would
        // wave this through for the `lumen` namespace.
        let identity = CertificateIdentity {
            dns_names: vec!["lumen.lumen-prod.svc.cluster.local".into()],
            spiffe_uri: None,
        };
        assert!(matches!(
            build(Purpose::Serving, identity),
            Err(ProfileError::ForeignDnsName { .. })
        ));
    }

    #[test]
    fn a_public_dns_name_is_refused() {
        let identity = CertificateIdentity {
            dns_names: vec!["lumen.example.com".into()],
            spiffe_uri: None,
        };
        assert!(matches!(
            build(Purpose::Serving, identity),
            Err(ProfileError::PublicDnsName { .. })
        ));
    }

    #[test]
    fn another_namespaces_spiffe_identity_is_refused() {
        let identity = CertificateIdentity {
            dns_names: vec!["lumen.lumen.svc.cluster.local".into()],
            spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/other-tenant/sa/lumen".into()),
        };
        assert!(matches!(
            build(Purpose::Serving, identity),
            Err(ProfileError::ForeignSpiffeUri { .. })
        ));
    }

    #[test]
    fn a_renewal_window_with_no_room_to_retry_is_refused() {
        let err = CertificateProfile::new(
            &scope(),
            Purpose::Serving,
            "lumen.lumen.svc.cluster.local",
            serving_identity(),
            Duration::from_secs(86_400),
            Duration::from_secs(60),
            Duration::ZERO,
        );
        assert!(matches!(
            err,
            Err(ProfileError::RenewWindowTooNarrow { .. })
        ));
    }

    #[test]
    fn identity_digest_ignores_dns_name_order_but_not_content() {
        let a = build(Purpose::Serving, serving_identity()).unwrap();
        let mut reordered = serving_identity();
        reordered.dns_names.reverse();
        let b = build(Purpose::Serving, reordered).unwrap();
        assert_eq!(
            a.identity_digest(),
            b.identity_digest(),
            "reordering the same names is not a reissue"
        );

        let extra = CertificateIdentity {
            dns_names: vec![
                "lumen.lumen.svc.cluster.local".into(),
                "lumen.lumen.svc".into(),
                "lumen-read.lumen.svc.cluster.local".into(),
            ],
            spiffe_uri: None,
        };
        let c = build(Purpose::Serving, extra).unwrap();
        assert_ne!(a.identity_digest(), c.identity_digest());
    }

    #[test]
    fn secret_names_are_derived_from_the_scope_not_supplied() {
        let scope = scope();
        assert_eq!(scope.secret_name(Purpose::Serving), "lumen-serving-tls");
        assert_eq!(scope.secret_name(Purpose::Peer), "lumen-peer-tls");
    }
}
// HANDWRITE-END
