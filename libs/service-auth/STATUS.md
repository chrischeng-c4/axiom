# service-auth status

## Scope

This document describes the current source contract for the reusable
`service-auth` crate. It does not claim that the named gate ran in this working
session.

Use the [README](README.md) for the public workflow. Use the
[roadmap](ROADMAP.md) for future outcomes and explicit non-goals.

## State definitions

| State | Meaning |
|---|---|
| Supported | The current source has a public contract, an implementation, and a named executable gate for the stated scope. |
| Limited | The current source supports the stated scope, but the Limits cell names a material boundary. |
| Not supported | The behavior is not part of the current product contract. The Evidence cell points to a future outcome or a non-goal. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| HTTP request auth middleware | `http-request-auth-middleware` | Supported | Rust services can extract bearer tokens, call an injected verifier, reject failures, and receive an authenticated principal. | Each app selects protected routes and supplies its authorization meaning. | `cargo test -p service-auth` |
| Kubernetes delegated auth | `kubernetes-delegated-auth` | Supported | Rust services can validate an explicit audience through TokenReview, accept only ServiceAccount principals, and authorize app-supplied attributes through SubjectAccessReview. | The app owns its API group, resource, name, verb, and namespace mapping. Kubernetes RBAC owns the answer. | `cargo test -p service-auth` |
| Rust projected token reader | `rust-projected-token-reader` | Supported | An explicitly configured `ProjectedTokenFile` reads the file for each call and rejects unreadable, expired, wrong-audience, or malformed token data. | The caller supplies the path and audience. The reader does not mount or rotate the token and does not inject an HTTP header. | `cargo test -p service-auth` |
| Kubeconfig TokenRequest and proxy | `kubeconfig-tokenrequest-proxy` | Supported | A Rust caller can use kubeconfig identity to mint and refresh an audience-bound ServiceAccount token and present it through the loopback proxy. | Kubernetes RBAC must permit the caller to create a token for the named ServiceAccount. | `cargo test -p service-auth` |
| Credential reload and redaction | `credential-reload-redaction` | Supported | Static role-map replacement keeps the last valid registry, and auth events omit raw bearer material. | This registry path is separate from Kubernetes delegated authorization. | `cargo test -p service-auth` |
| Portable opaque projected-token source | `portable-opaque-projected-token-source` | Not supported | The crate has no cross-language contract with caller-selected disabled and required modes that returns a fresh opaque token for each request. | The current Rust reader performs JWT preflight. Generated clients must not treat that preflight as a replacement for server-side TokenReview. | [Portable projected token contract](ROADMAP.md#portable-projected-token-contract) |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change. Move a future
outcome into current support only after its implementation and executable gate
exist.
