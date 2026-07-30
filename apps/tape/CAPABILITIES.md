# Tape Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Tape.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Tape (epic #2434 ordered
service 2, after Lumen run `0723041614`). The machine-readable capability
contract currently lives in `apps/tape/README.md` (`cap_path`); this section
records real-cloud proof runs until the #1848 cap_path relocation lands.
Harness: `acceptance/gcp` (`ACCEPTANCE_APPS=tape`). Runs recorded below that
predate #2705 executed the same harness at its former path
`benchmarks/gcp-operator-acceptance`; the relocation is a rename with no
behaviour change.

### Auth default flip (#2765, 2026-07-28 — closed `AuthMode` enum, proven against a live API server)

`spec.auth` was a free `String` defaulting to `"off"`, so three separate
mistakes all produced a Tape that served open and reported nothing: a
misspelling (`requried` deserialized to a string the render simply did not
match), an omission, and — the path the issue did not name — `TAPE_AUTH` being
derived from *whether a token source was set* rather than from `spec.auth`, so
`auth: required` with no `tokensSecret` rendered a pod with no `TAPE_AUTH` at
all. It is now a closed `AuthMode` enum (`disabled` | `required`) defaulting to
`required`, `TAPE_AUTH` is written unconditionally from `spec.auth`, and a CEL
rule rejects a CR naming both token sources.

A green local suite is not evidence for a CRD: every assertion in it reads YAML
text, never the compiled CEL expression, so a rule that no API server accepts
still passes. Each case below was applied with
`kubectl apply --dry-run=server` against a real API server
(kind `lumen-wave2`, Kubernetes server v1.36):

| Case | `spec` | Exit | API-server response |
|---|---|---:|---|
| CRD installs | — | 0 | `customresourcedefinition.apiextensions.k8s.io/tapes.tape.dev created` — the CEL rule compiles |
| typo rejected | `auth: requried` | 1 | `spec.auth: Unsupported value: "requried": supported values: "disabled", "required"` |
| both sources rejected | `tokensSecret` + `tokensSecretProviderClass` | 1 | `spec: Invalid value: set at most one of spec.tokensSecret … there is no way to tell which registry is actually being served` |
| bare `off` rejected | `auth: off` | 1 | `spec.auth in body must be of type string: "boolean"` — YAML 1.1 read it as `false`, which is exactly why the off-state is spelled `disabled` |
| omission defaults closed | no `auth` | 0 | accepted; defaults to `required` |
| `disabled` accepted | `auth: disabled` | 0 | accepted |
| one source accepted | `tokensSecret` only | 0 | accepted |

The two spellings are not interchangeable: the CRD says `disabled`, the serving
process's `TAPE_AUTH` env var still takes `off`. An env var is never parsed as
YAML, so `off` is safe there and nowhere else.

Because omission now means `required`, every instance profile states its mode
explicitly — `tape k8s instance render --profile dev|staging` emits
`auth: disabled`, `prod`/`template` emit `auth: required` — otherwise a
silently tokenless dev CR would render a pod that refuses to start. All three
appliable profiles were round-tripped through the same API server (exit 0).

With both token sources named the render now emits *neither*, rather than
picking one by precedence. Such a spec cannot come from an API server, so the
loud failure — `TAPE_AUTH=required` with no registry file, and the pod fails
startup — beats silently serving whichever registry a precedence rule chose
while the operator reads the other one.

Local gates: `cargo test -p tape --features operator --test operator`
(17 passed), `cargo test -p tape --test deploy_cli` (4 passed).

**Upgrade note (AC7).** This flip is not backward compatible for existing open
Tape resources. A CR that relied on the old `"off"` default, or that spells
`auth: off`, is rejected on its next apply; it must be edited to
`auth: disabled` (to keep serving tokenless) or given a token source. A CR
already carrying `auth: required` is unaffected. The release carrying this
change must repeat this note.

### Release tape@0.4.11 (2026-07-25, published — binaries + digest-pinned multi-arch GHCR image)

The GKE-proven 0.4.11 candidate shipped. Release run `30114475151`: all five
targets built (`aarch64-apple-darwin`, `x86_64`/`aarch64-unknown-linux-gnu`,
`x86_64`/`aarch64-unknown-linux-musl`), 10 assets attached, and the
`publish ghcr image` job pushed

```
ghcr.io/chrischeng-c4/tape:0.4.11@sha256:5af09a72a9e89edc30090183f7d4ce59f5a146b9229d567a55815253ec8b543a
```

verified by `docker manifest inspect` on the digest with no credentials — an
OCI image index carrying `linux/amd64` + `linux/arm64` (the #2462 tape leg).

The first attempt at this tag failed the image job with `curl: (22) 404` on the
musl tarball: the release matrix had been reverted to gnu-only by the rebase
onto main while `Dockerfile.release` still fetched musl for its glibc-free
`distroless/static-debian12` runtime. Producer and consumer of the release
assets must move together; restored in `c04fe67cdb`, drift gate tracked by
#2563.

### GKE acceptance run 0724164220 (2026-07-24, PASSED — final 0.4.11 candidate, #2557 declarative provisioning proven)

Tape-only run from HEAD `c06504d6e1` (the full 0.4.11 candidate). All 13
proofs `passed` again, and the run's pull/ack legs now execute against a
subscription **pre-provisioned via CR `spec.topics`** with zero imperative
setup (`kubernetes/tape-subscription-cr-provisioned.txt`) — the #2557
dual-path provisioning contract (declarative additive-only ensure alongside
the untouched client API) proven end to end on Standard GKE. Cleanup clean.
Evidence root: `axiom-gcp-run-backup/evidence/0724164220/`.

### GKE acceptance run 0724161853 (2026-07-24, PASSED — 0.4.11 candidate, #2468 restart + #2485 lag gauges proven)

Tape-only run on the unified harness (restored `ACCEPTANCE_APPS=tape` mode
after the app/tape→main rebase; Cloud Build from HEAD `7d063ff3d5`), all 13
proofs `passed` in `tape-acceptance.json` (`axiom.gcp.tape.acceptance.v1`)
and `cleanup.json` `status: clean` (verified `2026-07-24T16:38:13Z`):

- **`bootstrap_seed_uri_restart` (NEW, #2468)**: a pod restart while the CR
  still carries `bootstrapSeedUri` returns Ready with data intact —
  the bootstrap-if-empty fix (`562ff7ecfe`) proven in-cluster; the field is
  declarative bootstrap-if-empty, no longer one-shot.
- **`subscription_lag_gauge` (NEW, #2485)**: `/metrics` serves
  `tape_subscription_lag{topic,subscription}` after the append/consume
  steps (`895d8699cf` scrape-time gauges).
- Re-proven regression base on the 0.4.11-candidate code (which also
  carries #2484 end-to-end body limits and #2483's call-time backup-scheme
  docs): 1x1 reconcile, append/replay lifecycle, subscription pull/ack
  cursor, pod-restart data retention, Workload-Identity GCS backup (635-byte
  object), cold restore from the exact backup object, seed-cleared rolling
  restart retention, 1→3 topology (3 ready), raft leader-pod-replaced
  failover (term 111→113), post-failover write committed.

Evidence root: `axiom-gcp-run-backup/evidence/0724161853/`. Harness
hardening shipped en route (runs t1-t4): mode-aware
render/deploy/verify-clean/operator-cell, the tape backup CronJob restored
as hand-rolled+suspended (the Tape CRD has no CR-native backup field), and
a completion sentinel that makes bash expansion-error aborts exit non-zero
— false-green runs are structurally impossible for BOTH harness modes now.

### GKE acceptance run 0723135853 (2026-07-23, PASSED — full)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped bucket/GSA/Workload
  Identity created and destroyed by the run.
- Image: pinned immutable
  `courier/tape@sha256:05d2a6f0ff3a6de1ba2be2c9566a3c509e75c62025995713e6475a87792bd619`
  built by Cloud Build from commit `b69ad94947` (includes the #2443
  `lost+found` and #2465 genesis-index seed fixes) with features
  `operator backup`.
- Terminal artifacts: `acceptance.json` (`axiom.gcp.tape.acceptance.v1`,
  every claimed proof `passed`) and `cleanup.json` (`status: clean`,
  2026-07-23T14:18:01Z). Evidence root:
  `axiom-gcp-run-backup/evidence/0723135853/` (home-dir mirror of the
  volatile `/tmp` tree); `run.log` carries the full transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease, steady-state drift repair, leader-takeover reconcile | passed | `tape-operator-cell.json`; `kubernetes/tape-lease-holder-*.txt` |
| 1x1 reconcile with status-generation fence on Standard GKE | passed | `kubernetes/tape-crs.json`; `kubernetes/workloads-after-tape-deploy.json` |
| Domain lifecycle through the client Service: append offsets 0-2, replay, subscription create, pull cursor 0→3, cumulative ack, empty re-pull | passed | `kubernetes/tape-append.jsonl`; `kubernetes/tape-replay-initial.json`; `kubernetes/tape-pull-*.json`; `kubernetes/tape-ack.json` |
| Pod-restart data retention on the PVC journal (3 events + checkpoint offset 3 survive `tape-0` replacement) | passed | `kubernetes/tape-replay-after-restart.json`; `kubernetes/tape-checkpoint-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered `tape backup` writes a 635-byte `JournalSnapshot`; readback carries the appended events | passed | `kubernetes/tape-backup.log`; `gcs/tape-first-object.json` |
| Cold restore: the exact GCS object seeds a fresh-PVC 3-replica/3-voter topology via `bootstrapSeedUri`; restored replay shows exactly offsets 0-2 and checkpoint 3 | passed | `kubernetes/tape-restore-cr.yaml`; `kubernetes/tape-replay-after-restore.json`; `kubernetes/tape-checkpoint-after-restore.json` |
| One-shot seed cleared (#2468 contract): field removed post-restore, full no-seed rolling restart retains all events | passed | `kubernetes/tape-replay-after-seed-clear.json` |
| Raft leader disruption: leader pod deleted (grace 1s), group re-elects (term 12→14; the replaced pod legitimately re-won, `distinct: false` recorded honestly), post-failover write commits at offset 3 and replays as the fourth event | passed | `kubernetes/tape-raftz-initial.json`; `kubernetes/tape-raftz-after-failover.json`; `kubernetes/tape-append-after-failover.json`; `kubernetes/tape-replay-after-failover.json` |
| Verified cleanup: run-scoped bucket, GSA, IAM, image tag, namespaces, CRD destroyed; persistent cluster and pre-existing APIs preserved | passed | `cleanup.json` |

Product defects found and fixed by this campaign (each verified by the
passing run above): #2443 (`lost+found` on cloud PVs rejected cold seeds),
#2465 (single-node-origin `up_to: 0` seeds silently restored an EMPTY
group — genesis-index mapping added, regression
`apps/tape/tests/seed_ha_bootstrap.rs`), #2468 (`bootstrapSeedUri` is
one-shot: leaving it on the CR crash-loops any pod replacement; the
operator-side lifecycle decision remains open).

### GKE acceptance run 0723155311 (2026-07-23, PASSED — released GHCR image)

Re-run of the full 8-proof acceptance with the PUBLISHED release image —
`image_provenance: prebuilt`, zero Cloud Build (#2462's acceptance
condition): `ghcr.io/chrischeng-c4/tape@sha256:ca2928c83fd76681924fd419f35d128933c9abbd1da42342062f96b264b10a12`
(the `tape@0.4.10` musl-static release, pulled anonymously from public
GHCR). All eight proofs passed again on the release binary, which also
carries the adoption fixes (#2482 GET-retention contract, #2484 body
limit + bounded replay, #2468 runbook semantics). `cleanup.json`
`status: clean` (2026-07-23T16:04:36Z). Evidence root:
`axiom-gcp-run-backup/evidence/0723155311/`.

Exclusions (recorded, not claimed): shard migration (`shardCount` pinned
to 1), live in-place replica membership change (startup-static
membership), and CPU/memory pressure actuation — product gaps tracked in
#2437. Earlier partial runs' evidence (0723080156 six-proof subset and the
intermediate diagnosis runs) is retained under the same backup root.
