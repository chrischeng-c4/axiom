# server-http roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.

## Near-term outcomes

No items.

## Later outcomes

No items.

## Non-goals

### Certificate parsing and identity policy

- ID: `certificate-parsing-and-identity-policy`
- Reason: `peer-tls` or another security implementation parses material,
  validates identity and validity, and decides which configuration is active.
  This crate consumes an optional ready rustls configuration.

### Route and middleware policy

- ID: `route-and-middleware-policy`
- Reason: Apps define domain routes. `service-http` provides shared operational
  routes, request policy, error envelopes, and observability adapters.

### Client transport internals

- ID: `client-transport-internals`
- Reason: `transport-h2c` owns h2c client connections, pools, GOAWAY handling,
  safe retry, and mutation ambiguity.
