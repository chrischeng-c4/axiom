---
id: '1891'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-millisecond-reserve-policy
entry: serve_flags
nodes:
  flags: { kind: start, label: "Parse reserve, queue, and idle timeout flags in milliseconds." }
  policy: { kind: process, label: "Construct one Duration-based ReserveLeasePolicy without unit conversion loss." }
  wait: { kind: process, label: "Backend pool and reactor set deadlines from the Duration policy." }
  idle: { kind: process, label: "Reserve client compares idle elapsed Duration before release." }
  verify: { kind: terminal, label: "A 500ms value remains 500ms through all consumers." }
edges:
  - { from: flags, to: policy }
  - { from: policy, to: wait }
  - { from: policy, to: idle }
  - { from: wait, to: verify }
  - { from: idle, to: verify }
---
flowchart TD
    flags([Read *_MS flags]) --> policy[Build Duration policy]
    policy --> wait[Use Duration for queue and reserve deadlines]
    policy --> idle[Use Duration for idle release]
    wait --> verify([500ms is preserved])
    idle --> verify
```

The runtime policy owns timeout units as `Duration`. CLI milliseconds convert exactly with `Duration::from_millis`; no runtime code divides them by 1,000. Lease TTL remains seconds because it is persisted in the control-plane lease contract. The reserve client retains deterministic tests by accepting elapsed `Duration` for local timeout checks while converting only the lease-expiry clock to seconds at the Kubernetes boundary.
