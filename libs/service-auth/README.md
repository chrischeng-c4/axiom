# service-auth

## Brief

`service-auth` provides reusable authentication and authorization mechanisms
for Axiom HTTP services.

It extracts bearer credentials, verifies identities, rejects invalid requests,
and passes a principal to service code. Its Kubernetes support reads projected
ServiceAccount tokens, requests short-lived tokens, delegates identity checks
to TokenReview, and delegates permission checks to SubjectAccessReview.

The library does not decide what a service resource means. An app supplies its
audience, protected operations, Kubernetes resource mapping, and access policy.
Kubernetes remains the identity and authorization authority.

## Primary workflow

1. Select a verifier and state whether authentication is required.
2. Attach the shared middleware to the protected HTTP routes.
3. Map the authenticated principal to a service-owned authorization request.
4. Reject an unanswered, invalid, or denied decision without exposing the
   credential.

## Authenticate a Kubernetes caller

`DelegatedAuthenticator` accepts only a Kubernetes ServiceAccount principal.
It asks TokenReview to validate the bearer token for an explicit audience. It
then asks SubjectAccessReview about app-supplied resource attributes.

The app owns the API group, namespace, resource, resource name, and verb. The
library owns the review calls, response validation, principal parsing, caches,
metrics, and fail-closed behavior. A successful HTTP response from the
Kubernetes API is not enough. The review status must contain an explicit valid
or allowed result.

## Present a Kubernetes identity

`ProjectedTokenFile` reads an audience-bound token from a file. It reads the
file again for each call. This lets a caller see kubelet rotation without a
process restart. It checks token shape, expiration, and audience before it
returns the credential. It never verifies the signature because the receiving
service delegates that check to TokenReview.

This Rust JWT preflight is a current, explicit adapter. It is not the portable
generated-client contract. The planned portable source returns opaque token
bytes in a caller-selected required mode. A missing, unreadable, or empty file
then fails before transport. A caller-selected disabled mode never reads the
path. The server still uses TokenReview as the authority for signature, expiry,
audience, and ServiceAccount identity.

`TokenRequestTarget`, `KubeTokenMinter`, and the refreshing token source create
short-lived ServiceAccount tokens from a caller's Kubernetes identity. The
loopback proxy uses the same source for human developer access. Kubernetes RBAC
decides who may request a token for a ServiceAccount.

The caller supplies the audience, namespace, ServiceAccount name, file path,
and lifetime. The library does not create a ServiceAccount, token Secret,
Role, or RoleBinding.

## Keep credentials out of state

Authentication errors and audit events describe the decision and credential
source. They do not include the bearer token. Reloadable static role maps keep
the last valid registry when a replacement is invalid.

A token belongs only in an HTTP Authorization header or an in-memory request.
It must not enter an argument, environment variable, custom resource, status,
Event, or log.

## Contract discovery

| Need | Source of truth |
|---|---|
| Public Rust API | `cargo doc -p service-auth --no-deps` |
| HTTP middleware and verifier traits | `libs/service-auth/src/middleware.rs`, `async_verifier.rs`, and `verifier.rs` |
| Kubernetes delegated auth | `libs/service-auth/src/k8s/delegated.rs`, `review.rs`, and `kube_backend.rs` |
| Projected token behavior | `libs/service-auth/src/k8s/projected.rs` |
| Planned portable opaque-token contract | [ROADMAP.md](ROADMAP.md#portable-projected-token-contract) |
| TokenRequest and developer proxy | `libs/service-auth/src/k8s/token_request.rs` and `loopback_proxy.rs` |
| Redaction and reload behavior | `libs/service-auth/src/reload.rs` |
| Executable behavior | `cargo test -p service-auth` |

## Capabilities

Every entry below is an equal library capability. Each source states its direct
contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| HTTP request authentication | `http-request-authentication` | Apply one reusable bearer-auth middleware while the app keeps its domain policy. | `libs/service-auth` |
| Kubernetes delegated authorization | `kubernetes-delegated-authorization` | Validate a ServiceAccount identity and ask Kubernetes whether it may use an app-owned resource. | `libs/service-auth`, `external:kubernetes` |
| Projected workload token reading | `projected-workload-token-reading` | Read a rotated audience-bound token from a file without leaking the credential. | `libs/service-auth`, `external:kubernetes` |
| TokenRequest and loopback access | `tokenrequest-loopback-access` | Mint and refresh a short-lived ServiceAccount token for a caller that Kubernetes RBAC allows. | `libs/service-auth`, `external:kubernetes` |
| Credential reload and redacted audit | `credential-reload-redacted-audit` | Replace validated static credentials and report auth decisions without recording bearer material. | `libs/service-auth` |

### HTTP request authentication

- ID: `http-request-authentication`
- Promise: Extract a bearer credential, call an injected verifier, reject a
  failed decision, and attach the accepted principal to the request.
- Sources:
  - [`libs/service-auth`](./) provides bearer parsing, synchronous and
    asynchronous verifier contracts, middleware, roles, and stable errors.
- Gate: `cargo test -p service-auth`

### Kubernetes delegated authorization

- ID: `kubernetes-delegated-authorization`
- Promise: Accept only a reviewed ServiceAccount identity and require an
  explicit SubjectAccessReview allowance for app-supplied resource attributes.
- Sources:
  - [`libs/service-auth`](./) provides TokenReview and SubjectAccessReview
    clients, strict ServiceAccount principal parsing, caches, metrics, and
    fail-closed response validation.
  - `external:kubernetes` validates the token, resolves the ServiceAccount
    identity, evaluates RBAC, and returns the review decisions.
- Gate: `cargo test -p service-auth`

### Projected workload token reading

- ID: `projected-workload-token-reading`
- Promise: Read the current projected token for each call and reject an
  unreadable, expired, wrong-audience, or malformed credential.
- Sources:
  - [`libs/service-auth`](./) provides the file reader, expiration and audience
    checks, typed errors, and credential-free formatting.
  - `external:kubernetes` issues and rotates the projected ServiceAccount token
    through the kubelet.
- Gate: `cargo test -p service-auth`

### TokenRequest and loopback access

- ID: `tokenrequest-loopback-access`
- Promise: Use kubeconfig identity to request and refresh a short-lived,
  audience-bound ServiceAccount token and present it through a local proxy.
- Sources:
  - [`libs/service-auth`](./) provides the target validation, Kubernetes
    TokenRequest client, refresh clock, token source, and loopback proxy.
  - `external:kubernetes` authenticates the human caller, authorizes the
    TokenRequest, and issues the ServiceAccount token.
- Gate: `cargo test -p service-auth`

### Credential reload and redacted audit

- ID: `credential-reload-redacted-audit`
- Promise: Adopt only a validated replacement registry and emit decision events
  that never contain raw credentials.
- Sources:
  - [`libs/service-auth`](./) provides atomic last-known-good replacement,
    redacted event types, event sinks, and refusal tests.
- Gate: `cargo test -p service-auth`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current support boundaries and evidence |
| [ROADMAP.md](ROADMAP.md) | Future shared outcomes and non-goals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Edit rules and required verification |
| [Kubernetes Service Accounts](https://kubernetes.io/docs/concepts/security/service-accounts/) | Kubernetes identity and token behavior |
| [Kubernetes projected volumes](https://kubernetes.io/docs/concepts/storage/projected-volumes/) | Kubelet token projection and rotation |
