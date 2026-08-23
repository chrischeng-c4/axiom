# service-auth roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
Stable IDs here let current limits point to one destination.

## Near-term outcomes

### Portable projected token contract

- ID: `portable-projected-token-contract`
- Outcome: Client adapters can share one service-neutral decision contract for
  explicit disabled or required projected-token use, request-time rotation,
  opaque credential handling, and redaction.
- Boundary: The caller selects the mode. Disabled mode never reads a path.
  Required mode rereads its configured path for every request and treats a
  missing, unreadable, or empty file as a hard pre-transport failure. It returns
  the token only in memory and does not parse it into an authorization decision.
  TokenReview remains the server authority for signature, expiration,
  audience, and ServiceAccount identity. The current Rust
  `ProjectedTokenFile` may keep its optional local JWT preflight as a distinct
  adapter. This outcome does not define an HTTP client API, Lumen audience or
  path, Fleet policy, ServiceAccount creation, or RBAC.
- Completion evidence: Reusable fixtures cover disabled mode with no file,
  required-mode missing, unreadable, empty, rotated, and valid opaque files,
  repeated reads, provider cancellation, and concurrent requests. Failure,
  display, debug, and event capture prove that no token appears. Server fixtures
  prove expired, wrong-audience, and malformed credentials remain TokenReview
  decisions. Generated TypeScript, Python, and Rust clients consume the same
  portable cases through their owning library tests.
- Tracking: Not assigned.

## Later outcomes

No items.

## Non-goals

### Service permission semantics

- ID: `service-permission-semantics`
- Reason: Each app owns the meaning of its protected operations and maps them
  to resource attributes before it calls the shared authorization mechanism.

### Kubernetes access policy ownership

- ID: `kubernetes-access-policy-ownership`
- Reason: Kubernetes RBAC stores and evaluates access policy. An app or shared
  Kubernetes renderer may declare policy objects, but this auth library does
  not own their lifecycle.

### Generated client emission

- ID: `generated-client-emission`
- Reason: `libs/openapi-codegen` owns generated client APIs and request-time
  header providers. This library owns token-source behavior and conformance.
