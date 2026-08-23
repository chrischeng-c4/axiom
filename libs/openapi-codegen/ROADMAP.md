# openapi-codegen roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
Stable IDs here let current limits point to one destination.

## Near-term outcomes

### Media type and streaming operations

- ID: `media-type-and-streaming-operations`
- Outcome: The shared operation model and all three emitters preserve declared
  non-JSON media types and expose bounded streaming request and response APIs.
- Boundary: The generator owns reusable OpenAPI media-type modeling, framing
  hooks, backpressure, cancellation, and emitted client methods. Each service
  owns its stream record schema, size limits, ordering, and retry meaning.
- Completion evidence: Cross-language fixtures generate and run an NDJSON
  upload plus an incremental response without buffering the complete stream.
  Tests cover media-type selection, bounded buffering, malformed records,
  cancellation, early disconnect, and a normal JSON operation in the same API.
- Tracking: [#3807](https://github.com/chrischeng-c4/axiom/issues/3807) (`lumen@0.4.45`).

### Structured client errors

- ID: `structured-client-errors`
- Outcome: Generated methods decode declared non-success response schemas into
  a stable typed error while preserving status, headers, and an unknown-body
  fallback.
- Boundary: The service declares error schemas and their status mapping. The
  generator owns reusable response dispatch and language-specific error types.
  It must not invent service error meaning or expose credential material.
- Completion evidence: TypeScript, Python, and Rust fixtures decode declared
  `401`, `413`, `429`, and `500` JSON errors, retain `Retry-After`, preserve an
  unknown media type safely, and redact Authorization data from all display and
  debug paths.
- Tracking: [#3807](https://github.com/chrischeng-c4/axiom/issues/3807) (`lumen@0.4.45`).

### Cross-language type parity

- ID: `cross-language-type-parity`
- Outcome: Supported OpenAPI unions and enums retain equivalent typed meaning
  in generated TypeScript, Python, and Rust source.
- Boundary: The generator defines one documented schema subset. Unsupported or
  ambiguous schemas fail generation with a path to the source schema instead
  of silently degrading to an untyped value.
- Completion evidence: Shared fixtures prove discriminated and ordinary unions,
  string enums, nullable variants, round trips, invalid-value refusal, stable
  names, and equivalent wire JSON in all three languages. Rust no longer emits
  `serde_json::Value` for a supported union or `String` for a supported enum.
- Tracking: [#3807](https://github.com/chrischeng-c4/axiom/issues/3807) (`lumen@0.4.45`).

### Complete target dependency manifest

- ID: `complete-target-dependency-manifest`
- Outcome: Each explicit target emits complete provenance and dependency
  metadata that can install, compile, and run the generated source in a clean
  environment.
- Boundary: The manifest records the OpenAPI SHA, generator version, target
  profile, caller-supplied service compatibility label, and every generated
  runtime dependency. It does not interpret service compatibility, create or
  publish a language package, select a package registry, or own a consumer's
  application dependencies.
- Completion evidence: Clean TypeScript, Python, and Rust fixtures install only
  the emitted requirements, compile the generated source, and execute one JSON
  operation. A controlled test removes each required dependency and proves the
  clean build fails with that dependency named.
- Tracking: [#3807](https://github.com/chrischeng-c4/axiom/issues/3807) (`lumen@0.4.45`).

### Dynamic request auth provider

- ID: `dynamic-request-auth-provider`
- Outcome: Every generated client can obtain request headers from an injected
  service-neutral provider immediately before it sends each request.
- Boundary: The provider API can return no header, a current header, or a hard
  error. Provider failure stops the request before transport. The generator
  does not know Kubernetes, KSA, token paths, audiences, Fleet policy, or RBAC.
  An app composes its credential source with this generic request hook.
- Completion evidence: Generated TypeScript, Python, and Rust tests prove one
  provider call per request, rotated values on consecutive requests, no call
  when no provider is configured, transport refusal after a provider error,
  async behavior where applicable, cancellation, and credential-free error
  formatting. App integration tests own their optional or required credential
  modes, token-file semantics, and server authentication decisions.
- Tracking: [#3799](https://github.com/chrischeng-c4/axiom/issues/3799) (`lumen@0.4.36`).

### Operation-aware retry hooks

- ID: `operation-aware-retry-hooks`
- Outcome: Generated clients expose common deadline, cancellation, response
  metadata, and retry hooks so an app can define one policy for all languages.
- Boundary: The generator carries operation ID, HTTP method, caller-supplied
  safety metadata, response status, headers such as `Retry-After`, and an
  optional idempotency key into the hook. It can provide bounded exponential
  backoff and jitter primitives. It does not decide which service operation is
  safe, choose retry statuses, mint an idempotency key, or own authentication
  policy.
- Completion evidence: TypeScript, Python, and Rust fixtures prove deadline and
  cancellation propagation, `Retry-After`, bounded backoff and jitter, no retry
  without an app decision, safe read retry, keyed and unkeyed write decisions,
  provider failure before transport, redacted errors, and identical attempt
  counts across languages.
- Tracking: [#3808](https://github.com/chrischeng-c4/axiom/issues/3808) (`lumen@0.4.46`).

### Strict cross-language generation gates

- ID: `strict-cross-language-generation-gates`
- Outcome: A reusable required harness generates, installs, compiles, and runs
  TypeScript, Python, and Rust output without silently skipping a language.
- Boundary: Product repositories select and pin the service journey. The
  library supplies clean target setup, toolchain checks, result reporting, and
  shared generator fixtures. Missing required tooling is a failed setup, not a
  successful skip. Local convenience tests may remain optional.
- Completion evidence: The normal gate records all three executed languages
  and passes one common JSON, error, auth-provider, and retry-hook fixture. A
  controlled negative case removes each interpreter, compiler, runtime, and
  dependency in turn and proves the required gate fails with that prerequisite
  named.
- Tracking: [#3807](https://github.com/chrischeng-c4/axiom/issues/3807) (`lumen@0.4.45`).

## Later outcomes

### Additional operation emission

- ID: `additional-operation-emission`
- Outcome: Arbitrary OpenAPI `additionalOperations` entries generate typed
  request methods in TypeScript, Python, and Rust.
- Boundary: Dedicated path-item operations keep their current names and
  behavior. Duplicate methods remain rejected or deduplicated in the shared
  operation model before language emission.
- Completion evidence: Cross-language fixtures prove typed emission and wire
  dispatch for at least one non-standard method other than `QUERY`, plus
  duplicate and invalid-method refusal.
- Tracking: Not assigned.

## Non-goals

### Identity policy and token acquisition

- ID: `identity-policy-and-token-acquisition`
- Reason: An OpenAPI security scheme describes how a request presents a
  credential. Apps and auth libraries choose an identity system and obtain or
  validate the credential.

### Fleet and RBAC policy

- ID: `fleet-and-rbac-policy`
- Reason: A generated HTTP client does not own Kubernetes workload lifecycle,
  ServiceAccount selection, access declarations, RoleBindings, or permissions.

### Package publication

- ID: `package-publication`
- Reason: This library generates source and dependency metadata. Product teams
  and consumers decide whether to vendor it or operate a language-package
  release train.
