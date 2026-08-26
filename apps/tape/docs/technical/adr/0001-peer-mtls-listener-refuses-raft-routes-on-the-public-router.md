# ADR 0001 — Peer mTLS runs on its own listener and the public router refuses `/raft/*` (#1805)

Status: Accepted (2026-08-26; recorded from the retired `tech-design/interfaces/rest/wire-shared-peer-mtls-transport-into-the-raft-host.md`).

## Context

Tape replicates through a Raft group whose RPCs travel over HTTP. With no
peer TLS configured, the peer routes are merged into the same h2c app that
serves clients (`src/server.rs:656-662`). Once operators asked for
authenticated peer traffic, the question was whether to authenticate those
routes in place — a per-route policy on the public listener — or to move
them to a listener of their own.

The shared libraries already decide most of it: `libs/peer-tls` owns PEM
validation and the required-mTLS posture; `libs/raft-runtime` owns HTTPS
peer dialing, mutual-auth handshakes and reloadable transport state. Tape
owns only the `TAPE_PEER_*` environment prefix (`src/peer_tls.rs`) and the
choice of listener.

## Decision

- Peer TLS is all-or-nothing. Absent material keeps the h2c public-port
  topology. Present material must be complete and must set
  `TAPE_PEER_MTLS=on`; anything else fails before either listener binds
  (`src/bin/tape.rs:1032-1050`).
- In secure mode the Raft router is served **only** through
  `PeerTransport::serve` on the dedicated `--raft-port` / `TAPE_RAFT_PORT`
  (7138 by default), with peer URLs derived under the `https` scheme
  (`src/bin/tape.rs:1050,1107-1131`).
- The public app in secure mode is built by
  `router_without_raft_routes_with_admission` (`src/server.rs:530-546`),
  whose middleware answers `/raftz` and every `/raft/*` path with a refusal
  regardless of what a later merge might add.
- The operator renders named `http` and `raft` container ports and injects
  `TAPE_RAFT_PORT` (`src/operator/render.rs:672-675`); it never parses
  certificates and introduces no tape-specific certificate controller.

## Consequences

- A client can never reach Raft RPCs through the authenticated data plane,
  and a peer can never reach client routes through the peer port. The two
  audiences have two ports, and `raft_peer_mtls` proves the boundary from the
  outside while `secure_peer_mode_does_not_expose_raft_routes_on_public_router`
  (`src/server.rs:1631`) pins it from inside the crate.
- Public drain also drains the peer listener (`src/bin/tape.rs:1152`), so a
  node stops answering peers and clients together.
- Tape must not grow its own TLS parsing or transport code; changes to the
  handshake belong in `libs/peer-tls` and `libs/raft-runtime`.

## Status of work

Landed. Gate: `cargo test -p tape --test raft_peer_mtls` plus
`cargo test -p tape --lib server::tests::secure_peer_mode_does_not_expose_raft_routes_on_public_router`.
