# Lumen Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Lumen.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

## Operations Baseline Evidence

Proof for the `CONTRIBUTING.md` **Operations baseline** rows as they land on
Lumen (the ordered rollout `lumen → tape → defer → relay → keep → sift`).
Unit/contract proof is recorded here as it lands; the cluster half of each
row is proven by the GKE acceptance runs recorded below.

### Row 5 — `status.conditions[]` convergence API (#2601, 2026-07-25)

Lumen's `LumenStatus` now carries the Kubernetes-convention
`metav1.Condition` array — the shape `kubectl wait --for=condition=Ready`,
Argo CD health assessment, and Flux readiness gates already read — so an
integrator no longer has to poll and re-derive convergence from the flat
status fields. Three condition types are emitted: `Ready`, `Progressing`,
and Lumen's own `ReshardInProgress`. `kubectl get lumen -o wide` gains a
`Converged` printer column projecting
`.status.conditions[?(@.type=="Ready")].status`.

The mechanism is shared, not Lumen-local: `libs/service-k8s` gained
`Condition` / `ConditionStatus` / `ConditionFact` plus the `project()`
transition-time carry-forward, and two default-implemented `ManagedService`
hooks (`conditions`, `observed_conditions`) so the other five services keep
their status shape byte-for-byte until they opt in.

Two design constraints are pinned by tests rather than by comment:

- **Clock purity.** `status_patch` is synchronous and does no I/O by
  contract (`reconcile.rs:12`) and ~7 determinism tests depend on that, so
  `lastTransitionTime` is never stamped inside it. The service returns
  clock-free `ConditionFact`s and the already-async reconcile loop injects
  the time (`controller.rs` step 3b). Test:
  `conditions_are_a_pure_function_of_spec_and_observed_facts`.
- **Transition times survive `Patch::Merge`.** Merge replaces arrays
  wholesale, so prior times are read off the watched object via
  `observed_conditions()` and re-sent. Test:
  `observed_conditions_round_trip_through_the_projection`.

Both status surfaces derive from one `Observation`, so they cannot drift —
`the_flat_status_and_the_conditions_agree_on_readiness` pins `phase ==
"Ready"` iff `Ready` is `True` across 0/1/2 ready pods. `Ready` is gated on
genuine wedges only (`RESHARD_WEDGE_CONDITIONS` = `reshardOversizedDocument`,
`topologyConvergenceStalled`), never on the raw `blockingConditions` list —
`maxShardBytesUnset` is present on every default install, and gating on it
would report every default Lumen as permanently not-ready
(`a_fully_ready_default_cr_reports_ready_true` is that regression test).
The post-cutover write-pause fence being armed at phase `Complete` reports
`ReshardInProgress=True` while `Ready` stays `True`; only a *stalled* fence
demotes `Ready`.

Evidence: `cargo test -p service-k8s` 40 passed / 0 failed;
`cargo test -p lumen --lib --features operator operator::` 55 passed /
0 failed (27 in `operator::reconcile`, including 8 new condition tests);
`cargo clippy -p lumen --features operator --lib` clean. The cluster half —
printer column, `Progressing → Ready`, `observedGeneration` catch-up — is
proven in "Wave 1 cluster proof" below.

### Row 7 — operator HA floor (#2602 + #2532, 2026-07-26)

The control plane ran a single replica, so every voluntary disruption — a
node drain, an eviction, a rollout — left every Lumen CR in the cluster
unreconciled for the length of a pod restart. Leader election was already
built and simply unused: `libs/service-k8s/src/lease.rs` plus the `Election`
gate in `controller.rs` mean only the Lease holder applies, so raising the
floor is safe rather than a correctness change.

- `k8s/operator/deployment.yaml`: `replicas: 2`. The standby watches and
  reconciles nothing until it wins the Lease.
- `k8s/operator/pdb.yaml` (new): `maxUnavailable: 1`. Two replicas only
  survive a drain if evictions are serialized — without a PDB the node
  drain that motivated the second replica can take both at once.
- Anti-affinity is `preferredDuringScheduling`, deliberately not
  `required`: a required rule parks one replica `Pending` forever on a
  single-node kind/minikube cluster, which would make the failover proof
  itself impossible to run locally. Pinned by an assertion that
  `requiredDuringSchedulingIgnoredDuringExecution` stays null.
- Batched with **#2532** (same file, so not two PRs colliding): the manifest
  pinned `0.4.24` while the workspace shipped `0.4.25`, handing every
  kustomize consumer a release-old image. The fix is structural, not a
  one-off bump — `render_operator_yaml` now derives the pin it expects from
  `env!("CARGO_PKG_VERSION")` and *fails the render* when the checked-in
  manifest disagrees, so the next release that forgets this file cannot ship
  quietly.
- `lumen k8s operator render` emits the PDB alongside the Deployment, under
  `--namespace` too, so the CR consumers and the kustomize consumers install
  the same operator layer instead of diverging again.

Evidence: `cargo test -p lumen --features operator --test
operator_backup_kubernetes_wiring` 9 passed / 0 failed, including four new
gates — `operator_deployment_runs_two_leader_elected_replicas`,
`operator_pdb_serializes_eviction_across_the_replicas` (which asserts the
PDB selector equals the Deployment selector, so a silently inert PDB fails),
`operator_render_and_static_manifest_agree_on_the_ha_shape`, and
`operator_manifest_pins_this_workspaces_version`. The three checked-in
manifests were verified byte-exact against their SPEC-MANAGED source
(`tech-design/semantic/lumen-k8s-operator.md`). The failover half — kill the
Lease holder, confirm the peer takes over and reconciliation resumes — is
proven in "Wave 1 cluster proof" below.

### Row 6 — per-instance network isolation (#2603, 2026-07-26)

Lumen had no isolation on either install path, so any pod in the cluster
could open a socket to a serving pod's Raft port (7374) and speak to the
consensus layer directly. Both paths now get a default-deny posture, with
the deny expressed as the two peer classes a sharded service actually has:
the client API is cluster-open, everything else is instance-scoped.

- `libs/service-k8s/src/render/common.rs` (new `network_policy()`): the
  shared shape, so the five services behind Lumen inherit it rather than
  hand-rolling five near-identical policies. Ingress splits into a
  `namespaceSelector: {}` rule carrying only `client_ports` and a
  `podSelector` rule carrying `peer_ports`; egress is DNS over both
  transports (a truncated UDP answer retries over TCP) plus outbound TLS,
  with plaintext `:80` deliberately not granted.
- `spec.networkPolicy: bool` on the CRD, defaulting **off**. Opt-in because
  a NetworkPolicy is inert unless the CNI enforces it — GKE needs Dataplane
  V2 or the Calico add-on, and a default kind cluster (kindnet) accepts the
  object and enforces nothing. Defaulting on would have shipped isolation
  that silently does nothing on the clusters most likely to test it.
- `k8s/components/network-policy/` (new) for the direct kustomize install,
  composed by the staging and prod overlays only. The dev overlay stays on
  a vanilla cluster. The direct install is single-node embedded mode, so
  that policy admits 7373 and no peer port at all.
- `plural_for` in `controller.rs` gained `"NetworkPolicy" => "networkpolicies"`.
  The naive `lower(kind) + "s"` fallback yields `networkpolicys`, a plural no
  apiserver serves — every apply would have 404'd at runtime with nothing
  failing at build time. Pinned by a test that asserts the derived and
  declared plurals genuinely differ for this kind.
- RBAC grants `networking.k8s.io/networkpolicies`. `networking.k8s.io` is a
  built-in group, so the grant resolves even where the CNI ignores the
  objects.

Evidence: `cargo test -p service-k8s --lib` 46 passed / 0 failed (was 40),
the six new gates being four render tests — including
`peer_ports_are_never_reachable_from_outside_the_instance`, the central
claim, and `extra_egress_is_appended_not_substituted`, which pins that a
caller-supplied rule cannot silently drop the DNS/TLS baseline — plus two
`plural_for` tests. `cargo test -p lumen --features operator --test
operator_render --test operator_backup_kubernetes_wiring` 40 + 9 passed / 0
failed; `rendered_network_policy_is_opt_in_and_never_exposes_the_raft_port`
asserts the field defaults false, that nothing renders while off, and then
with it on: ownership/namespace, an instance+component podSelector, exactly
`[7373]` on the cluster-facing rule, an instance-scoped 7374 peer rule (two
Lumen CRs in one namespace cannot reach each other's Raft ports), and that
the backup CronJob's own component label is not the selected one. Static
wiring proven with `kubectl kustomize`: prod and staging each emit exactly
one NetworkPolicy with `namespace: lumen` applied by the overlay
transformer, dev emits none. The enforcement half is proven on cluster
below.

### Wave 1 cluster proof (rows 5, 6, 7 — 2026-07-26)

The three Wave 1 rows share one cluster session. Substrate: a two-node kind
cluster with `disableDefaultCNI: true` and **Calico v3.30.2**, running the
operator from a `lumen dockerfile render --variant source` image. The CNI
choice is the point of the exercise — kindnet accepts NetworkPolicy objects
and enforces nothing, which would have made the row 6 denial pass
vacuously. GKE could not host this proof: the acceptance cluster reports
`addonsConfig.networkPolicyConfig.disabled = true` and no Dataplane V2, and
enabling either would have forced a node re-creation on a shared cluster.

> Trap for the next person building this cluster: Calico's documented pool
> `192.168.0.0/16` **cannot** be used under OrbStack, whose `kind` docker
> network is `192.168.97.0/24`. The pool swallows the node addresses and
> every pod→apiserver connect times out with `client error (Connect)` while
> the nodes still look `Ready`. Pin `podSubnet: 10.244.0.0/16` and set
> `CALICO_IPV4POOL_CIDR` to match.

**Row 5 — conditions API.** `kubectl get lumen -o wide` renders the
`Converged` column, and all three condition types carry the full
`metav1.Condition` shape. Transition-time carry-forward is confirmed
against a live apiserver, not only in the unit test: after the reconcile
that flipped `Ready`, `ReshardInProgress` still held its original
`01:48:25Z` while `Ready` had moved to `01:49:25Z` — had `Patch::Merge`
array replacement won, all three would carry one timestamp. Driving the CR
out of convergence and back (`replicasPerShard` 2→3→2; the third pod is
unschedulable, so "not converged" is durable rather than a blip):

| t | gen | observedGeneration | phase | ready/desired | Ready | Progressing | reason |
|---|---|---|---|---|---|---|---|
| pre | 2 | 2 | Ready | 2/2 | True | False | Converged |
| after 2→3 | 3 | 3 | Reconciling | 2/3 | **False** | **True** | ReplicasConverging |
| after 3→2 | 4 | **3** | Reconciling | 2/3 | False | True | ReplicasConverging |
| +4s | 4 | **4** | Ready | 2/2 | **True** | **False** | Converged |

`Ready.lastTransitionTime` moved on each flip (`01:54:02Z`, `01:55:44Z`)
and stayed put otherwise. Row 3 is the one an integrator most needs:
`generation` 4 with `observedGeneration` 3 makes "the operator has not read
your edit yet" directly observable.

**Known limit, found here.** `controller.rs` does not `.owns()` any child
kind — it watches the CR and requeues on a 30s period — so condition
freshness is bounded by that period, not event-driven on pod readiness. A
first attempt to observe `Progressing` by bumping `serving.cpu` recorded
nothing: the one-pod rolling restart began and finished inside a single
requeue window, so no reconcile ever saw the intermediate `1/2`. The
conditions are correct; a state living less than one requeue period is not
observable through this API by design.

**Row 7 — operator HA.** Both replicas run; one holds all three Leases
(`lumen-operator`, `lumen-hpa-handoff`, `lumen-reshard-driver`) and the
standby idles. Force-killing the Lease holder three times, the **warm
standby won 3/3** with a leaderless window of 15.3s / 16.7s / 16.3s —
`LEASE_DURATION_SECS` (15) plus up to one 5s poll phase. Reconciliation
resumes rather than merely leadership: a spec edit applied while the
cluster had *no* leader (generation 6) was observed by the new leader 20s
later.

> Trap: measure this only on a settled cluster, with
> `--grace-period=0 --force`. A gracefully deleted leader keeps renewing
> its Lease throughout its termination window, and a replacement pod gets a
> free acquisition attempt the instant it starts — which lands within
> ~1s of the expiry moment. Unsettled runs make it look like the standby
> never wins and that `replicas: 2` buys nothing; it does.

**Row 6 — network isolation.** The CR renders exactly one NetworkPolicy
with the instance+component podSelector. Enforcement was measured as an A/B
against the same target, with the *elapsed time* recorded — without it a
policy drop is indistinguishable from a port nothing listens on
(`exit=0, 0s` = allowed; `exit≠0, 0s` = TCP refused, so the packet arrived;
`exit≠0, 5s` = silently dropped):

| source → target | policy ON | policy OFF | reading |
|---|---|---|---|
| unrelated namespace → svc:7373 | exit 0, 0s | exit 0, 0s | client API cluster-open |
| same ns, non-serving pod → svc:7373 | exit 0, 0s | exit 0, 0s | client API cluster-open |
| serving-labelled pod → svc:7373 | **drop 5s** | exit 0, 0s | egress restricted by policy |
| unrelated namespace → pod:7374 | **drop 5s** | refused 0s | **Raft port denied** |
| same ns, non-serving pod → pod:7374 | **drop 5s** | refused 0s | **Raft port denied** |
| serving-labelled pod → pod:7374 | refused 0s | refused 0s | **peer admitted** |

Every cell that changed moved in the direction the policy predicts, which
is what establishes that Calico is genuinely enforcing rather than the
traffic failing for an unrelated reason. The last two rows are the
controlled pair that matters: same destination IP, same port, same instant
— only the *source pod's labels* differ, and that alone decides dropped
versus delivered. The probe was cross-node (control-plane → worker), so
this is Calico's inter-node path, not just a local one. Row 3 was not
predicted: a probe wearing the serving labels is *selected* by the policy,
so its own egress narrows to 7374/DNS/TLS — the egress half proving itself
by refusing a port the contract never granted. Deleting the CR removed the
NetworkPolicy along with the Services and StatefulSet, confirming the
`ownerReference` GC path.

**Gap found here, and closed.** Flipping `spec.networkPolicy` back to `false`
did **not** remove an already-rendered NetworkPolicy: render stopped emitting
it, but server-side apply reconciles *fields*, never object lifetime, so the
live object survived until the CR was deleted and kept enforcing. The
isolation contract was one-way — opt-in took effect, opt-out did not.

Closed by giving `ManagedService` a `prunes()` default method and having the
shared controller delete what it names, guarded by the live object's
controller `ownerReference.uid`. Ownership, not the name, is the license to
delete: a name in a namespace is not proof of authorship, but only the
apiserver writes that uid link.

### Wave 1 follow-up proof — NetworkPolicy off-switch (2026-07-26)

Same kind cluster, image `lumen:w2proof`.

| Phase | Setup | Result |
|---|---|---|
| A | `networkPolicy: true` | policy exists, `ownerReferences[0] = Lumen/search`, `controller=true` |
| B | patch to `false` | deleted within one requeue; operator logs `prune: deleted a child the spec no longer asks for` |
| C | hand-made NetworkPolicy named `search`, no owner, flag still `false` | **survives 70 s (3 reconciles)**; each logs `not controller-owned by it — leaving it alone` |

Phase C is what separates a prune from a delete-anything-at-this-name bug: it
runs with the flag *off*, so the operator is actively targeting that exact
name on every requeue and choosing not to act.

Object deletion is not the contract, though — enforcement stopping is. Probed
from an unrelated namespace against the Raft port (7373 is deliberately open
to every namespace, so it can prove nothing here):

| `spec.networkPolicy` | probe → `10.244.3.200:7374` | reading |
|---|---|---|
| absent | `exit=1` in **0 s** | TCP refused — packet arrived |
| `true` | `exit=1` in **5 s** | silently dropped — enforced |
| `false` (pruned) | `exit=1` in **0 s** | packet arrives again — enforcement stopped |

The `0s → 5s → 0s` swing is the proof: elapsed time, not exit code,
distinguishes "dropped" from "refused". Both fail; only one of them means the
policy is doing something.

### Bug found while proving the above — Raft peer DNS ignored the CR name (#2610, 2026-07-26)

The `voterCount: 2` instance from the Wave 1 table never elected a leader.
The first guess — an even voter set deadlocking on a split vote — was wrong:
`raft-core` staggers `election_timeout` per node id specifically so that
cannot happen. The real cause was one layer down, and it is not lumen's.

`ClusterTopology::from_env_with_scheme` built peer URLs from a prefix the
**caller** passed, and every caller passed its own binary name as a literal.
An operator names the StatefulSet after the custom resource, so a CR named
`quorum` produced pods `quorum-0`/`quorum-1` while the binary addressed
`lumen-1.quorum-headless` — NXDOMAIN. No `RequestVote` was ever delivered, so
both voters campaigned forever.

`GET /debug/cluster` carried the contradiction in a single response, which is
what makes this findable without cluster access:

```json
{ "pod_name": "quorum-0", "role": "candidate",
  "peers": [ { "pod_name": "lumen-0", "host": "lumen-0.quorum-headless" },
             { "pod_name": "lumen-1", "host": "lumen-1.quorum-headless" } ] }
```

It knows it is `quorum-0` and calls its peers `lumen-N` in the same object.
Meanwhile the CR reported `Ready 2/2 CONVERGED=True`, because readiness and
`status.phase` never consult Raft — the health signal was green throughout a
total, permanent loss of quorum.

**Blast radius is the shared library, not lumen.** `lumen`, `tape`, `defer`,
`relay`, and `keep` all call the same constructor with the same hardcoded
literal. Any of them is silently unable to form a quorum whenever the CR is
not named exactly after the binary — which is the normal case, since the
acceptance fixtures happen to name theirs after the binary and so never hit
it.

Fixed at the root: the prefix now comes from `ClusterDims::pod_prefix()` —
the half of `POD_NAME` that `pod_ordinal()` was already parsing and throwing
away. A pod trusts that string's ordinal to decide *who it is*, so the prefix
from the identical parse is exactly as trustworthy for deciding *who to
call*. The caller's argument survives as a fallback only. All five services
are fixed without touching four of them, and deployments where the two
already agreed produce byte-identical URLs.

Regression tests assert the peer **URL**, not a boolean, because the failure
was never an error — it was a well-formed address for a host that does not
exist: `peer_dns_prefix_follows_the_pod_not_the_callers_binary_name`
(raft-runtime) and `raft_group_peer_dns_follows_the_pod_not_the_binary_name`
(lumen, covering the second derivation behind `/debug/cluster` and
read-consistency routing).

**Proof, on the same `voterCount: 2` CR named `quorum`:**

| | before | after |
|---|---|---|
| peers named | `lumen-0`, `lumen-1` (NXDOMAIN) | `quorum-0`, `quorum-1` |
| roles after 15 min | `Candidate`, `Candidate` | `Leader` + `Follower` in **1.8 s** |
| `PUT`/`POST` a document | no leader — cannot commit | `{"indexed":3}` |
| `applied_index` | `0` / `0`, frozen | `4` / `4`, in lockstep |
| same query on the follower | — | `{"hits":[{"external_id":"d3",…}],"total":1}` |

Election is not the contract — committing is, so the run drives a real write
and reads it back from the *other* member. Two controls keep that from being a
false green. An idle 20 s window first confirms `applied_index` is not a timer
(it held at 2, then moved only on the create and the index). And the follower's
default `leader`-consistency read is *refused* — `read_consistency_not_leader`,
naming `quorum-0` — which is itself the fix working: before, there was no
leader to name. Re-asking with `x-read-consistency: any` returns a document the
follower never received directly, so the log replicated *and* the state machine
applied it.

This is now a **permanent leg of the GKE acceptance**, not a one-off kind
session: `verify-lumen.sh` applies a `lumen-quorum` CR whose name differs from
the binary on every `lumen sift` run and fails the run if the peer list, the
leader/follower split, or the follower read regresses — closing the fixture
blind spot that let the bug through. First green: run `0726053353` below.

`replication_lag_ms` is deliberately not cited above. `lumen.rs:2676` stores
`0` on a leader and `u64::MAX` on anything else unconditionally, because
`RaftHost` exposes no peer-timing RPC and a fabricated figure would be worse
than an honest sentinel (#1349, documented at `api.rs:919-924`). It therefore
restates "am I leader" and can never evidence replication — `applied_index`,
read off the raft applied watch channel, is the field that can.

One observation independent of Wave 1, recorded so it is not re-discovered:
serving pods carry `requiredDuringSchedulingIgnoredDuringExecution` hostname
anti-affinity, which the scheduler enforces *symmetrically* — any pod wearing
the serving labels is refused on a node already running one — so
`replicasPerShard: N` needs N schedulable nodes.

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Lumen (epic #2434 ordered
service 1, before Tape run `0723135853`). The machine-readable capability
contract currently lives in `apps/lumen/README.md` (`cap_path`); this
section records real-cloud proof runs until the #1848 cap_path relocation
lands. Harness: `benchmarks/gcp-operator-acceptance` (mode noted per run).

### GKE acceptance run 0726053353 (2026-07-26, PASSED — #2610 peer DNS on a managed cluster + Wave 1 rows 5/7)

Full two-service digest-mode run (GHCR `sha-7745ba935d20` images, zero Cloud
Build; harness `ACCEPTANCE_APPS='lumen sift'` at `a2a45ea9b3ea`). Every prior
leg re-passed — reconcile 1×1, pod-restart retention, admission exposure, GCS
backup before split, cold restore onto a fresh PVC, seed-set restart
retention, auto-split 1→2, and the #2457 auth+CSI stack — alongside the full
Sift phase (CRI collector, Lumen structured stdout materialized, scheduled
backup) and both operator cells.

**#2610 on a managed cluster.** The kind proof above used a CR named `quorum`;
this run adds the case the acceptance fixtures had never covered, which is why
the bug survived them. `lumen-quorum` — `replicasPerShard: 2`, `voterCount: 2`,
CR name deliberately ≠ the binary name — reported:

```json
{"issue":"#2610","cr":"lumen-quorum","leader":"lumen-quorum-0",
 "follower":"lumen-quorum-1","replicated_read":"passed"}
```

The leg asserts the peer list is exactly `["lumen-quorum-0","lumen-quorum-1"]`
(the pre-fix build produced `lumen-0`/`lumen-1` → NXDOMAIN), then requires one
`leader` and one `follower`, writes a document through the leader, and reads it
back **off the follower** with `x-read-consistency: any`. Election alone would
not have been proof; committing and replicating is. Teardown removes the CR,
StatefulSet, and both `raft-lumen-quorum-*` PVCs.

Why the acceptance was blind to this: multi-member CRs were named after the
binary (`lumen/lumen`, `tape/tape`), and the differently-named CRs
(`lumen-restore`, `lumen-authcsi`) are single-replica, where `--wal auto`
picks embedded WAL and no peer is ever addressed. Neither had both properties
at once.

**Row 5 (#2601) and row 7 (#2602/#2532) re-confirmed on GKE**, not only kind.
The `lumen` CR carried all three conditions with distinct carry-forward
timestamps (`Ready=True/AllReplicasReady` and `ReshardInProgress` holding
`05:35:02Z` while `Ready` moved to `05:35:32Z`), and
`lumen-system/lumen-operator` ran `spec.replicas=2` / `readyReplicas=2` on the
pinned digest — with the lease genuinely changing hands between two
simultaneously-live pods (`…mbz67` → `…j4pj7`), which a single-replica
deployment cannot exhibit.

**Row 6 (#2603) is deliberately absent from this run.** The acceptance cluster
reports `addonsConfig.networkPolicyConfig.disabled = true` and no Dataplane V2
(re-verified this run), so it would have *accepted* a NetworkPolicy and
enforced nothing — a green with no meaning. That row stays proven on kind +
Calico above, and this is recorded so nobody "fixes" the gap by applying the
object here.

Cleanup: `Destroy complete! Resources: 9 destroyed`, and zero buckets, service
accounts, secrets, or namespaces bearing the run id remain. Evidence root:
`/tmp/axiom-gcp-operator-evidence/0726053353/`
(`kubernetes/lumen-quorum-*`, `lumen-acceptance.json`).

> **Harness trap that cost a run.** The first attempt, `0726052225`, aborted at
> `terraform apply` having created nothing. Two defects, both introduced by the
> tape-mode refactor `ce6635f57a` and both reachable **only** in the default
> `lumen sift` mode — which is why the two tape runs that followed stayed green
> and hid them: (1) `terraform_apply_var_args` was populated only in tape mode
> and then expanded bare, and macOS bash 3.2 under `set -u` treats an *empty*
> array expansion as an unbound variable; (2) `LUMEN_AUTHCSI_SECRET_ID` lost its
> assignment and export while both readers survived. Fixed in `a2a45ea9b3`.
> Before re-running, the lumen path was re-proven without spending cloud time:
> `terraform validate` plus a read-only `terraform plan` (the cluster is a data
> source, so the stack only creates bucket/secret/bindings), and a static
> unbound-variable scan over the whole call chain — itself validated by
> confirming it flags defect 2 on the pre-fix tree. Second trap, on the
> operator side: invoking `run.sh | tee` masks its exit code, so the aborted run
> was *reported* as exit 0 even though the harness's `run_completed` sentinel
> had correctly forced a failure. Redirect, don't pipe.

### GKE acceptance run 0724105144 (2026-07-24, PASSED — auth+CSI Secret Manager stack proven, #2457/#2456)

Full two-service digest-mode run (GHCR `sha-54742a8d6e40` images — zero
Cloud Build) adding the first live validation of the auth+CSI regression
leg: a `lumen-authcsi` CR with `auth: required`,
`tokensSecretProviderClass`, and `tokensSecretCsiDriver:
secrets-store-gke.csi.k8s.io` against a run-scoped Secret Manager secret
(SecretProviderClass `provider: gke`, `principal://` secretAccessor grant,
no GSA). Proven: CSI volume mounted with the GKE driver name, pod Ready
with **zero FailedMount events** (the exact #2456 failure signature),
tokens genuinely loaded from the CSI mount — authenticated search returns
the seeded document (`total: 1`) while unauthenticated returns exactly 401
`{"error":"unauthenticated"}`. All prior legs re-passed on the 0.4.26
candidate HEAD (cold-restore, admission, backup, auto-split 1→2), and
verified cleanup covers the new Secret Manager resources. Cluster
prerequisite recorded: the GKE Secret Manager add-on
(`--enable-secret-manager`) registers the CSIDriver; the leg self-skips
with evidence when absent. Evidence root:
`axiom-gcp-run-backup/evidence/0724105144/` (`kubernetes/lumen-authcsi-*`).

### GKE acceptance run 0724061548 (2026-07-24, PASSED — #2489 fix + cold-restore #2492 proven)

- Full two-service mode (Lumen and Sift both passed; the Sift rows live in
  the shared `acceptance.json`). Cluster: persistent Standard GKE
  `axiom-operator-acceptance` (`asia-east1-a`, project `axiom-502607`),
  run-scoped bucket/GSA/Workload-Identity bindings plus the restore-reader
  grant created and destroyed by the run.
- Image: Cloud Build from commit `70fd48ca5c44` (the `lumen@0.4.25`
  candidate — carries the #2489 scatter fix `9ffdb30513`, #2497
  `spec.serviceAccountName`, and the #2487 alert fix), tag
  `70fd48ca5c44-0724061548`, dirty-tree gate clean.
- Terminal artifacts: `lumen-acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-24T06:56:11Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0724061548/`.

| Proof | Result | Artifact |
|---|---|---|
| Post-split read visibility (#2489): after the CONVERGED 1→2 auto-split, the pre-split collection is searchable through the client Service immediately — readability lag 0 s (vs `collection not found` for 180 s+ on both 0.4.24 retest runs). Restores the Dynamic Shard Topology GKE claim. | passed | `kubernetes/lumen-search-after-split.json`; `kubernetes/lumen-split-readable-after-seconds.txt` (`0`) |
| Cold-restore onto a fresh PVC (#2492): a second `lumen-restore` CR with `spec.serving.bootstrap.seedUri` pointed at the run's backup object (271 B, carries the `acceptance` collection) boots a genuinely fresh PVC and the seeded document is queryable (`total: 1`) | passed | `kubernetes/lumen-restore-search.json`; `gcs/lumen-first-object.json` |
| Seed-set restart retention: the restored instance keeps the seeded document across a serving-pod replacement | passed | `kubernetes/lumen-restore-after-restart-search.json` |
| Admission CR exposure (#2477): patching `spec.admission` renders the five `LUMEN_ADMISSION_*` env vars onto the StatefulSet pod spec (operator-propagation-aware poll), and removing the block rolls them back off | passed | `kubernetes/lumen-admission-env.txt` |
| Re-proven from `0723041614`: 1x1 reconcile, domain lifecycle (create/index/search), pod-restart data retention, Workload-Identity GCS backup (271-byte object) | passed | `kubernetes/…` per the matching rows in the `0723041614` table below |
| Verified cleanup: 6 run-scoped resources destroyed; "no run-tagged Lumen/Sift operator acceptance resources remain"; persistent cluster and Artifact Registry preserved | passed | `cleanup.json`; `run.log` |

Exclusions unchanged from `0723041614` (`cpu_memory_actuator`,
`live_replica_membership`: `not_claimed`). Deployer note for cold-restore:
the SERVING ServiceAccount of a `seedUri` instance reads GCS itself — it
needs `roles/storage.objectViewer` on the seed bucket (the backup GSA's
write grant does not cover it). The harness provisions this via
`storage.tf`'s `lumen_restore_reader` principal binding; real deployments
carry the same responsibility.

### GKE retest runs 0723160506 / 0723163748 (2026-07-23, FAILED — post-split read visibility, #2489)

Retest with the released `lumen@0.4.24` GHCR image
(`ghcr.io/chrischeng-c4/lumen@sha256:f460c6cf…493e90`, pulled anonymously —
the GHCR distribution path itself works). Passed on both runs: 1x1
reconcile, operator cell, index/search, Workload-Identity GCS backup,
pod-restart retention, and the 1→2 split convergence with a fully converged
post-cutover fence. FAILED both runs at post-split read visibility:
searching the pre-split collection through the client Service returns
`collection not found` and stays unreadable through a bounded 180-second
poll while `phase: Complete` and `convergedShardMapVersion ==
shardMap.version` — tracked as #2489. The prior run `0723041614`'s
post-split pass asserted a single probe and cannot stand as disproof;
treat the Dynamic Shard Topology GKE claim as NOT proven until #2489
closes. Default 1-shard deployments (no reshardPolicy) are unaffected.
Evidence: `axiom-gcp-run-backup/evidence/<run>/`. Resolution: the #2489
scatter fix (`9ffdb30513`) is proven by run `0724061548` above — the claim
is restored there.

### GKE acceptance run 0723041614 (2026-07-23, PASSED)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped
  bucket/GSA/Workload-Identity binding created and destroyed by the run.
- Image: pinned immutable
  `courier/lumen@sha256:da154652ff3fdf16fb406674240f0a3f4567047d5eb6e0e547bee0f389c68b1b`
  built from commit `f4762759d810` (`git_dirty: false`, `image_provenance:
  prebuilt`, tag `f4762759d810-0723041614`).
- Terminal artifacts: `acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-23T04:25:33Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0723041614/` (home-dir
  mirror of the volatile `/tmp` tree); `run.log` carries the full
  transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease creation, steady-state drift repair, leader-takeover reconcile (holder `...rrc6f` → `...5mlwx`) | passed | `lumen-operator-cell.json`; `kubernetes/lumen-lease-holder-*.txt` |
| 1x1 reconcile: one `Lumen` CR drives exactly one StatefulSet/shard to `Ready` on Standard GKE | passed | `kubernetes/lumen-crs.json`; `kubernetes/workloads-after-lumen-deploy.json` |
| Domain lifecycle through the client Service: create collection, index one document, search hit | passed | `kubernetes/lumen-create-collection.json`; `kubernetes/lumen-index.json`; `kubernetes/lumen-search-before-restart.json` |
| Pod-restart data retention: the indexed document survives a serving-pod replacement and is still searchable via the PVC-backed segment/WAL | passed | `kubernetes/lumen-search-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered backup writes a non-empty 271-byte snapshot object; readback is non-empty | passed | `kubernetes/lumen-backup.log`; `gcs/lumen-first-object.json` (`gs://axiom-502607-axo-0723041614-backup/lumen/0723041614-1784780377.json`) |
| Acceptance-only disk-pressure auto-split: `reshardPolicy.maxShardBytes: 1` (a test-only trigger, not a production threshold) drives shard count 1 → 2, 2 ready pods, at least 2 PVCs, and the document stays searchable post-split | passed | `kubernetes/lumen-after-split.json`; `kubernetes/lumen-search-after-split.json` |
| Shard-map fence convergence: post-split CR status settles at `reshard.phase: Complete`, `targetShardCount` == `shardCount` == 2, `usageMeasuredAtMapVersion: 1`, `convergenceRemediationRestartCount: 0` (no remediation restarts needed) | passed | `kubernetes/lumen-after-split.json` |
| Verified cleanup: run-scoped GCS bucket, backup GSA, IAM bindings destroyed (`Destroy complete! Resources: 4 destroyed`, "no run-tagged Lumen/Sift operator acceptance resources remain"); persistent cluster, Artifact Registry, and pre-existing APIs preserved | passed | `cleanup.json`; `run.log` |

Exclusions (recorded, not claimed): CPU/memory pressure actuation
(`cpu_memory_actuator: not_claimed`) and live in-place replica-membership
change (`live_replica_membership: not_claimed`) — neither is exercised by
this harness. `reshardPolicy.maxShardBytes: 1` is an acceptance-only
trigger value chosen to force a split deterministically inside a short
run; it is not evidence of any production disk-pressure threshold. Sift
was deferred from this run (`sift_collection_deferred`); Tape's own run
(`0723135853`) is recorded separately in `apps/tape/CAPABILITIES.md`.
