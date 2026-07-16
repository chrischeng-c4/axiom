---
id: vat-source-projects-vat-tests-vat-k8s-ephemeral-rs
summary: Process-level regression for bounded Apple Container K3s sessions, including independently installed non-OrbStack kubectl provenance, bootstrap diagnostics, bounded JSON leased exec, and hardened loopback Service port-forward cleanup and recovery.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: headless-ephemeral-kubernetes-session
    coverage: partial
    rationale: "#1693 proves public command behavior without making every CI host mutate Apple Container."
---

# Source mirror: apps/vat/tests/vat_k8s_ephemeral.rs

## Overview
<!-- type: overview lang: markdown -->

The deterministic test replaces container and kubectl with tiny PATH-local
executables. It proves the public CLI creates and deletes only the owned machine,
injects private K3s paths into exactly one host child, removes those paths before
return, and preserves the child numeric exit code. It also proves one active lease
can forward only a literal Service to loopback for one host child that receives
endpoint metadata while VAT strips kubeconfig, cache/API variables, and VAT_HOME
from its environment; this is not a same-UID OS sandbox or adversarial-child
security boundary. The fake host child shares kubectl's authenticated process
group, so the suite proves a TERM-ignoring ordinary descendant is gone before
`cleanup=confirmed`; children are explicitly cooperative and non-daemonizing,
not a detached-process guarantee. It also proves a SIGKILLed VAT parent is
recovered through an exec-wrapper kubectl once a later operation acquires the
released, non-inherited `CLOEXEC` flock. Recovery is based on a v2 CSPRNG marker identity and exact forward
shape after a later operation holds the lock, not on recorded owner-PID liveness;
an unauthenticated leader fails closed. The suite covers stale v2 cleanup and a
legacy v1 marker only when its recorded group is already absent, because v1 is
storage-only and is never signalled. The implementation records a durable
`cleaning` tombstone before unlink so torn cleanup remains retryable. Current
real-host gates require an independently installed kubectl first on PATH;
Homebrew `/opt/homebrew/bin/kubectl` is installed locally. Independent-kubectl
one-shot, leased, local-image, and Service-forward E2Es passed 1/1 (36 filtered)
in 28.38s, 29.97s, 49.73s, and 49.57s. The local-image run loads one already-
local Apple `alpine:3.20` into one lease, runs a pod with `imagePullPolicy=Never`,
observes its marker log, and completes exact session cleanup; it does not
establish registry-pull generality, persistence, GUI, or Docker Engine/API
behavior. The leased run includes strict JSON exec with `--timeout 30`; the
Service-forward run includes one Service-only loopback strict JSON tunnel with a
credential-free child, confirmed cleanup, and closed local ports. They do not
establish same-UID OS isolation. The passed deterministic fake bootstrap regression keeps the original
error primary, then checks staged non-sensitive installer evidence and exactly
`guest_install_log`, `guest_k3s_system`, `backing_container_logs`,
`machine_boot_log`, `machine_inspect`, and `container_system_status` under a
six-second total / one-second-per-probe fixed read-only budget. It confirms the
diagnostic path excludes private kubeconfig/cache and host credentials, does not
rerun `k3s --version`, leaves the existing 300-second bootstrap behavior
unchanged, introduces no wrapper/recovery command, and still completes exact
owned-machine and private-session cleanup.

The focused status API fake cases cover an exact reachable owned API,
non-probing retained-forward recovery, non-probing expiry, and fail-closed
busy/unavailable/identity mismatch with no session marker or private credential
mutation (4/4). The precise expiry-recheck unit passed 1/1. A real-host status
API run remains unproven; those checks do not change the one-boot/nonpersistent
boundary.

The deterministic JSON-exec fake cases distinguish bounded text exec from
`vat k8s session exec --format json [--timeout SECONDS] <id> -- COMMAND`. They
assert that omission uses remaining TTL, an overlong explicit timeout fails
before spawn, normal/deadline/interrupted execution reaps its owned group, and
the marker is removed only after group absence. A starting/live crash marker
blocks later exec/delete/cleanup fail-closed rather than claiming termination.
They also assert that a spawned credentialed child produces one
`vat.k8s.session.exec.v1` `vat_json` document with its exit code, separate
stdout/stderr, no raw replay, `api_verified=true`, `runtime_invoked=true`,
`session_record_mutated=false`, no private-path output, and unchanged session
marker contents. Credential-validation and owned-API-probe failures mask private
paths, and an owned API probe which crosses the lease deadline prevents child
spawn. The focused unit covers the 64 KiB serialized JSON-value cap for
lossy/control-heavy streams. The independent-kubectl leased E2E passed 1/1
(36 filtered) in 29.97s with text commands, strict JSON `--timeout 30`, status,
and exact delete. The child receives private credentials, so this does not claim
a credential-free/untrusted-child boundary or crash-safe termination.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// HANDWRITE tracker #1693. Full source lives in apps/vat/tests/vat_k8s_ephemeral.rs.
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/tests/vat_k8s_ephemeral.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Fake-runtime and opt-in real Apple Container K3s session coverage for #1693, including an independently installed PATH kubectl requirement that rejects OrbStack-provided binaries; timeout-bounded JSON leased exec shape/exit/no-raw-replay/no-marker-mutation/private-path-masking; group cleanup and fail-closed crash markers; primary-error-first fixed-label bounded bootstrap diagnostics; focused 4/4 no-mutation session-status API proof plus 1/1 expiry-recheck; Service-only loopback credential-free-child filtering (not OS isolation); TERM-ignoring descendant cleanup; v2 CSPRNG fail-closed identity; v1 storage-only cleanup; and durable cleanup tombstones. Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s. The local-image run loads one already-local Apple alpine:3.20 into one lease, runs a pod with imagePullPolicy=Never, observes its marker log, and completes exact session cleanup only; it does not establish registry-pull generality, persistence, GUI, or Docker Engine/API behavior. The leased run includes text commands, strict JSON exec with --timeout 30, status verification, and exact delete; the Service-forward run includes text plus strict JSON loopback forwarding, confirmed cleanup, and closed local ports. The credentialed exec child is not an untrusted-child boundary; this does not claim a general tunnel, same-UID OS isolation, or crash-safe termination."
```
