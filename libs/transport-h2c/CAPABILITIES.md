# Transport H2C Capabilities

## Brief

`transport-h2c` is how one axiom service talks to another over HTTP/2
cleartext. Everything inside the mesh is prior-knowledge h2c — no TLS, no
upgrade dance — so the interesting problem is not the protocol, it is the
pool: how many sockets to open, which one to put the next stream on, how to
notice one has died, and how to stop a burst from becoming unbounded stream
fan-out.

It owns two surfaces at different levels. `H2cPool` is a fixed-size set of
reqwest clients for callers who just want a client and nothing more.
`H2cManager` is the frame-level pool that actually manages connections:
least-loaded dispatch, bounded admission, demand-driven growth, and recovery
when a peer disappears. The signals reqwest hides — GOAWAY, driver death,
per-connection in-flight depth — are exactly the ones the manager acts on.

It does not own retries beyond one, request semantics, service discovery, or
authentication. It owns the socket layer under them.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** are the manager's dispatch contract: where a stream goes,
  how many are admitted at once, and how many sockets exist to carry them.
- **Non-Core Features** keep that contract true over time — recovery after a
  peer dies, the statistics an operator reads, and the simpler client helpers
  callers reach for first. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Least-Loaded Stream Dispatch | - | implemented | verified | smoke | ready | core; each request lands on the healthy connection carrying the fewest in-flight streams, and a request whose connection dies is retried exactly once on a fresh one |
| Bounded Admission | - | implemented | verified | smoke | ready | core; a per-origin semaphore caps concurrent requests, a burst queues rather than fanning out streams, and the wait has a deadline |
| Adaptive Connection Sizing | - | implemented | verified | smoke | ready | core; the pool grows when the least-loaded connection saturates and never exceeds its cap, which is logarithmic in target concurrency and bounded by the core count |
| Connection Health Recovery | - | implemented | verified | smoke | ready | non-core; a connection that dies is evicted and replaced without the caller seeing a failure |
| Live Pool Statistics | - | implemented | verified | smoke | ready | non-core; one snapshot reports connection count, healthy count, in-flight depth, and cumulative request and error totals |
| Prior-Knowledge Client Helpers | - | implemented | verified | smoke | ready | non-core; a fixed pool of h2c clients handed out round-robin, speaking HTTP/2 cleartext without an upgrade negotiation |

### Core Features

#### Least-Loaded Stream Dispatch

ID: least-loaded-stream-dispatch
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A request is dispatched to the healthy connection with the fewest in-flight
streams, never by blind round-robin. The in-flight slot is reserved at
dispatch time rather than at send time, so concurrent dispatchers see the
chosen connection's load rise immediately and spread across the pool instead
of piling onto one socket. The slot is released when the lease drops, which
includes the cancellation path, so a cancelled request cannot leak load.
Bodies are flow-controlled: a body larger than the stream window is sent in
capacity-bounded chunks rather than buffered into one frame. When a send fails
with an error that means the connection is gone — GOAWAY, stream reset, or I/O
death — the request is retried exactly once on a freshly leased connection;
any other error is returned immediately. A retry reuses the same method, URI,
version, headers, and body. A connect that cannot be established surfaces as
an error the caller can classify, not as a hang.
Surfaces:
- Rust API: `transport_h2c::H2cManager::request` - dispatch a fully-built request with the retry contract.
- Rust API: `transport_h2c::H2cManager::get` / `put` / `post` - the method shorthands.
- Rust API: `transport_h2c::H2cManager::with_config` - build a manager against one authority.
- Rust API: `transport_h2c::H2cError::is_connection_lost` - which failures are retryable.
Rust internal: the least-loaded selection, the lease that reserves before send and releases on drop, the flow-controlled body send, and the request duplication used for the retry.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --test manager` - a GET round-trips with the expected status and body; a PUT of a 50 KB body is echoed back byte-for-byte through the flow-controlled send path; and under 64 concurrent requests every one succeeds while the pool spreads them instead of concentrating on one socket.
- security: `cargo test -p transport-h2c --test manager` - a peer torn down and replaced does not fail the caller's next request, because the dead connection is detected and the request retried once; and a connect to an address with nothing listening fails fast with an error that reports itself as connection-lost rather than hanging or reporting success.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Request round-trip | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `get_and_put_roundtrip` asserts a 200 with the exact body, so the dispatch path is proven end to end rather than by construction |
| Flow-controlled body send | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; the same test echoes a 50 KB PUT body verbatim, so a body larger than one frame is chunked correctly rather than truncated at the window |
| Least-loaded selection under load | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `concurrency_grows_the_pool_least_loaded` runs 64 concurrent slow requests and all 64 succeed, so one saturated multiplexed socket does not hold the whole burst behind its flow-control window |
| Single retry on connection loss | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `reconnects_after_connections_die` tears the server down and back up and the next request still returns 200, so a healing pool recovers a request instead of surfacing the outage |
| Connect failure is classifiable | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `connect_failure_is_reported` asserts the error reports `is_connection_lost`, so a caller can tell a dead peer from a rejected request |

#### Bounded Admission

ID: bounded-admission
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Requests admitted into one manager at once are capped by a per-origin
semaphore. A caller arriving at a full manager queues for an admission slot
rather than opening another stream, and that wait has a deadline: when
`pool_timeout` elapses the caller gets a timeout error, not an unbounded
block. This is what keeps async task fan-out from becoming unbounded
concurrent streams on a socket whose peer advertises a much smaller
`MAX_CONCURRENT_STREAMS`. A slot released by a completed request admits the
next caller, so the cap throttles rather than deadlocks. Shutdown closes
admission, so every subsequent caller fails fast with a distinct shutdown
error instead of waiting on a pool that is gone.
Surfaces:
- Rust API: `transport_h2c::ManagerConfig::max_in_flight_per_origin` - the admission cap.
- Rust API: `transport_h2c::ManagerConfig::pool_timeout` - the deadline for waiting on a slot.
- Rust API: `transport_h2c::H2cManager::shutdown` - close admission and drop connections.
- Rust API: `transport_h2c::H2cError::Timeout` / `Shutdown` - the two refusal outcomes, distinguishable by the caller.
Rust internal: the owned semaphore permit carried by the lease, and the ordering that acquires admission before selecting a connection.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --test manager` - with the cap set to one, a second caller is refused while the first holds the slot, and the first request still completes with a 200, so admission throttles concurrency without losing the admitted work.
- security: `cargo test -p transport-h2c --test manager` - the refused caller receives `Timeout` within the configured `pool_timeout` rather than blocking indefinitely, and after `shutdown` a request receives `Shutdown` rather than being served by a half-torn-down pool.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Per-origin admission cap | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `admission_queue_times_out_when_in_flight_cap_is_full` proves concurrency into the transport is bounded by configuration rather than by how many tasks the caller happened to spawn |
| Deadlined queueing | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; the refused caller gets `Timeout` while the holder is still running, so backpressure reaches the caller as an error it can shed instead of an indefinite stall |
| Admitted work still completes | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; the first request returns 200 after the second is refused, so the cap throttles arrivals rather than deadlocking the holder |
| Shutdown refuses new work | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `shutdown_refuses_new_requests` asserts the distinct `Shutdown` error, so a drained manager cannot look merely slow |

#### Adaptive Connection Sizing

ID: adaptive-connection-sizing
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Connection count tracks demand instead of being guessed. The recommended size
for a target peak concurrency is `clamp(ceil(ln(concurrency)), 1, cores)`,
with one connection for a concurrency of two or less — HTTP/2 multiplexes, so
the count is logarithmic, not linear, and never exceeds available parallelism.
At run time the manager opens a new connection when the least-loaded healthy
one is at or above `grow_threshold` and the pool is under `max_connections`,
and when no healthy connection exists at all. Growth stops exactly at the cap:
a burst far larger than the pool queues on admission rather than opening
sockets without limit. A growth attempt that fails falls back to the
least-loaded existing connection rather than failing the request.
Surfaces:
- Rust API: `transport_h2c::recommended_h2c_connections` - the sizing heuristic against the live core count.
- Rust API: `transport_h2c::recommended_h2c_connections_for` - the same heuristic with an explicit core cap, for deterministic sizing.
- Rust API: `transport_h2c::cpu_parallelism` - the core count, defaulting to one.
- Rust API: `transport_h2c::ManagerConfig::for_concurrency` - a config sized for a target peak concurrency.
- Rust API: `transport_h2c::ManagerConfig::min_connections` / `max_connections` / `grow_threshold` - the sizing bounds.
Rust internal: the slot reservation that enforces the cap without holding the connection write lock across a dial, and the fallback when growth fails.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --lib` - the heuristic returns the exact clamped logarithm across representative concurrencies, returns one for a concurrency of one or two, and never exceeds the supplied core cap.
- security: `cargo test -p transport-h2c --test manager` - 64 concurrent requests against a pool configured with a minimum of one and a maximum of four grow it to at least two and at most four, so demand raises the socket count but the configured cap is a hard ceiling rather than a hint.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Logarithmic sizing heuristic | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --lib`; `heuristic_is_log_shaped` pins the exact clamped natural log, so a service sized for 10k concurrent requests opens single-digit sockets rather than a socket per request |
| Core-count ceiling and floor | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --lib`; `heuristic_clamps_to_cores_and_floor` proves the count never exceeds available parallelism and never drops below one, so a small container does not run more connection drivers than it has cores |
| Demand-driven growth within the cap | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `concurrency_grows_the_pool_least_loaded` asserts the observed pool size lands in `2..=4` under a 64-request burst, so growth is real and the cap holds |

### Non-Core Features

#### Connection Health Recovery

ID: connection-health-recovery
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
When a connection dies, the pool heals itself. The connection driver flips a
liveness bit the moment its connection ends for any reason — clean close,
GOAWAY, or I/O failure — so a dead socket is never selected for the next
dispatch. A background supervisor sweeps on a fixed cadence: it pings each
healthy connection for positive liveness, evicts what is dead, sheds at most
one connection that is idle past `idle_timeout` or above the keepalive
ceiling and carrying no streams, and replenishes back to `min_connections`.
The supervisor holds only a weak reference, so it stops when the last manager
handle is dropped rather than keeping the pool alive forever. The observable
promise is the one a caller cares about: a peer that restarts costs no
request.
Surfaces:
- Rust API: `transport_h2c::ManagerConfig::ping_interval` / `idle_timeout` / `max_keepalive_connections` - the supervision cadence and shrink bounds.
- Rust API: `transport_h2c::H2cManager::stats` - observe health after a sweep.
Rust internal: the PING/PONG liveness probe, the driver task's liveness store, the single-shrink-per-sweep rule, and the weak reference that ends the supervisor.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --test manager` - a server torn down and restarted on the same port leaves the manager reporting at least one healthy connection again, so the pool refills without caller involvement.
- security: `cargo test -p transport-h2c --test manager` - the request issued after the peer died still returns 200, so a dead connection is evicted rather than dispatched onto, and the caller never observes the outage as a failure.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dead-connection eviction and refill | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `reconnects_after_connections_die` asserts a 200 and `healthy >= 1` after a full server teardown and restart, so a peer restart heals without the caller paying for it |

#### Live Pool Statistics

ID: live-pool-statistics
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
One `stats` call reports the whole pool: how many connections exist, how many
of those are healthy, how many streams are in flight across them, and the
cumulative request and error totals. The health split is the point — a pool
that is nominally sized but mostly dead reads as such instead of looking
fine. The totals accumulate across the pool rather than per connection, and
when a connection is evicted, shrunk, or dropped at shutdown its counters are
retired into the manager rather than discarded, so a self-healing pool does
not silently reset an operator's error rate.
Surfaces:
- Rust API: `transport_h2c::H2cManager::stats` - the snapshot.
- Rust API: `transport_h2c::ManagerStats` - connections, healthy, in-flight, total requests, total errors.
Rust internal: the retired-request and retired-error accumulators, and the retirement performed on eviction, shrink, and shutdown alike.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --test manager` - after 64 successful concurrent requests the snapshot reports at least 64 total requests and a connection count inside the configured bounds, so the totals reflect real work rather than a construction-time constant.
- security: `cargo test -p transport-h2c --test manager` - the same all-success run reports exactly zero total errors, so the error counter has a proven zero point and cannot drift upward on its own; and after a peer restart the healthy count is asserted separately from the connection count, so a degraded pool stays distinguishable from a healthy one.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Cumulative request and error totals | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `concurrency_grows_the_pool_least_loaded` asserts `total_requests >= 64` and `total_errors == 0`, so the counters track real dispatches and the error counter is anchored |
| Health split in the snapshot | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --test manager`; `reconnects_after_connections_die` asserts `healthy` independently of `connections`, so an operator can tell a sized pool from a working one |

#### Prior-Knowledge Client Helpers

ID: prior-knowledge-client-helpers
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
For callers who want a client rather than a pool, this crate builds reqwest
clients that speak HTTP/2 cleartext by prior knowledge — no upgrade
negotiation, no TLS. `H2cPool` holds a fixed set of such clients, sized by the
same logarithmic heuristic, and hands them out round-robin so a caller that
does not need managed connections still spreads its load. An optional
per-request timeout and user agent are the only knobs, because everything
else belongs to the caller's request. A pool asked for zero connections still
reports one and yields a usable client: a bad size is a performance problem,
not a panic at startup. This is the drop-in path, and it manages nothing — a
caller that needs health, growth, or in-flight visibility uses `H2cManager`
instead, and this contract says so rather than implying the simple pool
supervises anything.
Surfaces:
- Rust API: `transport_h2c::h2c_client` - one prior-knowledge h2c client.
- Rust API: `transport_h2c::h2c_client_with` - the same with a timeout and user agent.
- Rust API: `transport_h2c::H2cPool::for_concurrency` / `with_connections` / `with_connections_and` - a fixed pool of clients.
- Rust API: `transport_h2c::H2cPool::connections` / `client` / `get` / `post` - round-robin access.
- Rust API: `transport_h2c::serve_connection` / `serve_connection_with_options` - the server side of the same cleartext contract.
Rust internal: the shared builder that sets prior knowledge before applying optional settings, and the round-robin index.
EC Dimensions:
- behavior: `cargo test -p transport-h2c --lib` - a pool built with three connections reports three and rotates across all three over successive calls before repeating, so access is genuinely round-robin rather than always returning the first client.
- security: `cargo test -p transport-h2c --lib` - a pool built with zero connections reports one and yields a usable client rather than panicking or dividing by zero, so a misconfigured size degrades instead of crashing the caller at startup.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Round-robin pool access | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --lib`; `pool_round_robins_across_connections` proves successive calls rotate across the configured clients rather than pinning to one |
| Degenerate size tolerated | change | - | implemented | verified | smoke | `cargo test -p transport-h2c --lib`; `pool_floor_is_one_connection` proves a zero request still yields one usable client, so a bad configuration cannot panic the caller |
