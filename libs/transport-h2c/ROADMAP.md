# transport-h2c roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.

## Near-term outcomes

No items.

## Later outcomes

No items.

## Non-goals

### Listener ownership

- ID: `listener-ownership`
- Reason: `server-http` and `server-tcp` bind sockets, admit accepted
  connections, supervise tasks, and aggregate terminal reports.

### TLS policy

- ID: `tls-policy`
- Reason: h2c is cleartext prior-knowledge HTTP/2. TLS configuration,
  certificate material, identity policy, and ALPN belong to listener and
  security libraries.

### Application routing

- ID: `application-routing`
- Reason: Apps and HTTP policy libraries define resources, routes, middleware,
  authentication, errors, and retry meaning.
