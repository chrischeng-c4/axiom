---
id: vat-source-projects-vat-src-commands-k8s-rs
summary: Hand-written Apple Container one-boot K3s command with one-shot, bounded leased agent sessions, bounded bootstrap diagnostics, verified local-image delivery, hardened loopback Service port-forward recovery, and independently installed non-OrbStack kubectl provenance.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: headless-ephemeral-kubernetes-session
    coverage: partial
    rationale: "#1693 owns a bounded agent-facing local Kubernetes session without presenting a persistent Docker or Desktop backend."
---

# Source mirror: apps/vat/src/commands/k8s.rs

## Overview
<!-- type: overview lang: markdown -->

This hand-written command is intentionally separate from the Docker-backed
cluster subsystem. It builds an asset-revision-tagged Phase-0-proven systemd image
only on explicit request, then uses one exact owned Apple machine and its
inspect-returned backing container to bootstrap K3s. `ephemeral run` exposes it
to one foreground host command; `session create/exec/port-forward/image/status/delete` preserves a
bounded private credential directory across explicit agent calls. K3s commands
run only with an independently installed `kubectl` first on `PATH`; VAT rejects
an OrbStack-provided binary before K3s use. This is host-tool provenance, not a
GUI or Docker Engine dependency. Homebrew `/opt/homebrew/bin/kubectl` is now
installed locally. Independent-kubectl one-shot, leased, local-image, and
Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and
49.57s respectively. The local-image E2E loaded one already-local Apple
`alpine:3.20` into one lease, ran a pod with `imagePullPolicy=Never`, observed
its marker log, and completed exact session cleanup; it is not registry-pull
generality, persistence, GUI, or Docker Engine/API evidence. A lease can also
`image load` one already-local `linux/arm64` reference through a private,
bounded OCI archive into K3s `k8s.io`, then verifies the canonical reference and
removes both archive copies. Its `session port-forward run` path accepts only one
literal Service and runs one host child through a temporary `127.0.0.1` tunnel;
VAT strips K3s credential variables and VAT_HOME from that child environment
while kubeconfig stays with kubectl. The child joins kubectl's authenticated
process group, so normal cleanup reaps the leader and confirms ordinary
cooperative, non-daemonizing descendants are gone; daemonizing or escaping the
group is outside the contract. That is credential hygiene, not a same-UID OS
sandbox or adversarial-child security boundary. Every leased operation
revalidates backing id/API endpoint and host API reachability. A retained private
0600 `operation.lock` serializes a session operation and is opened `CLOEXEC`, so
kubectl/host work cannot retain the flock after a SIGKILLed VAT parent. Once a
later mutating operation holds that lock, it may reconcile without trusting the
recorded owner PID: a v2 marker's CSPRNG private token plus exact loopback-forward
shape authenticates the only leader that may be signalled. Missing, changed, or
ambiguous leader identity fails closed and retains the marker. A durable
`cleaning` tombstone makes torn storage unlink retryable. Pre-CSPRNG v1 markers
are never signalled and can receive storage-only cleanup only after their recorded
group is already absent. Private kubeconfig/cache/home paths never leave child
environments; cleanup failure overrides a successful child exit, and no command
claims restart-safe persistence. On bootstrap failure, the root error remains
primary, then VAT emits staged non-sensitive installer evidence through exactly
`guest_install_log`, `guest_k3s_system`, `backing_container_logs`,
`machine_boot_log`, `machine_inspect`, and `container_system_status`. Those
fixed read-only probes share a six-second total and one-second-per-probe budget,
exclude private kubeconfig/cache and host credentials, and finish before the
same exact cleanup. They do not repair or retry bootstrap, alter its existing
300-second behavior, rerun `k3s --version`, add a wrapper/recovery command, or
turn this into persistent Kubernetes. The deterministic fake regression passed.
Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es
each passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s. The
local-image result loads one already-local Apple `alpine:3.20` into one lease,
runs a pod with `imagePullPolicy=Never`, observes its marker log, and completes
exact session cleanup; it is not registry-pull generality. The Service-forward
result covers one Service-only loopback text and strict JSON tunnel with a
credential-free child, confirmed cleanup, and closed local ports; it does not
establish persistence or same-UID OS isolation.

Text `session exec` and optional `vat k8s session exec --format json [--timeout
SECONDS] <id> -- COMMAND` validate the active unexpired lease, exact backing
identity/API endpoint, private credentials, and owned host API under the private
operation lock. Omitted timeout uses remaining lease TTL; explicit timeout is
1..=14400 seconds and cannot exceed it. VAT rechecks expiry before child spawn,
owns/reaps the process group on normal exit, deadline, or SIGINT/SIGTERM, and
holds the lock through cleanup. A starting/live crash marker blocks later
exec/delete/cleanup fail-closed rather than claiming recovery termination. JSON
then produces one `vat.k8s.session.exec.v1` `vat_json` document with its numeric
exit code, separate concurrently drained stdout/stderr snapshots, per-stream
truncation/lossy-UTF-8 indicators, `api_verified=true`, `runtime_invoked=true`,
and `session_record_mutated=false`. Each serialized JSON string value is bounded
to 64 KiB; raw child output is not replayed, private credential/cache paths are
masked including validation/API-probe failures, and `session.json` is unchanged.
The child does receive private credentials, so this is not a credential-free or
untrusted-child security boundary. The leased real-host E2E passed 1/1
(36 filtered) in 29.97s with text commands, strict JSON exec using `--timeout
30`, status verification, and exact delete.

Text `session port-forward run` forwards the foreground child stream and starts
its terminal record on a new line afterward. Its only JSON form, `vat k8s session port-forward run --format json
<id> service/<name> <port> -- COMMAND`, remains Service-only and loopback-only.
The host child receives no K3s credentials, joins kubectl's authenticated process
group, and the private operation lock stays held through tunnel/group cleanup.
VAT silently rechecks the lease after owned API proof and immediately before the
exact kubectl and host-child spawns, so a crossed expiry opens no tunnel. Only
after cleanup and marker removal are confirmed does JSON emit one
`vat.k8s.session.port-forward.v1` `vat_json` document with child exit, separate
64 KiB serialized-capped stdout/stderr, truncation/lossy flags, no raw replay,
and `status --verify-api` next. VAT-owned setup/API/tunnel/cleanup failures are
masked; opaque credential-free child output in a successful result is not
arbitrarily redacted. A partial reader setup reaps the direct child and completes
outer group cleanup before reader join. The independent-kubectl Service-forward
E2E passed 1/1 (36 filtered) in 49.57s, including one Service-only loopback
strict JSON tunnel, confirmed cleanup, and closed local ports; it is not a
general tunnel or persistent-Kubernetes claim.

No-flag `session status` remains an observation-only non-secret lease/machine
report. `session status --verify-api` is a bounded proof only for an active,
unexpired lease with no retained port-forward or exec marker: it takes the same private
operation lock, rechecks expiry after lock and immediately before the probe,
validates exact backing identity/endpoint plus private credentials, and reports
`api_checked=true`, `api_state="reachable"` only after one bounded exact-owned
API probe. Expired/recovery state is non-probing
`api_checked=false`, `api_state="not_checked"`; busy, unavailable, and
identity-mismatched state fails closed without lease or credential mutation.
Focused fake coverage passed 4/4 and the precise expiry-recheck unit passed
1/1. The independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s and
includes `status --verify-api` after text and strict JSON exec. This is bounded
active-lease evidence only, not a persistent K8s, general API-status, GUI, or
Docker Engine surface.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|------|
| EphemeralRunArgs | apps/vat/src/commands/k8s.rs | struct | pub | image + host command |
| ActiveSessionCreateArgs | apps/vat/src/commands/k8s.rs | struct | pub | image + bounded TTL |
| ActiveSessionImageLoadArgs | apps/vat/src/commands/k8s.rs | struct | pub | id + local image + platform |
| ActiveSessionPortForwardArgs | apps/vat/src/commands/k8s.rs | struct | pub | id + Service selector + ports + one host command |
| build_default_image | apps/vat/src/commands/k8s.rs | function | pub | build_default_image() -> Result<ExitCode> |
| ephemeral_run | apps/vat/src/commands/k8s.rs | function | pub | ephemeral_run(args: EphemeralRunArgs) -> Result<ExitCode> |
| cleanup_abandoned | apps/vat/src/commands/k8s.rs | function | pub | cleanup_abandoned(json: bool) -> Result<ExitCode> |
| session_create | apps/vat/src/commands/k8s.rs | function | pub | session_create(args: ActiveSessionCreateArgs) -> Result<ExitCode> |
| session_exec | apps/vat/src/commands/k8s.rs | function | pub | session_exec(id, command, json) -> Result<ExitCode> |
| session_port_forward | apps/vat/src/commands/k8s.rs | function | pub | session_port_forward(args: ActiveSessionPortForwardArgs) -> Result<ExitCode> |
| session_image_load | apps/vat/src/commands/k8s.rs | function | pub | session_image_load(args: ActiveSessionImageLoadArgs) -> Result<ExitCode> |
| session_status | apps/vat/src/commands/k8s.rs | function | pub | session_status(id) -> Result<ExitCode> |
| session_status_verify_api | apps/vat/src/commands/k8s.rs | function | pub | session_status_verify_api(id) -> Result<ExitCode> |
| session_delete | apps/vat/src/commands/k8s.rs | function | pub | session_delete(id) -> Result<ExitCode> |
| session_cleanup | apps/vat/src/commands/k8s.rs | function | pub | session_cleanup(json) -> Result<ExitCode> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// HANDWRITE tracker #1693. Full sources live in apps/vat/src/commands/k8s.rs
// plus apps/vat/src/commands/k8s/session_exec.rs and port_forward_json.rs.
// Public entrypoints: build_default_image, ephemeral_run, cleanup_abandoned,
// session_create, session_exec(id, command, json), session_port_forward, session_image_load,
// session_status, session_status_verify_api, session_delete, session_cleanup.
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/commands/k8s.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: "Bounded Apple K3s command, one-shot and leased private credential lifecycles, and an independently installed PATH kubectl requirement that rejects OrbStack-provided binaries before K3s use. Omitted session-exec timeout uses remaining TTL; explicit 1..=14400 seconds cannot exceed it. Every exec owns/reaps its process group and holds the private lock through cleanup; a starting/live crash marker blocks later exec/delete/cleanup fail-closed rather than claiming termination. JSON exec has bounded concurrent stream capture, private-path masking, and no record mutation; post-cleanup JSON Service forward is Service-only loopback with credential-free child, silent expiry checks, separate 64 KiB streams, masked VAT-owned errors, and no raw replay. Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s. The local-image result is one already-local Apple alpine:3.20 pod with imagePullPolicy=Never, a marker log, and exact session cleanup only; it does not establish registry-pull generality, persistence, GUI, or Docker Engine/API behavior. JSON exec evidence is strict --timeout 30 only, and JSON tunnel evidence is one Service-only loopback session with confirmed cleanup and closed local ports."
  - path: apps/vat/src/commands/k8s/session_exec.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: "Own VAT-native leased-session JSON exec: omission uses remaining TTL and explicit 1..=14400 seconds cannot exceed it; private lock remains through owned-process-group cleanup; crash marker blocks later lifecycle operations fail-closed rather than claiming termination. JSON captures concurrent 64 KiB serialized streams, preserves child exit, masks private paths, and does not mutate session.json. The credentialed child is not an untrusted-child boundary. The independent-kubectl E2E passed 1/1 (36 filtered) in 29.97s with strict JSON --timeout 30, status verification, and exact delete."
  - path: apps/vat/src/commands/k8s/port_forward_json.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: "Own post-cleanup concurrent JSON capture for the credential-free Service-forward host child: 64 KiB serialized stream caps, no raw replay, and reader joins deferred until direct-child reap plus outer shared-group cleanup. The independent-kubectl E2E passed 1/1 (36 filtered) in 49.57s for one Service-only loopback strict JSON tunnel with confirmed cleanup and closed local ports."
```
