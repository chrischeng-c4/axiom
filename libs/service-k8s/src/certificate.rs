// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-facade" tracker="#3110" reason="Name the certificate lifecycle's public surface and the layering that keeps its correctness independent of any particular certificate authority."
//! Short-lived certificates, reconciled.
//!
//! Serving TLS and Raft peer mTLS both need a leaf that expires in hours, gets
//! replaced before it does, and is signed by a CA the rest of the fleet already
//! trusts. That is a control loop, and this module is it: the shared, service-
//! neutral half. A service supplies a [`profile::CertificateProfile`] naming
//! what it wants; everything about *when* and *in what order* lives here.
//!
//! ```text
//!   profile   what identity, for how long, renewed how early
//!   state     given what the cluster shows and what time it is, do what
//!   issuer    the boundary: a CSR goes out, a leaf comes back
//!   projection where material lands, and what is read back from where
//!   status    what any of this is allowed to say about itself
//!   reconcile the seam -- authorize, read, decide, write
//! ```
//!
//! The layering has one rule, and it is worth stating because it is what makes
//! the whole thing testable: **nothing above `issuer` knows which CA is in
//! use.** [`ephemeral::EphemeralIssuer`] signs in-process and
//! [`cas::CasIssuer`] calls GCP CA Service; the state machine cannot tell them
//! apart, so every property about renewal timing, rotation ordering, and expiry
//! is checked in a unit test rather than in a cloud gate somebody remembers to
//! run.
//!
//! ### What this deliberately does not do
//!
//! It does not apply Terraform, restart Pods, or edit a Deployment. Renewal is
//! a Secret write and nothing else — the trust foundation is provisioned once
//! (#3109) and the runtime picks up new material without a restart (#3112). A
//! renewal path that shelled out to infrastructure would make certificate
//! expiry an operational event, which is the thing short-lived certificates
//! exist to stop being.

pub mod digest;
pub mod ephemeral;
pub mod issuer;
pub mod kubernetes_store;
pub mod profile;
pub mod projection;
pub mod reconcile;
pub mod state;
pub mod status;

#[cfg(feature = "gcp-cas-client")]
pub mod cas;

pub use ephemeral::EphemeralIssuer;
pub use issuer::{IssuanceRequest, IssuedMaterial, Issuer, IssuerError, IssuerId, PrivateKey};
pub use kubernetes_store::{
    classify_kube_error, prepare_ssa_patch, KubernetesSecretStore, KubernetesStoreError,
    FIELD_MANAGER, RBAC_VERBS, REQUIRED_RBAC_VERBS,
};
pub use profile::{
    CertificateIdentity, CertificateProfile, ExtendedUsage, InstanceScope, ProfileError, Purpose,
};
pub use projection::{Owner, ProjectedState, TrustBundle};
pub use reconcile::{
    MemoryStore, Outcome, ReconcileError, Reconciler, RuntimeReport, SecretStore, StoreError,
    StoreErrorKind, StoredSecret,
};
pub use state::{
    next_action, renew_at, retry_after, Action, Desired, IssueReason, Observed, ObservedLeaf,
};
pub use status::{redact, CertificateFacts, READY_CONDITION, ROTATING_CONDITION};

#[cfg(feature = "gcp-cas-client")]
pub use cas::{
    AccessTokenSource, CaPool, CasIssuer, GkeMetadataTokenSource, WorkloadIdentityTokenSource,
};
// HANDWRITE-END
