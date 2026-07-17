---
id: '1882'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-discovery-self-connection-accounting
entry: frontend_startup
nodes:
  frontend_startup: { kind: start, label: "Validated frontend StartupMessage" }
  identity: { kind: process, label: "Replace every client application_name with pgpool-<pod> from PGPOOL_POD_NAME" }
  cache: { kind: process, label: "Use the rewritten startup as every replay and bootstrap identity" }
  backend: { kind: process, label: "Forward rewritten startup on session, legacy transaction, and reactor backend handshakes" }
  discovery: { kind: process, label: "Count only pg_stat_activity client backends and classify pgpool-* identities" }
  capacity: { kind: process, label: "Cap raw max_connections then subtract superuser reserve, foreign client usage, configured reserve, and headroom" }
  reconcile: { kind: terminal, label: "Preserve an unchanged replica target unless actual external capacity is insufficient" }
edges:
  - { from: frontend_startup, to: identity }
  - { from: identity, to: cache }
  - { from: cache, to: backend }
  - { from: backend, to: discovery }
  - { from: discovery, to: capacity }
  - { from: capacity, to: reconcile }
---
flowchart TD
  frontend_startup([validated StartupMessage]) --> identity[replace client application_name with pgpool-pod]
  identity --> cache[replay and bootstrap cache use rewritten identity]
  cache --> backend[all backend handshakes forward the rewritten startup]
  backend --> discovery[discover client-only pg_stat_activity usage]
  discovery --> capacity[min raw limit then subtract reserved and foreign usage]
  capacity --> reconcile([unchanged target remains admitted])
```

### Backend identity

`pgpool serve` derives one backend identity, `pgpool-<pod>`, from
`PGPOOL_POD_NAME`; the Deployment renderer supplies that value through the
Downward API. Before any data-plane path stores, compares, replays, bootstraps,
or forwards a `StartupMessage`, it removes every client-supplied
`application_name` entry and appends that controlled identity. Session mode,
legacy transaction mode, and the dense-buffer reactor use this same helper.
The rewritten message, rather than the client-original message, is the exact
startup-replay key and is the message sent to PostgreSQL, so a cached trust
reply can never authorize a backend carrying a different identity.

### Runtime discovery and capacity

The discovery query reads `max_connections` and
`superuser_reserved_connections` and counts only
`pg_stat_activity.backend_type = 'client backend'`. Its pgpool count is the
subset whose controlled `application_name` begins `pgpool-`; background workers
never contribute to either total or foreign usage. The effective allocatable
limit is the minimum of the raw runtime, configured, and advisory ceilings,
then reduced by the runtime superuser reservation. Foreign usage is client
total minus pgpool usage, with saturating arithmetic. The existing endpoint
reserve and safety headroom remain later capacity deductions.

### Reconcile invariant

Capacity planning continues to deny a new replica target when its requested
per-pod quota exceeds this allocatable capacity. With an unchanged target,
pgpool-held connections are never double-subtracted as foreign usage, so a busy
pool cannot manufacture a `Blocked` status or freeze reconciliation. Discovery
errors remain fail-closed for new scale-up.
