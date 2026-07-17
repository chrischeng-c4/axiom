// SPEC-MANAGED: libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The [`ManagedService`] trait a service implements + the shared CRD fragments.

use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;

use kube::core::NamespaceResourceScope;
use kube::{Client, CustomResourceExt, Resource};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A workload to poll for `.status.readyReplicas` during reconcile.
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ReadinessTarget {
    pub kind: &'static str,
    pub name: String,
}

/// Observed readiness handed to [`ManagedService::status_patch`]
/// (workload name → `readyReplicas`).
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ReadyFacts {
    pub ready: HashMap<String, i64>,
}

/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
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
/// @spec apps/pgpool/tech-design/logic/converge-deployment-provider-and-reconcile-planning-on-service-k.md#logic
pub struct ReconcilePlan {
    pub children: Vec<serde_json::Value>,
    pub context: serde_json::Value,
}

/// One service's contribution to the shared operator. Implemented on the CRD
/// root type (e.g. lumen's `Lumen`). The [`crate::controller`] is generic over
/// `S`, so the watch/apply/lease loop is written once.
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
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
}

#[cfg(test)]
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

/// The generic cluster shape every sharded-HA service embeds in its CRD spec via
/// `#[serde(flatten)] pub cluster: service_k8s::ClusterSpec`.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
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
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
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
