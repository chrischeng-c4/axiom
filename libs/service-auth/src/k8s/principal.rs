// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-principal" tracker="#2869" reason="Strict ServiceAccount principal parsing for delegated authentication; no generator primitive models a reserved-prefix username grammar."
//! What a reviewed token is allowed to *be*.
//!
//! `TokenReview` answering `authenticated: true` is not the end of the
//! question. A GKE cluster's authenticator will happily authenticate a Google
//! user or a Google service account against kube-apiserver and hand back
//! `alice@example.com` or `svc@project.iam.gserviceaccount.com`. Those are
//! real, verified identities — they are simply not the kind of identity a
//! delegating service accepts, because accepting them would make the service a
//! second Google identity verifier with its own parallel authorization story.
//!
//! So the reviewed username has to *parse*, strictly, as
//! `system:serviceaccount:<namespace>:<name>` before anything else happens.
//! "Strictly" is doing real work here:
//!
//! - `system:serviceaccount:a:b:c` is not a ServiceAccount, it is a username
//!   with the right prefix and the wrong shape. Kubernetes would never mint it,
//!   so something else did.
//! - `system:serviceaccount::name` and `system:serviceaccount:ns:` name nothing.
//! - `system:anonymous` and the `system:unauthenticated` group are the
//!   apiserver's way of saying "nobody", which is never a caller.
//! - A namespace and a ServiceAccount name are DNS-1123 *labels*. Checking that
//!   is not pedantry: it is what makes the parse total, because a label cannot
//!   contain a colon, so the split above can never be ambiguous.

use std::collections::BTreeMap;
use std::fmt;

/// The `system:serviceaccount:` username prefix, reserved by Kubernetes.
pub const SERVICE_ACCOUNT_PREFIX: &str = "system:serviceaccount:";

/// Usernames and groups the apiserver uses for "no credential".
const ANONYMOUS_USERNAME: &str = "system:anonymous";
const UNAUTHENTICATED_GROUP: &str = "system:unauthenticated";

/// The identity `TokenReview` returned, carried whole.
///
/// Every field survives into the `SubjectAccessReview` that follows: RBAC can
/// bind by group and policy can read `extra`, so dropping any of it here would
/// silently narrow what the cluster administrator is able to express.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ReviewedIdentity {
    pub username: String,
    pub uid: String,
    pub groups: Vec<String>,
    pub extra: BTreeMap<String, Vec<String>>,
}

/// A namespaced ServiceAccount, parsed out of a reviewed username.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceAccountRef {
    pub namespace: String,
    pub name: String,
}

impl fmt::Display for ServiceAccountRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{SERVICE_ACCOUNT_PREFIX}{}:{}",
            self.namespace, self.name
        )
    }
}

/// A verified caller: the ServiceAccount it is, plus the identity the
/// apiserver described it with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAccountPrincipal {
    pub service_account: ServiceAccountRef,
    pub identity: ReviewedIdentity,
}

impl ServiceAccountPrincipal {
    pub fn namespace(&self) -> &str {
        &self.service_account.namespace
    }

    pub fn name(&self) -> &str {
        &self.service_account.name
    }

    /// The canonical `system:serviceaccount:<ns>:<name>` username. Built from
    /// the parsed parts rather than echoed from the response, so a principal
    /// can never carry a username its own parse would reject.
    pub fn username(&self) -> String {
        self.service_account.to_string()
    }
}

/// Why a reviewed identity is not an acceptable caller.
///
/// Each variant is a *classification*, not an echo of the value: these are
/// what gets logged. A rejected username is frequently a person's email
/// address, and an audit line that reprints it turns every probe of the API
/// into a way to write arbitrary identities into the service's logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalRejection {
    /// `status.authenticated` was false, or absent.
    NotAuthenticated,
    /// Authenticated, but the response named no user.
    MissingUsername,
    /// `system:anonymous`, or a member of `system:unauthenticated`.
    Anonymous,
    /// A verified identity that is not a ServiceAccount at all — the Google
    /// user and Google service-account case.
    NotAServiceAccount,
    /// The `system:serviceaccount:` prefix with a shape Kubernetes does not
    /// mint: wrong segment count, an empty segment, or a segment that is not a
    /// DNS-1123 label.
    MalformedServiceAccount,
}

impl PrincipalRejection {
    /// A stable, credential-free token for logs and metrics.
    pub fn reason(self) -> &'static str {
        match self {
            Self::NotAuthenticated => "not_authenticated",
            Self::MissingUsername => "missing_username",
            Self::Anonymous => "anonymous",
            Self::NotAServiceAccount => "not_a_service_account",
            Self::MalformedServiceAccount => "malformed_service_account",
        }
    }
}

impl fmt::Display for PrincipalRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason())
    }
}

impl ServiceAccountRef {
    /// Parse a reviewed username, accepting only what Kubernetes itself mints.
    pub fn parse(username: &str) -> Result<Self, PrincipalRejection> {
        if username.is_empty() {
            return Err(PrincipalRejection::MissingUsername);
        }
        if username == ANONYMOUS_USERNAME {
            return Err(PrincipalRejection::Anonymous);
        }
        let Some(rest) = username.strip_prefix(SERVICE_ACCOUNT_PREFIX) else {
            return Err(PrincipalRejection::NotAServiceAccount);
        };

        // Exactly two segments. `splitn` would silently accept a third by
        // folding it into the name, which is the whole failure this guards.
        let mut segments = rest.split(':');
        let (Some(namespace), Some(name), None) =
            (segments.next(), segments.next(), segments.next())
        else {
            return Err(PrincipalRejection::MalformedServiceAccount);
        };
        if !is_dns1123_label(namespace) || !is_dns1123_label(name) {
            return Err(PrincipalRejection::MalformedServiceAccount);
        }

        Ok(Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
        })
    }
}

impl ServiceAccountPrincipal {
    /// Promote a reviewed identity to a caller, or say why it cannot be one.
    ///
    /// `authenticated` is passed separately because a `TokenReview` that says
    /// `authenticated: false` may still carry a partially-filled user object,
    /// and reading the username first would treat a rejection as an identity.
    pub fn from_review(
        authenticated: bool,
        identity: ReviewedIdentity,
    ) -> Result<Self, PrincipalRejection> {
        if !authenticated {
            return Err(PrincipalRejection::NotAuthenticated);
        }
        if identity
            .groups
            .iter()
            .any(|group| group == UNAUTHENTICATED_GROUP)
        {
            return Err(PrincipalRejection::Anonymous);
        }
        let service_account = ServiceAccountRef::parse(&identity.username)?;
        Ok(Self {
            service_account,
            identity,
        })
    }
}

/// DNS-1123 label: 1–63 chars, lowercase alphanumeric or `-`, and the first
/// and last characters must be alphanumeric.
fn is_dns1123_label(value: &str) -> bool {
    if value.is_empty() || value.len() > 63 {
        return false;
    }
    let bytes = value.as_bytes();
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    if !alnum(bytes[0]) || !alnum(bytes[bytes.len() - 1]) {
        return false;
    }
    bytes.iter().all(|&b| alnum(b) || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(username: &str) -> ReviewedIdentity {
        ReviewedIdentity {
            username: username.into(),
            ..Default::default()
        }
    }

    #[test]
    fn a_kubernetes_service_account_username_parses_into_its_two_parts() {
        let parsed = ServiceAccountRef::parse("system:serviceaccount:tenant-a:reader").unwrap();
        assert_eq!(parsed.namespace, "tenant-a");
        assert_eq!(parsed.name, "reader");
        assert_eq!(
            parsed.to_string(),
            "system:serviceaccount:tenant-a:reader",
            "the canonical username must round-trip"
        );
    }

    /// The rejection this whole module exists for: a verified human or a
    /// verified Google service account is still not a ServiceAccount.
    #[test]
    fn a_verified_google_identity_is_not_a_service_account() {
        for username in [
            "alice@example.com",
            "deployer@my-project.iam.gserviceaccount.com",
            "accounts.google.com:117302456789012345678",
        ] {
            assert_eq!(
                ServiceAccountRef::parse(username),
                Err(PrincipalRejection::NotAServiceAccount),
                "{username} must not parse as a ServiceAccount"
            );
        }
    }

    /// The reserved prefix is not a password. Everything after it still has to
    /// be the shape Kubernetes mints.
    #[test]
    fn the_reserved_prefix_with_a_wrong_shape_is_malformed_not_accepted() {
        for username in [
            "system:serviceaccount:tenant-a:reader:extra",
            "system:serviceaccount:tenant-a",
            "system:serviceaccount::reader",
            "system:serviceaccount:tenant-a:",
            "system:serviceaccount::",
            "system:serviceaccount:Tenant-A:reader",
            "system:serviceaccount:tenant-a:read er",
            "system:serviceaccount:-tenant:reader",
            "system:serviceaccount:tenant-:reader",
            "system:serviceaccount:tenant.a:reader",
        ] {
            assert_eq!(
                ServiceAccountRef::parse(username),
                Err(PrincipalRejection::MalformedServiceAccount),
                "{username} must be rejected as malformed"
            );
        }
    }

    #[test]
    fn a_namespace_or_name_longer_than_a_dns_label_is_malformed() {
        let long = "a".repeat(64);
        assert_eq!(
            ServiceAccountRef::parse(&format!("system:serviceaccount:{long}:reader")),
            Err(PrincipalRejection::MalformedServiceAccount)
        );
        assert_eq!(
            ServiceAccountRef::parse(&format!("system:serviceaccount:ns:{long}")),
            Err(PrincipalRejection::MalformedServiceAccount)
        );
        let at_limit = "a".repeat(63);
        assert!(
            ServiceAccountRef::parse(&format!("system:serviceaccount:{at_limit}:reader")).is_ok(),
            "63 characters is a legal label"
        );
    }

    #[test]
    fn anonymous_and_empty_usernames_are_their_own_rejections() {
        assert_eq!(
            ServiceAccountRef::parse("system:anonymous"),
            Err(PrincipalRejection::Anonymous)
        );
        assert_eq!(
            ServiceAccountRef::parse(""),
            Err(PrincipalRejection::MissingUsername)
        );
    }

    /// `authenticated: false` is checked before the username, so a rejection
    /// carrying a leftover user object cannot be read as an identity.
    #[test]
    fn an_unauthenticated_review_is_rejected_before_its_username_is_read() {
        let rejection = ServiceAccountPrincipal::from_review(
            false,
            identity("system:serviceaccount:tenant-a:reader"),
        )
        .unwrap_err();
        assert_eq!(rejection, PrincipalRejection::NotAuthenticated);
    }

    /// Membership of `system:unauthenticated` outranks a plausible username:
    /// an authenticator that produced both is describing "nobody".
    #[test]
    fn the_unauthenticated_group_is_rejected_even_with_a_service_account_username() {
        let mut who = identity("system:serviceaccount:tenant-a:reader");
        who.groups = vec!["system:unauthenticated".into()];
        assert_eq!(
            ServiceAccountPrincipal::from_review(true, who).unwrap_err(),
            PrincipalRejection::Anonymous
        );
    }

    /// R4: nothing the apiserver said about the caller is dropped, because the
    /// authorization step is entitled to use all of it.
    #[test]
    fn the_full_reviewed_identity_survives_into_the_principal() {
        let who = ReviewedIdentity {
            username: "system:serviceaccount:tenant-a:reader".into(),
            uid: "1a2b3c".into(),
            groups: vec![
                "system:serviceaccounts".into(),
                "system:serviceaccounts:tenant-a".into(),
            ],
            extra: BTreeMap::from([(
                "authentication.kubernetes.io/pod-name".to_string(),
                vec!["client-0".to_string()],
            )]),
        };
        let principal = ServiceAccountPrincipal::from_review(true, who.clone()).unwrap();
        assert_eq!(principal.identity, who);
        assert_eq!(principal.namespace(), "tenant-a");
        assert_eq!(principal.name(), "reader");
        assert_eq!(principal.username(), who.username);
    }

    #[test]
    fn every_rejection_has_a_stable_credential_free_reason() {
        for (rejection, expected) in [
            (PrincipalRejection::NotAuthenticated, "not_authenticated"),
            (PrincipalRejection::MissingUsername, "missing_username"),
            (PrincipalRejection::Anonymous, "anonymous"),
            (
                PrincipalRejection::NotAServiceAccount,
                "not_a_service_account",
            ),
            (
                PrincipalRejection::MalformedServiceAccount,
                "malformed_service_account",
            ),
        ] {
            assert_eq!(rejection.reason(), expected);
            assert_eq!(rejection.to_string(), expected);
        }
    }
}
// HANDWRITE-END
