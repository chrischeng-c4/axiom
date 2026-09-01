// CODEGEN-BEGIN
//! The [`ManagedService`] trait a service implements + the shared CRD fragments.

#[cfg(feature = "controller")]
use std::collections::{BTreeMap, HashMap};
#[cfg(feature = "controller")]
use std::fmt::Debug;
#[cfg(feature = "controller")]
use std::future::Future;

#[cfg(feature = "controller")]
use kube::core::NamespaceResourceScope;
#[cfg(feature = "controller")]
use kube::{Client, CustomResourceExt, Resource};
use schemars::JsonSchema;
#[cfg(feature = "controller")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A workload to poll for `.status.readyReplicas` during reconcile.
#[cfg(feature = "controller")]
pub struct ReadinessTarget {
    pub kind: &'static str,
    pub name: String,
}

/// Observed readiness handed to [`ManagedService::status_patch`]
/// (workload name → `readyReplicas`).
#[cfg(feature = "controller")]
pub struct ReadyFacts {
    pub ready: HashMap<String, i64>,
}

#[cfg(feature = "controller")]
impl ReadyFacts {
    /// Ready replicas for `name`, or 0 if the workload was absent.
    pub fn get(&self, name: &str) -> i64 {
        self.ready.get(name).copied().unwrap_or(0)
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1849" reason="Add ReconcilePlan plus backwards-compatible default reconcile_plan and status_patch_with_context hooks to ManagedService.">
/// One service-specific planning result consumed by the shared controller.
/// `context` is opaque to service-k8s and is handed back to the same service
/// only after children have been applied and readiness has been observed.
#[cfg(feature = "controller")]
pub struct ReconcilePlan {
    pub children: Vec<serde_json::Value>,
    pub context: serde_json::Value,
}

/// One service's contribution to the shared operator. Implemented on the CRD
/// root type (e.g. lumen's `Lumen`). The [`crate::controller`] is generic over
/// `S`, so the watch/apply/lease loop is written once.
#[cfg(feature = "controller")]
pub trait ManagedService:
    Resource<DynamicType = (), Scope = NamespaceResourceScope>
    + CustomResourceExt
    + Clone
    + Debug
    + DeserializeOwned
    + Send
    + Sync
    + 'static
{
    /// Server-side-apply field manager **and** the leader-election Lease name.
    /// Per-service so two operators never collide on the same Lease.
    const MANAGER: &'static str;

    /// Pure render: the spec (+ metadata via `ResourceExt`) → the child objects
    /// to server-side-apply. No I/O.
    fn render(&self) -> Vec<serde_json::Value>;

    /// Optional async pre-apply planning hook. Existing services keep the pure
    /// render behavior; services with external admission can inspect Kubernetes
    /// or remote state and carry contextual facts into status projection.
    fn reconcile_plan(
        &self,
        _client: Client,
    ) -> impl Future<Output = anyhow::Result<ReconcilePlan>> + Send {
        let children = self.render();
        async move {
            Ok(ReconcilePlan {
                children,
                context: serde_json::Value::Null,
            })
        }
    }

    /// The workloads whose `.status.readyReplicas` feed [`Self::status_patch`].
    fn readiness_targets(&self) -> Vec<ReadinessTarget>;

    /// The `{ "status": { … } }` subresource patch given observed readiness.
    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value;

    /// Context-aware status projection paired with [`Self::reconcile_plan`].
    /// Defaults to the original readiness-only contract.
    fn status_patch_with_context(
        &self,
        ready: &ReadyFacts,
        _context: &serde_json::Value,
    ) -> serde_json::Value {
        self.status_patch(ready)
    }

    /// The `status.conditions[]` this service reports for the observed state,
    /// in the order they should appear (#2601).
    ///
    /// Clock-free by construction: [`project`] stamps `lastTransitionTime` with
    /// a time the controller injects, so this stays a pure function of spec +
    /// observed facts and its tests stay deterministic.
    ///
    /// Defaults to none, so a service that has not adopted conditions keeps its
    /// existing status shape byte-for-byte.
    fn conditions(&self, _ready: &ReadyFacts, _context: &serde_json::Value) -> Vec<ConditionFact> {
        Vec::new()
    }

    /// The conditions already persisted on this object's status (#2601).
    ///
    /// The controller writes status with `Patch::Merge`, which replaces arrays
    /// wholesale, so transition times cannot survive server-side — they have to
    /// be read back off the watched object (which carries `.status`) and carried
    /// forward explicitly by [`project`].
    fn observed_conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    /// Children this service rendered under a previous spec but no longer
    /// wants to exist (#2603).
    ///
    /// Server-side apply reconciles *fields*, never object lifetime: a child
    /// that drops out of [`Self::render`] simply stops being updated and keeps
    /// running until the owning CR is deleted. For most children that is
    /// harmless. For one whose entire purpose is to enforce something — a
    /// NetworkPolicy — it makes the toggle a one-way door: turning it on takes
    /// effect, turning it off does not, and the operator silently keeps
    /// enforcing a posture the spec no longer asks for. Naming the object here
    /// closes that door.
    ///
    /// Only ever name objects this CR owns. The controller re-checks ownership
    /// against the live object before deleting, so a target that turns out to
    /// belong to something else is inert rather than destructive — but the
    /// check is a safety net, not a license to guess.
    ///
    /// Defaults to none, so a service that has not adopted pruning keeps its
    /// existing behavior exactly.
    fn prunes(&self) -> Vec<PruneTarget> {
        Vec::new()
    }

    /// Cluster-scoped children this namespaced CR may create.
    ///
    /// Kubernetes forbids a cluster-scoped object from carrying an owner
    /// reference to a namespaced CR. Services that render such an object must
    /// therefore declare it here. The shared controller installs a finalizer
    /// before the object is applied, removes an undesired object during normal
    /// reconcile, and removes every declared object before CR deletion.
    ///
    /// `expected_labels` and the server-side-apply manager are both checked
    /// before deletion. A name alone is never accepted as ownership proof.
    fn cluster_scoped_children(&self) -> Vec<ClusterScopedChild> {
        Vec::new()
    }
}

/// One object a service no longer renders and wants removed (#2603).
///
/// Namespace is not a field: the controller prunes in the CR's own namespace,
/// which is the only place [`ManagedService::render`] can place children.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg(feature = "controller")]
pub struct PruneTarget {
    pub api_version: &'static str,
    pub kind: &'static str,
    pub name: String,
}

/// One cluster-scoped child whose lifetime follows a namespaced CR.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg(feature = "controller")]
pub struct ClusterScopedChild {
    pub api_version: &'static str,
    pub kind: &'static str,
    pub name: String,
    pub expected_labels: BTreeMap<String, String>,
    /// `true` means the current spec renders the child. `false` means a prior
    /// version may have rendered it and the controller must remove it.
    pub desired: bool,
}

#[cfg(all(test, feature = "controller"))]
mod tests {
    use super::*;
    use http::{Request, Response};
    use kube::client::Body;
    use kube::CustomResource;
    use serde_json::json;
    use tower::service_fn;

    #[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
    #[kube(
        group = "service-k8s.test",
        version = "v1",
        kind = "PureRenderService",
        namespaced
    )]
    struct PureRenderServiceSpec {
        replicas: u32,
    }

    impl ManagedService for PureRenderService {
        const MANAGER: &'static str = "pure-render-test";

        fn render(&self) -> Vec<serde_json::Value> {
            vec![json!({
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": { "name": "pure-render" },
                "spec": { "replicas": self.spec.replicas },
            })]
        }

        fn readiness_targets(&self) -> Vec<ReadinessTarget> {
            vec![ReadinessTarget {
                kind: "Deployment",
                name: "pure-render".into(),
            }]
        }

        fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
            json!({ "status": { "readyReplicas": ready.get("pure-render") } })
        }
    }

    fn inert_client() -> Client {
        let service = service_fn(|_request: Request<Body>| async move {
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        });
        Client::new(service, "default")
    }

    #[tokio::test]
    async fn default_plan_and_status_preserve_existing_contract() {
        let service = PureRenderService::new("pure-render", PureRenderServiceSpec { replicas: 2 });
        let expected = service.render();
        let plan = service
            .reconcile_plan(inert_client())
            .await
            .expect("pure render plan");
        assert_eq!(plan.children, expected);
        assert!(plan.context.is_null());

        let ready = ReadyFacts {
            ready: HashMap::from([("pure-render".into(), 2)]),
        };
        assert_eq!(
            service.status_patch_with_context(&ready, &json!({ "ignored": true })),
            service.status_patch(&ready)
        );
    }
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:logic" tracker="#2601" reason="The metav1.Condition projection is a hand-written seam: a JsonSchema-deriving Condition the generator cannot synthesize from k8s-openapi, plus the clock-free fact/projection split that keeps status projection deterministic.">
/// One entry of a Kubernetes `status.conditions[]` array, in the shape every
/// controller-aware tool already reads — `kubectl wait --for=condition=…`,
/// Argo CD health assessment, Flux readiness gates.
///
/// Hand-written rather than reused from
/// `k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition` because that type
/// does not derive `JsonSchema`, so it cannot be embedded in a CRD schema
/// generated by `kube`'s derive.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// CamelCase condition name, e.g. `Ready`.
    #[serde(rename = "type")]
    pub type_: String,
    /// `"True" | "False" | "Unknown"`.
    pub status: String,
    /// CamelCase machine-readable cause of the current status.
    pub reason: String,
    /// Human-readable detail. May be empty.
    #[serde(default)]
    pub message: String,
    /// RFC3339 instant the status last *changed* — not the last time it was
    /// observed. Carried across reconciles by [`project`].
    pub last_transition_time: String,
    /// The `.metadata.generation` this condition was computed from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_generation: Option<i64>,
}

/// The three values Kubernetes allows in a condition's `status`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

impl ConditionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::True => "True",
            Self::False => "False",
            Self::Unknown => "Unknown",
        }
    }

    /// `True`/`False` from a plain predicate — the common case.
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

/// A condition as a service computes it: everything except the clock.
///
/// Services project status synchronously and without I/O, so they cannot stamp
/// `lastTransitionTime` themselves. [`project`] does it with a time the
/// controller injects — that split is what keeps status projection
/// deterministically testable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionFact {
    pub type_: String,
    pub status: ConditionStatus,
    pub reason: String,
    pub message: String,
}

impl ConditionFact {
    pub fn new(
        type_: impl Into<String>,
        status: ConditionStatus,
        reason: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            type_: type_.into(),
            status,
            reason: reason.into(),
            message: message.into(),
        }
    }
}

/// Stamp `facts` into full [`Condition`]s, carrying `lastTransitionTime` forward
/// from `prior` for every condition whose `status` did not change.
///
/// `lastTransitionTime` means "when this condition last *changed*", so a
/// reconcile that re-observes the same state must not move it — otherwise the
/// periodic 30s requeue would look like a state change to everything watching.
///
/// Conditions absent from `facts` are dropped: a service declares its whole
/// condition set every pass, and the controller writes status with
/// `Patch::Merge`, which replaces the array wholesale anyway.
pub fn project(
    prior: &[Condition],
    facts: Vec<ConditionFact>,
    observed_generation: i64,
    now: &str,
) -> Vec<Condition> {
    facts
        .into_iter()
        .map(|fact| {
            let status = fact.status.as_str().to_string();
            let last_transition_time = prior
                .iter()
                .find(|c| c.type_ == fact.type_ && c.status == status)
                .map(|c| c.last_transition_time.clone())
                .unwrap_or_else(|| now.to_string());
            Condition {
                type_: fact.type_,
                status,
                reason: fact.reason,
                message: fact.message,
                last_transition_time,
                observed_generation: Some(observed_generation),
            }
        })
        .collect()
}

/// Now, in the RFC3339 form Kubernetes expects in `lastTransitionTime`.
/// Second precision: metav1 timestamps carry no sub-second component, and
/// emitting one makes the API server rewrite the value on every write.
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod condition_tests {
    use super::*;

    fn prior(type_: &str, status: &str, at: &str) -> Condition {
        Condition {
            type_: type_.into(),
            status: status.into(),
            reason: "Whatever".into(),
            message: String::new(),
            last_transition_time: at.into(),
            observed_generation: Some(1),
        }
    }

    #[test]
    fn unchanged_status_keeps_its_original_transition_time() {
        let before = vec![prior("Ready", "True", "2026-01-01T00:00:00Z")];
        let out = project(
            &before,
            vec![ConditionFact::new(
                "Ready",
                ConditionStatus::True,
                "AllReplicasReady",
                "3/3 ready",
            )],
            7,
            "2026-06-06T06:06:06Z",
        );
        assert_eq!(out[0].last_transition_time, "2026-01-01T00:00:00Z");
        // Everything else *does* refresh — only the transition instant is sticky.
        assert_eq!(out[0].reason, "AllReplicasReady");
        assert_eq!(out[0].message, "3/3 ready");
        assert_eq!(out[0].observed_generation, Some(7));
    }

    #[test]
    fn flipped_status_takes_the_injected_time() {
        let before = vec![prior("Ready", "True", "2026-01-01T00:00:00Z")];
        let out = project(
            &before,
            vec![ConditionFact::new(
                "Ready",
                ConditionStatus::False,
                "ReplicasNotReady",
                "1/3 ready",
            )],
            7,
            "2026-06-06T06:06:06Z",
        );
        assert_eq!(out[0].status, "False");
        assert_eq!(out[0].last_transition_time, "2026-06-06T06:06:06Z");
    }

    #[test]
    fn a_condition_seen_for_the_first_time_takes_the_injected_time() {
        let out = project(
            &[],
            vec![ConditionFact::new(
                "Progressing",
                ConditionStatus::Unknown,
                "NoObservation",
                "",
            )],
            0,
            "2026-06-06T06:06:06Z",
        );
        assert_eq!(out[0].last_transition_time, "2026-06-06T06:06:06Z");
        assert_eq!(out[0].status, "Unknown");
    }

    /// The same prior + facts must project identically no matter how often it
    /// runs — this is the property that lets services keep clock-free,
    /// deterministic status tests.
    #[test]
    fn projection_is_deterministic_and_order_preserving() {
        let before = vec![
            prior("Ready", "False", "2026-01-01T00:00:00Z"),
            prior("Progressing", "True", "2026-02-02T00:00:00Z"),
        ];
        let facts = || {
            vec![
                ConditionFact::new("Ready", ConditionStatus::False, "NotReady", "0/3"),
                ConditionFact::new("Progressing", ConditionStatus::True, "Converging", "0/3"),
            ]
        };
        let a = project(&before, facts(), 3, "2026-06-06T06:06:06Z");
        let b = project(&before, facts(), 3, "2026-06-06T06:06:06Z");
        assert_eq!(a, b);
        assert_eq!(a[0].type_, "Ready");
        assert_eq!(a[1].type_, "Progressing");
    }

    /// A condition that reappears after being dropped is a *new* condition —
    /// nothing to carry forward, so it takes the injected time.
    #[test]
    fn dropped_conditions_do_not_resurrect_their_old_transition_time() {
        let before = vec![prior("ReshardInProgress", "True", "2026-01-01T00:00:00Z")];
        let dropped = project(&before, vec![], 1, "2026-03-03T00:00:00Z");
        assert!(dropped.is_empty());
        let back = project(
            &dropped,
            vec![ConditionFact::new(
                "ReshardInProgress",
                ConditionStatus::True,
                "Splitting",
                "",
            )],
            2,
            "2026-04-04T00:00:00Z",
        );
        assert_eq!(back[0].last_transition_time, "2026-04-04T00:00:00Z");
    }

    #[test]
    fn serialized_shape_is_metav1_condition() {
        let out = project(
            &[],
            vec![ConditionFact::new(
                "Ready",
                ConditionStatus::True,
                "AllReplicasReady",
                "3/3 ready",
            )],
            9,
            "2026-06-06T06:06:06Z",
        );
        assert_eq!(
            serde_json::to_value(&out[0]).expect("serialize"),
            serde_json::json!({
                "type": "Ready",
                "status": "True",
                "reason": "AllReplicasReady",
                "message": "3/3 ready",
                "lastTransitionTime": "2026-06-06T06:06:06Z",
                "observedGeneration": 9,
            })
        );
    }

    #[test]
    fn now_rfc3339_has_no_subsecond_component() {
        let now = now_rfc3339();
        assert!(now.ends_with('Z'), "{now} is not UTC-suffixed");
        assert!(!now.contains('.'), "{now} carries sub-second precision");
    }
}
// </HANDWRITE>

/// The generic cluster shape every sharded-HA service embeds in its CRD spec via
/// `#[serde(flatten)] pub cluster: service_k8s::ClusterSpec`.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSpec {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<String>,
    #[serde(default = "one")]
    pub shard_count: u32,
    /// Starting/minimum members per shard. With startup-static raft-runtime
    /// membership this is also the fixed desired value; a future membership
    /// controller may plan whole replica layers above this floor.
    #[serde(default = "one")]
    pub replicas_per_shard: u32,
    #[serde(default = "one")]
    pub voter_count: u32,
    #[serde(default)]
    pub resources: ResourceSpec,
}

/// Per-pod CPU/memory requests. Empty values resolve to the shared data-plane
/// defaults (`1` CPU / `4Gi`) at render time. Limits are intentionally omitted
/// so a dedicated-node pod can use otherwise-idle node capacity.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpec {
    #[serde(default)]
    pub cpu: String,
    #[serde(default)]
    pub memory: String,
}

fn one() -> u32 {
    1
}
// CODEGEN-END
