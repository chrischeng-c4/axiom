"""Hot-loop bench for `socket.AF_INET` / `socket.SOCK_STREAM` /
`socket.socket` / `socket.gethostname` / `socket.gethostbyname` /
`socket.getaddrinfo` / `socket.create_connection` /
`socket.create_server` module-attribute reads (#1415).

End-user scenario: network library glue (HTTP server adapters,
client connection pools, async-IO transports) typically reads the
`socket` module-level constants and callables on every dispatch
site rather than caching a local alias. Wrapper code that builds
addresses via `socket.getaddrinfo`, opens listeners via
`socket.create_server`, and probes connectivity via
`socket.gethostbyname` re-resolves these names through the module's
attribute table on each call site. That per-call module-attribute
octet-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `socket.AF_INET` family are top-level module-dict probes
on 3.12 returning enum members and callables). Mamba's shim returns
the same values directly from a dense constant table in the
`socket` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only socket sentinels.

Workload: 10_000 paired reads of `socket.AF_INET`,
`socket.SOCK_STREAM`, `socket.socket`, `socket.gethostname`,
`socket.gethostbyname`, `socket.getaddrinfo`,
`socket.create_connection`, and `socket.create_server` per
iteration, compared by identity (`is`) against the hoisted baseline
references taken once before the loop. The accumulator increments
when all eight reads resolve to identical objects; under CPython
the enum members are interned singletons and the callables are
module-dict-resident, so `is` holds. Under mamba the dense
constant table guarantees the same. A misread (different identity /
wrong binding) would immediately fail the correctness assert and
dead-code elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import socket as _socket

# Hoist baseline references once before the loop. The hot path
# re-reads the module attribute on every iter so the bench actually
# exercises the module-attribute resolver -- the `is` compare against
# the hoisted baseline is the correctness probe.
_AF_INET_BASELINE = _socket.AF_INET
_SOCK_STREAM_BASELINE = _socket.SOCK_STREAM
_SOCKET_BASELINE = _socket.socket
_GETHOSTNAME_BASELINE = _socket.gethostname
_GETHOSTBYNAME_BASELINE = _socket.gethostbyname
_GETADDRINFO_BASELINE = _socket.getaddrinfo
_CREATE_CONNECTION_BASELINE = _socket.create_connection
_CREATE_SERVER_BASELINE = _socket.create_server

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    a = _socket.AF_INET
    b = _socket.SOCK_STREAM
    c = _socket.socket
    d = _socket.gethostname
    e = _socket.gethostbyname
    f = _socket.getaddrinfo
    g = _socket.create_connection
    h = _socket.create_server
    # Accumulator readback prevents DCE -- every iteration must
    # resolve to the identical objects bound at the `socket.AF_INET`
    # / `socket.SOCK_STREAM` / `socket.socket` / `socket.gethostname`
    # / `socket.gethostbyname` / `socket.getaddrinfo` /
    # `socket.create_connection` / `socket.create_server` module
    # slots.
    if (a is _AF_INET_BASELINE
            and b is _SOCK_STREAM_BASELINE
            and c is _SOCKET_BASELINE
            and d is _GETHOSTNAME_BASELINE
            and e is _GETHOSTBYNAME_BASELINE
            and f is _GETADDRINFO_BASELINE
            and g is _CREATE_CONNECTION_BASELINE
            and h is _CREATE_SERVER_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical objects
# via the module-attribute resolver. acc == ITERS or we have a
# regression in mamba's socket module-attribute table.
assert acc - ITERS == 0, f"socket module-attribute read acc drift: acc={acc} expected={ITERS}"
print("socket_type_read_hot:", acc)
