# service-http roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
Stable IDs here let current limits point to one destination.

## Near-term outcomes

### OpenAPI middleware response projection

- ID: `openapi-middleware-response-projection`
- Outcome: An app can project every reachable shared middleware response and
  its error envelope into each affected OpenAPI operation without duplicating
  response definitions by hand.
- Boundary: This library owns reusable response components and middleware-to-
  operation projection hooks. The app owns which middleware is mounted, its
  auth policy, domain statuses, operation schemas, and final document.
- Completion evidence: A fixture composes auth, body-limit, admission, and
  internal-error middleware and proves the generated document includes the
  reachable `401`, `413`, `429`, and `500` responses with the shared envelope.
  A negative fixture omits a mounted response and makes the parity gate fail.
- Tracking: Not assigned.

### Outbound trace propagation

- ID: `outbound-trace-propagation`
- Outcome: A service-neutral client adapter injects the current valid W3C trace
  context into an outbound HTTP request.
- Boundary: The adapter propagates context only. It does not own an HTTP client,
  retry policy, destination policy, sampling, or service authentication.
- Completion evidence: Tests prove child-span injection, local-root injection,
  invalid incoming header replacement, concurrent-request isolation, and no
  credential or request-body data in trace fields.
- Tracking: Not assigned.

## Later outcomes

No items.

## Non-goals

### Domain routes and schemas

- ID: `domain-routes-and-schemas`
- Reason: Each app owns its resources, operations, request and response schemas,
  and domain error meaning.

### Authentication policy

- ID: `authentication-policy`
- Reason: Apps and auth libraries decide who may call an operation and how an
  identity is obtained or verified. This crate only supplies HTTP policy hooks.

### Listener and TLS ownership

- ID: `listener-and-tls-ownership`
- Reason: `server-http` owns the listener lifecycle and TLS serving. This crate
  composes a router with that runtime.
