# Service K8s Capabilities

## Brief

Every Axiom data service that runs on Kubernetes needs the same operator: watch
a CR, render children, server-side-apply them, observe readiness, project
status, hold a lease. Writing that loop once per service produces six subtly
different answers to the same questions — and the questions that differ are the
ones that matter, because they are where an operator can be wrong without
looking wrong.

`service-k8s` is that loop written once, plus the four decisions underneath it
that are worth stating rather than discovering.

A **certificate profile is an authorization artifact**: whoever can name the
identity on it can obtain that identity. So a profile is validated against the
instance scope that will own it at construction time, and there is no
public field-literal path. A profile naming another namespace's DNS name or
another tenant's SPIFFE URI is not rejected later — it cannot be built.

**Rotation is memoryless.** `next_action` is a pure function of what the service
wants, what the cluster currently shows, and what time it is. A controller that
remembers "I issued one a minute ago" mints a duplicate on every restart, and
the failure is invisible: two valid certificates look exactly like one.
Reconstructing the decision from the leaf's own metadata makes restart a
non-event by construction, and makes an interrupted rotation resume rather than
restart or skip.

**A data workload scales in whole replica layers.** With `N` shards, changing
replicas-per-shard by one changes the StatefulSet by exactly `N` pods; a vanilla
HPA targets total pods and can therefore request an invalid partial layer. The
planner does the HPA utilization arithmetic in per-shard units and always
returns a valid whole-layer total — and it plans, never applies, because a
replica-layer change is also a Raft membership change.

**Status projection is clock-free.** A service computes condition facts; the
controller injects the instant. That split is what keeps `lastTransitionTime`
meaning "when this last *changed*" across a 30-second requeue, and what lets
every adopting service keep deterministic status tests.

Everything else in the crate — the render helpers, the CAS issuer, the Secret
store, the metrics endpoint, the lease — is machinery in service of those four.

## Capabilities

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

Capabilities split into two roots. **Core Features** are the decisions an
adopting operator inherits and cannot override: identity scoping, the rotation
state machine, whole-layer capacity planning, and condition projection.
**Non-Core Features** are the surfaces a service opts into — where material is
laid out and what the lifecycle is allowed to say about itself, and the
compatibility shims between Rust-derived schemas and what a live API server and
a bound PVC will actually accept.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Scope-Validated Certificate Identity | 3381 | implemented | verified | smoke | ready | core; a profile naming another namespace, a public name, or a foreign SPIFFE URI is unconstructible, and the identity digest covers certified content only |
| Memoryless Rotation Decision | 3381 | implemented | verified | smoke | ready | core; one pure function of desired/observed/now, deterministic per-leaf jitter, trust widened before issuance and narrowed only after observed activation |
| Whole-Layer Capacity Planning | 3381 | implemented | verified | smoke | ready | core; per-shard HPA arithmetic that cannot emit a partial layer, a strict one-GiB shard-split threshold, and plan-not-apply for both axes |
| Clock-Free Condition Projection | 3381 | implemented | verified | smoke | ready | core; services emit facts, the controller injects the instant, and an unchanged status keeps its original transition time across requeues |
| Owner-Scoped Material Projection | 3381 | implemented | verified | smoke | ready | non-core; one Secret per purpose per instance, garbage-collected with its owner, whose trust-widening write cannot blank the serving leaf, and whose status path redacts by construction |
| Cluster Compatibility Surfaces | 3381 | implemented | verified | smoke | ready | non-core; unsigned-format and YAML-1.1 normalization that keep a derived CRD installable, and a grow-only PVC resize for the field a StatefulSet template cannot change |

### Core Features

#### Scope-Validated Certificate Identity

ID: scope-validated-certificate-identity
Root WI: 3381
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
A certificate profile states what a service is asking a certificate *for*:
which direction of TLS, which names, and how long it may live. It is validated
against the `InstanceScope` that will own it, and `CertificateProfile::new` is
the only way to build one — there is no public field-literal path, because an
unvalidated profile is exactly the value this module exists to prevent.

The validation is a tenancy boundary, not a lint. A DNS name must carry the
`.<namespace>.svc` segment Kubernetes itself uses to separate tenants, so a
prefix match cannot wave `lumen.lumen-prod.svc` through for namespace `lumen`.
A name outside the cluster-internal suffixes is refused as public. A SPIFFE URI
must begin with the scope's own `spiffe://<trust-domain>/ns/<namespace>/`
prefix. Each refusal names the offending value, so an operator reading it in a
condition does not have to diff two lists.

Purpose selects extended key usages exactly, not at least. A serving leaf
carries `serverAuth` alone: one that also carried `clientAuth` would be a
credential its holder could replay to authenticate *as* the service to its own
peers. A peer leaf carries both, and must carry a SPIFFE URI, because DNS alone
cannot distinguish two members of the same headless Service from each other's
point of view.

Lifetime bounds are stated locally rather than learned from a rejected CSR, and
the renewal window has a floor wide enough for a dozen retries. Where the
Secret lives is derived from the scope, never supplied: a caller-chosen Secret
name is a caller-chosen place to write.

The identity digest covers purpose, common name, sorted DNS names, the SPIFFE
URI, and the usages — the certified content only. Lifetime is deliberately
outside it, because changing the renewal cadence is not a reason to throw away
a valid identity.

Surfaces:
- Rust API: `service_k8s::certificate::CertificateProfile::new` - the fallible constructor and the whole validation ladder.
- Rust API: `service_k8s::certificate::InstanceScope` - namespace, instance, and trust domain, with the derived `secret_name` and `spiffe_prefix`.
- Rust API: `service_k8s::certificate::Purpose::extended_key_usages` - the exact usage set per direction.
- Rust API: `service_k8s::certificate::CertificateProfile::identity_digest` - the reissue-relevant content digest.
- Rust API: `service_k8s::certificate::profile::ProfileError` - one variant per refusal, each naming its offending value.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - each purpose's usage set is exact, the digest is order-insensitive across DNS names but content-sensitive, and Secret names derive from the scope.
- security: `cargo test -p service-k8s` - a foreign namespace, a namespace *prefix*, a public suffix, and a foreign SPIFFE URI are each refused at construction, and a peer profile without a SPIFFE URI cannot be built.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| A serving leaf cannot authenticate outward | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::profile::tests::serving_leaves_do_not_carry_client_auth` paired with `peer_leaves_carry_both_directions` pins both sets, so an over-broad usage list fails one of the two |
| A namespace prefix is not a namespace match | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::profile::tests::a_namespace_prefix_is_not_a_namespace_match` uses `lumen.lumen-prod.svc.cluster.local` against namespace `lumen`, which a suffix or prefix check would accept |
| A foreign SPIFFE identity is refused | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::profile::tests::another_namespaces_spiffe_identity_is_refused` with `another_namespaces_dns_name_is_refused` and `a_public_dns_name_is_refused` covers all three name channels |
| The identity digest is content-sensitive, not order-sensitive | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::profile::tests::identity_digest_ignores_dns_name_order_but_not_content` asserts both directions in one test, so a digest that ignores names entirely fails |
| A renewal window with no room to retry is refused | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::profile::tests::a_renewal_window_with_no_room_to_retry_is_refused` |

#### Memoryless Rotation Decision

ID: memoryless-rotation-decision
Root WI: 3381
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
`next_action` decides the single next step from what the service wants, what
the cluster currently shows, and what time it is. It keeps no memory, and that
is the whole design: a controller that remembers having issued a certificate
mints a duplicate every time it restarts, and two valid certificates look
exactly like one.

The ladder is ordered by what fails worst. Trust comes before identity: a leaf
from an issuer nobody trusts fails at handshake time, on the far side, where
the error says "unknown CA" and names nothing useful. Then bootstrap, then
issuer change, then identity change, then expiry, then the rotation tail, then
renewal. Each issuance carries a reason, so an operator can tell a routine
renewal from a scramble — `Expired` is deliberately distinct from `Renewal`
because it means something was already wrong.

Trust is widened by a step that is always a superset of what is published now.
Narrowing is a separate action with a precondition: anchors are retired only
once the runtime reports it is *presenting* the new leaf. A written file, a
successful API call, or a sleep would retire an issuer that is still
authenticating live connections.

Renewal jitter is derived from the leaf's own fingerprint, not from a random
number generator. That is the requirement, not a shortcut: a random offset
chosen at reconcile time changes on every restart, so a frequently-restarting
controller would drift its own deadline and could decide a leaf is not yet due
after a previous process had already decided it was. Deriving it from the leaf
makes every process compute the same instant for the same certificate, while
different certificates still spread out.

Note what the action type cannot express: no variant removes the current leaf.
"A failed step retains the last valid serving material" is not a rule this
module follows — it is a sentence it cannot say. Retry backoff climbs to a
five-minute ceiling and stays there, because a controller racing an expiry must
not back off past the window it is racing.

Surfaces:
- Rust API: `service_k8s::certificate::state::next_action` - the pure decision.
- Rust API: `service_k8s::certificate::state::Action` - the closed set of next steps, none of which removes material.
- Rust API: `service_k8s::certificate::state::renew_at` - the deterministic per-leaf renewal instant.
- Rust API: `service_k8s::certificate::state::retry_after` - bounded backoff.
- Rust API: `service_k8s::certificate::state::Observed` - the observation set, including the runtime's actually-presented fingerprint.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - the ladder selects each rung, the renewal instant is stable across calls yet differs between leaves, and backoff saturates.
- security: `cargo test -p service-k8s` - trust is published before any issuance, no reachable action removes existing material, and retirement is gated on observed activation rather than on a write having succeeded.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| Trust is published before anything is issued | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::state::tests::trust_is_published_before_anything_is_issued` starts from a wholly empty observation, where an identity-first ladder would issue instead |
| The renewal instant survives a restart | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::state::tests::the_renewal_instant_does_not_move_when_the_controller_restarts` paired with `different_certificates_still_spread_out`, so a constant-zero jitter fails the second |
| An expired leaf is distinguishable from a due one | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::state::tests::an_expired_leaf_is_distinguishable_from_a_due_one` with `an_identity_change_reissues_a_perfectly_valid_leaf` covering the non-time-driven rung |
| No reachable action removes the current leaf | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::state::tests::no_action_can_remove_the_current_leaf` enumerates every variant over three observation states rather than asserting one case |
| Backoff climbs to a ceiling and stays there | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::state::tests::backoff_climbs_to_a_ceiling_and_stays_there` asserts `u32::MAX` as well as the curve |

#### Whole-Layer Capacity Planning

ID: whole-layer-capacity-planning
Root WI: 3381
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
A sharded data workload scales in whole replica layers. With `N` shards,
changing replicas-per-shard by one changes the StatefulSet by exactly `N` pods,
so a vanilla HPA — which targets total pods — can request a topology that does
not exist. `plan_replica_layer` performs the same utilization arithmetic HPA
does, `ceil(current × observed / target)`, but in per-shard units, evaluates
CPU and memory independently and takes the larger, clamps to the per-shard
bounds, and returns a total that is always a multiple of the shard count.

A missing metric holds steady rather than forcing a decision: scale-down
recomputes from the observed signals only when at least one signal exists, so a
gap in the metrics pipeline cannot be read as idleness.

Storage pressure is a separate axis with its own policy. `plan_shard_split`
plans at most one new physical shard from the busiest shard's durable bytes.
The threshold is strict — exactly one GiB holds steady and one GiB plus one
byte plans a split — and ties choose the lowest shard index so the plan is
deterministic across observations.

Both functions plan and neither applies, and that is the load-bearing part. A
replica-layer change is also a Raft membership change: the caller must complete
the membership transition before patching the replica count, and this library
never treats adding StatefulSet pods as a completed shard split. The service
still owns its own routing-map cutover, fencing, and data movement.

Resource requests resolve to shared data-plane defaults (`1` CPU, `4Gi`) when a
service leaves them empty, without inventing a node-pool-specific size. Limits
are deliberately omitted so a dedicated-node pod can use otherwise-idle node
capacity.

Surfaces:
- Rust API: `service_k8s::plan_replica_layer` - the per-shard HPA calculation and its clamped whole-layer result.
- Rust API: `service_k8s::plan_shard_split` - the one-shard-at-a-time storage decision.
- Rust API: `service_k8s::ReplicaLayerPlan::requires_membership_change` - the Raft precondition a caller must honor.
- Rust API: `service_k8s::ShardSplitPlan::requires_split` / `busiest_shard` / `max_shards_reached` - the decision and the evidence behind it.
- Rust API: `service_k8s::stateful::resource_request_or_default` with `DEFAULT_CPU_REQUEST` / `DEFAULT_MEMORY_REQUEST`.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - CPU and memory each drive the layer, the split threshold is strict at exactly one GiB, ties resolve deterministically, and the ceiling holds.
- security: `cargo test -p service-k8s` - a zero shard count, inverted bounds, an out-of-range utilization target, a zero threshold, and an observation naming a shard outside the topology are each refused rather than producing a plan.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| A plan is always a whole replica layer | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `stateful::tests::cpu_scale_out_is_a_whole_shard_layer` asserts both the per-shard value and the total, and `memory_can_drive_the_larger_layer` proves the two axes are taken independently |
| A missing metric holds rather than scales | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `stateful::tests::missing_metrics_hold_and_bounds_clamp` pairs the absent-signal case with a floor-clamped one, so a planner that always held would fail the second half |
| The split threshold is strictly greater | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `stateful::tests::disk_split_threshold_is_strictly_greater_than_one_gib` asserts at-threshold and threshold-plus-one in one test |
| A split adds one shard and honors the ceiling | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `stateful::tests::disk_split_adds_one_shard_and_honors_the_ceiling` also asserts the tie-break shard index rather than only the count |
| Invalid topologies and observations are refused | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `stateful::tests::invalid_partial_topologies_are_rejected` and `disk_split_rejects_invalid_policy_and_observations` cover each error variant by identity |

#### Clock-Free Condition Projection

ID: clock-free-condition-projection
Root WI: 3381
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
An adopting service implements one trait and inherits the whole operator loop:
render children, server-side-apply them under a per-service field manager,
observe `.status.readyReplicas`, project status, hold a lease named after the
same manager so two operators never collide.

Status conditions use the shape every controller-aware tool already reads —
`kubectl wait --for=condition=…`, Argo CD health assessment, Flux readiness
gates. The type is hand-written rather than reused from `k8s-openapi` because
that one does not derive `JsonSchema` and therefore cannot be embedded in a CRD
schema generated by `kube`'s derive.

The projection is split so it stays deterministic. A service emits
`ConditionFact`s — type, status, reason, message — and the controller injects
the instant. `lastTransitionTime` means "when this condition last *changed*",
so a reconcile that re-observes the same state must not move it; otherwise the
periodic 30-second requeue would look like a state change to everything
watching. Everything else on the condition does refresh: only the transition
instant is sticky.

Because the controller writes status with a merge patch, which replaces arrays
wholesale, transition times cannot survive server-side. They are read back off
the watched object and carried forward explicitly, and a condition that drops
out and later returns is a *new* condition that takes the injected time.

Two hooks stay optional and default to the previous behavior exactly, so a
service that has not adopted conditions, planning context, or pruning keeps its
existing status shape byte-for-byte. Pruning exists because server-side apply
reconciles *fields*, never object lifetime: a child that drops out of `render`
keeps running. For a NetworkPolicy that makes the toggle a one-way door, and
naming the object closes it — with an ownership re-check before deletion, so a
mis-named target is inert rather than destructive.

Surfaces:
- Rust API: `service_k8s::ManagedService` - the trait, its per-service `MANAGER`, and the defaulted `reconcile_plan` / `conditions` / `observed_conditions` / `prunes` hooks.
- Rust API: `service_k8s::service::project` - fact plus prior plus injected instant to full conditions.
- Rust API: `service_k8s::Condition` / `ConditionStatus` / `ConditionFact` - the metav1-shaped, JsonSchema-deriving projection types.
- Rust API: `service_k8s::service::now_rfc3339` - second-precision UTC, the one place a clock is read.
- Rust API: `service_k8s::PruneTarget` - a child the service no longer renders and wants removed.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - an unchanged status keeps its instant while reason, message, and observed generation refresh; a flip, a first sighting, and a resurrection each take the injected instant; and the serialized shape is exactly metav1's.
- security: `cargo test -p service-k8s` - projection is deterministic and order-preserving for the same inputs, and the emitted timestamp carries no sub-second component the API server would rewrite.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| An unchanged status keeps its transition time | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `service::condition_tests::unchanged_status_keeps_its_original_transition_time` also asserts that reason, message, and observed generation *did* refresh, so a wholesale carry-forward fails |
| A flip takes the injected instant | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `service::condition_tests::flipped_status_takes_the_injected_time` with `a_condition_seen_for_the_first_time_takes_the_injected_time` |
| A resurrected condition is a new condition | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `service::condition_tests::dropped_conditions_do_not_resurrect_their_old_transition_time` drops and re-adds in one test |
| The serialized shape is metav1's | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `service::condition_tests::serialized_shape_is_metav1_condition` asserts the whole JSON object, so a renamed or added field fails |
| Projection is deterministic and clock-free | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `service::condition_tests::projection_is_deterministic_and_order_preserving` and `now_rfc3339_has_no_subsecond_component` |

### Non-Core Features

#### Owner-Scoped Material Projection

ID: owner-scoped-material-projection
Root WI: 3381
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
One Secret per purpose per instance, carrying the three keys consumers already
read — `tls.crt`, `tls.key`, `ca.crt`. Keeping that layout is deliberate: the
consumer contract for peer material is already deployed, and changing it would
require changing the pod spec, which is the one thing renewal must never
require.

`ca.crt` is the trust *bundle*, not a single anchor. During a rotation it holds
the outgoing and incoming issuers at once, which is what makes the overlap in
the state machine mean anything. The Secret type is `Opaque` rather than
`kubernetes.io/tls` because the TLS type requires both leaf and key to be
present at all times, which would make the bootstrap step — publishing trust
*before* any leaf exists — unrepresentable.

The trust-only write is a merge patch carrying `ca.crt` alone. Widening trust
must never be able to blank the leaf that is currently serving traffic, and
that is enforced by what the write contains rather than by ordering. Every
Secret carries an owner reference with `controller` and `blockOwnerDeletion`
set, so material is garbage-collected with its instance: an orphaned Secret
full of key material is exactly the kind of residue nobody notices until an
audit.

Reading state back splits by evidence. Expiry and fingerprint are parsed from
the certificate itself; issuer ids and the identity digest come from
annotations, because they are not derivable from the DER. An annotation is a
claim and a certificate is evidence, so anywhere both could answer, the
certificate answers — a hand-edited annotation cannot talk the controller into
believing a leaf expires later than it does. When the bundle's PEM block count
and its issuer-id annotation disagree, the result is an empty bundle rather
than a guess, which the state machine reads as "trust is not published" and
republishes from the issuers themselves.

What the lifecycle may say about itself has exactly one production path, and
that path redacts. The facts type carries no secret-bearing field in the first
place — no PEM, no key, no token, no request body, and no caller-supplied
message — and every string it emits passes a scrubber for PEM blocks, bearer
tokens, and JWT-shaped strings on the way out. The published fingerprint is a
sixteen-character prefix: enough to correlate two observations, not enough to
be mistaken for an artifact. Readiness and rotation are separate conditions,
because a routine renewal must not make an instance unready.

Surfaces:
- Rust API: `service_k8s::certificate::projection::material_secret` / `trust_bundle_secret` - the two writes, and what each one may contain.
- Rust API: `service_k8s::certificate::projection::TrustBundle` - insert, retain, and the annotation-checked `parse`.
- Rust API: `service_k8s::certificate::projection::read_state` / `parse_leaf` - certificate-over-annotation readback.
- Rust API: `service_k8s::CertificateFacts::conditions` - the two purpose-namespaced conditions.
- Rust API: `service_k8s::certificate::status::redact` - the PEM, bearer, and JWT backstop.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - a bundle round-trips through its PEM and annotation, conditions report ready with issuer and expiry, and serving and peer conditions do not collide.
- security: `cargo test -p service-k8s` - the trust-only write contains no leaf keys, Secrets carry a controlling owner reference, a bundle whose annotation disagrees with its contents is emptied rather than guessed at, and PEM blocks, bearer tokens, and projected tokens never survive into status.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| Widening trust cannot blank the serving leaf | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::projection::tests::publishing_trust_writes_no_leaf_keys` asserts the exact key count as well as the absences |
| A disagreeing bundle is emptied, not guessed at | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::projection::tests::a_bundle_whose_annotation_disagrees_with_its_contents_is_not_guessed_at` paired with the round-trip test, so an always-empty parse fails the first |
| Material is garbage-collected with its instance | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::projection::tests::secrets_are_garbage_collected_with_their_instance` and `the_secret_lands_in_the_instances_own_namespace` |
| Three secret shapes never survive into status | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::status::tests::a_pem_block_never_survives_into_status`, `a_bearer_token_never_survives_into_status`, and `a_projected_token_never_survives_into_status`, with `ordinary_text_passes_through_unharmed` as the negative control |
| A rotation does not make an instance unready | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `certificate::status::tests::a_rotation_does_not_make_the_instance_unready` with `a_failing_lifecycle_says_so_rather_than_staying_pending` |

#### Cluster Compatibility Surfaces

ID: cluster-compatibility-surfaces
Root WI: 3381
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
A CRD derived from Rust types is not automatically a CRD a live API server will
accept, and the two gaps are both silent. Schemars emits `uint32` and `uint64`
formats for Rust unsigned types, which Kubernetes structural OpenAPI does not
recognize; the normalizer removes the format recursively and retains the
unsigned contract as `minimum: 0` without disturbing a tighter minimum already
present. And `serde_yaml` follows YAML 1.2, so it emits `off`, `on`, `yes`, and
`no` unquoted — while Kubernetes still accepts YAML 1.1 input, where those are
booleans, and rejects a string schema whose default arrives as `false`.
Quoting those spellings cannot change a real boolean, because JSON booleans
serialize as `true`/`false`.

Cross-field invariants are attached as CEL rules at admission rather than
enforced in the reconcile loop, because an operator's complaint lives in its
log, which is not where the person who ran `apply` is looking. The attach
function returns how many versions it reached, so a caller can assert non-zero
and notice when the CRD's shape changed under it, and it attaches nothing
rather than panicking on a CRD without a spec schema.

There is one rule about writing those expressions that the local tests cannot
catch, and it is recorded here because it has already produced a CRD that
passed every local gate and could not be installed on any cluster: test
presence with `has(self.x)` and nothing else, including for `nullable: true`
fields. Kubernetes prunes an explicitly-null field before CEL runs, so `has()`
already reports it absent, and a defensive `self.x != null` does not merely
duplicate that — it fails to compile, because Kubernetes types a
`nullable: true` string as plain `string`.

PVC growth is the other silent gap. StatefulSet `volumeClaimTemplates` are
immutable after creation, so bumping a CR's declared storage and letting the
operator reconcile is a no-op for that field. The resize path parses
Kubernetes storage quantities with the real `resource.Quantity` semantics —
binary suffixes are powers of 1024, decimal ones powers of 1000 — classifies
the comparison, and patches only `spec.resources.requests.storage`, only on
PVCs whose bound StorageClass allows expansion. Shrink is reported and never
attempted, because Kubernetes cannot shrink a bound PVC.

Surfaces:
- Rust API: `service_k8s::crd::normalize_unsigned_integer_formats` - recursive format removal preserving non-negative semantics.
- Rust API: `service_k8s::crd::add_spec_validation_rule` - CEL attach with an attached-version count.
- Rust API: `service_k8s::crd::quote_yaml_1_1_boolean_like_strings` - the YAML 1.1 coercion shim.
- Rust API: `service_k8s::resize::parse_storage_bytes` / `decide` - quantity parsing and the four-way classification.
- Rust API: `service_k8s::resize::resize_instance` - the expansion-gated, grow-only patch driver.

EC Dimensions:
- behavior: `cargo test -p service-k8s` - formats are removed at every depth including inside arrays, rules accumulate across versions, the four quantity families parse, and each resize classification is produced.
- security: `cargo test -p service-k8s` - a tighter existing minimum is preserved rather than overwritten, a CRD without a spec schema attaches nothing rather than panicking, a real boolean is left unquoted, and a negative or unparseable quantity is an error rather than a byte count.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|---|
| Unsigned formats are normalized without loosening a minimum | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `crd::tests::removes_unsigned_formats_recursively_and_keeps_nonnegative_semantics` asserts the `minimum: 1` field keeps its own value and the array element is reached |
| A second CEL rule does not replace the first | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `crd::tests::spec_validation_rules_attach_to_every_version_and_accumulate` asserts the array length and the first rule's content after two attaches |
| A CRD without a spec schema attaches nothing | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `crd::tests::a_crd_without_a_spec_schema_attaches_nothing_rather_than_panicking` covers both the missing-schema and empty-document forms |
| Only YAML 1.1 boolean spellings are quoted | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `crd::tests::quotes_yaml_1_1_boolean_like_strings_without_touching_real_booleans` asserts `default: false` survives unquoted and a trailing colon in prose is untouched |
| Shrink is classified, never attempted | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `resize::tests::decide_detects_shrink_unsupported` with `decide_detects_grow`, `decide_detects_no_op`, and `decide_detects_unparseable` covering the full classification |
| Quantity parsing matches `resource.Quantity` | change | 3381 | implemented | verified | smoke | `cargo test -p service-k8s`; `resize::tests::parses_binary_and_decimal_suffixes_and_bare_bytes` pins 1024- and 1000-based suffixes in one test, and `rejects_unparseable_quantities` covers empty, non-numeric, and negative |
