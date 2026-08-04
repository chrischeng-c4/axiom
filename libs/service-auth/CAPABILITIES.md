# Service Auth Capabilities

## Brief

`service-auth` is the shared request-authentication and authorization library
for HTTP services. It turns a credential presented on a request into a typed
principal, decides whether that principal may perform a named action on a named
resource, and keeps both answers correct while credentials rotate underneath a
running process. It is not a token issuer, not a certificate authority, and not
a transport: callers own where credentials come from and who listens on the
socket.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `service-auth` fundamentally does: decide a
  principal from a credential, and decide an authorization outcome from that
  principal. There are two independent decision sources — a locally held role
  map, and a delegated Kubernetes authority — and both must refuse by default.
- **Non-Core Features** keep those decisions correct in a running service while
  the credential registry is replaced and while operators need to observe what
  was decided. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Static Role Map Authorization | - | implemented | verified | smoke | ready | core; bearer or identity credential to typed principal, then a per-resource role decision with an explicit open mode |
| Delegated Kubernetes Authorization | - | implemented | verified | conformance | ready | core; the cluster is the sole authority on identity and permission, and no failure of that authority produces an allow |
| Credential Reload & Redacted Audit | - | implemented | verified | conformance | ready | non-core; validated all-or-nothing registry replacement plus an event schema that cannot carry a credential |

### Core Features

#### Static Role Map Authorization

ID: static-role-map-authorization
Root WI: -
Status: verified
Type: Security
Feature Class: core
Required Verification: smoke
Promise:
Decide, from a locally held registry, whether a request that carries a given credential may perform a named action on a named resource, and say which credential namespace the answer came from. A caller never has to interpret role strings itself, and a registry that cannot be interpreted unambiguously is refused at load time rather than half-applied at request time.
Surfaces:
- Rust API: `service_auth::Registry` / `service_auth::Registry::parse` - the registry document and the single interpretation point for its two credential namespaces.
- Rust API: `service_auth::RoleMapPrincipal::ensure` - the per-resource authorization decision, including the wildcard resource and the explicit open mode.
- Rust API: `service_auth::StaticRoleMapVerifier` - the `Verifier` implementation that binds a presented credential to a principal.
EC Dimensions:
- behavior: `cargo test -p service-auth --lib role_map` - role ordering, per-resource and wildcard resolution, open mode, registry interpretation, and merge collision handling are decided from the registry document alone.
- security: `cargo test -p service-auth --lib role_map` - the secret-keyed and identity-keyed namespaces never resolve one another, reserved subjects are refused, and a required-but-absent credential is never treated as an open request.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Role ordering and resource resolution | change | - | implemented | verified | smoke | `cargo test -p service-auth --lib role_map`; a role covers exactly itself and every role below it, resource lookup prefers an exact resource entry over the wildcard entry, and a resource named by neither is denied |
| Registry interpretation and merge | change | - | implemented | verified | smoke | `cargo test -p service-auth --lib role_map`; a document is interpreted as namespaced or flat by a stated rule rather than by guessing, a duplicate key inside one namespace is a load-time error, and a merge that fails leaves no partial registry behind |
| Credential namespace separation | change | - | implemented | verified | smoke | `cargo test -p service-auth --lib role_map`; a secret presented as a bearer token is resolved only against the secret-keyed namespace and a reviewed public identity only against the identity-keyed namespace, so a secret that happens to look like an identity cannot borrow that identity's grants |
| Reserved subject refusal | change | - | implemented | verified | smoke | `cargo test -p service-auth --lib role_map`; a registry that grants a reserved subject name is refused at load time and names the offending key, subject, and reason |

#### Delegated Kubernetes Authorization

ID: delegated-kubernetes-authorization
Root WI: -
Status: verified
Type: Security
Feature Class: core
Required Verification: conformance
Promise:
When authorization is delegated to a Kubernetes cluster, the cluster is the sole authority on both who the caller is and what the caller may do, and this library adds no local exception to either answer. The delegation is cacheable, and the cache has a stated upper bound on how long a revoked credential can still be honored.
Surfaces:
- Rust API: `service_auth::k8s::ServiceAccountPrincipal::from_review` - the single point that turns a review result into an admitted service-account principal or a typed rejection.
- Rust API: `service_auth::k8s::DelegatedAuth` - authenticate and authorize against the delegated authority, including the cache and its stale window.
- Rust API: `service_auth::k8s::ReviewBackend` - the port through which the cluster's token review and access review answers arrive.
EC Dimensions:
- behavior: `cargo test -p service-auth --lib k8s` - the judgement order, the deny-outranks-allow rule, the cache hit/miss/stale classification, and the revocation bound are decided by stated rules.
- security: `cargo test -p service-auth --lib k8s` - only a well-formed service-account identity with an intersecting audience is admitted, and no unavailability, malformed response, or evaluation error of the delegated authority yields an allow.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Identity admission | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib k8s`; only a username of the exact service-account form whose namespace and name are both valid DNS-1123 labels is admitted, and an unauthenticated review, an anonymous username, a non-service-account username, and a malformed one each carry their own stable rejection reason |
| Judgement order and audience binding | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib k8s`; a credential is judged authenticated first, then for audience intersection, then for identity shape, a configuration that names no audience is refused at construction, and a token minted for a different audience is rejected even when its identity is admissible |
| Deny outranks allow | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib k8s`; an access review that is both allowed and denied is an authorization failure, and an evaluation error is treated as a malformed answer rather than as either an allow or a silent pass |
| Cache classification and revocation bound | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib k8s`; a lookup is classified hit, miss, or stale, an expired entry is returned only through the explicit stale path and only while the delegated authority is unreachable, and the worst-case time a revoked credential can still be honored equals the allow lifetime plus the stale window |
| Outage never allows | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib k8s`; when the delegated authority is unreachable and no usable cached decision exists, the outcome is unavailable rather than allowed, and a raw credential is never used as a cache key or carried into a rejection |

### Non-Core Features

#### Credential Reload & Redacted Audit

ID: credential-reload-audit
Root WI: -
Status: verified
Type: Security
Feature Class: non-core
Required Verification: conformance
Promise:
Replace a running service's credential registry without restarting the process and without ever serving a partially applied registry: a rejected replacement leaves the last known good registry in place. Every reload and every authorization decision can be observed by an operator, and the observation schema has no place to put a credential.
Surfaces:
- Rust API: `service_auth::ReloadableRoleMapVerifier::reload_registry` / `reload_files` - validated, all-or-nothing replacement returning a monotonic revision.
- Rust API: `service_auth::AuthEvent` / `service_auth::AuthEventSink` - the observation schema and the port that receives it.
EC Dimensions:
- behavior: `cargo test -p service-auth --lib reload` - revision monotonicity, the entry count reported for an applied revision, all-or-nothing multi-source merge, and last-known-good retention on every failure class.
- security: `cargo test -p service-auth --lib reload` - the event schema cannot represent a credential, a rejected registry never becomes servable, and an unknown credential is reported only as a classification.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Monotonic validated replacement | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib reload`; a replacement is validated before it is installed, an applied replacement advances the revision by exactly one and reports the entry count it installed, and a rejected replacement advances nothing |
| Last known good retention | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib reload`; a read failure, a parse failure, and a validation failure each leave the previously serving registry intact and each is reported as its own failure class, and a multi-source reload in which any source fails installs none of them |
| Credential-free observation | change | - | implemented | verified | conformance | `cargo test -p service-auth --lib reload`; the event schema has no field that can carry a credential, an authorization event names the subject, resource, and required role but never the presented credential, and an unpresented or unrecognized credential is reported only as a stable reason value |
