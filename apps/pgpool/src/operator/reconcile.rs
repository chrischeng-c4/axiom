// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// <HANDWRITE gap="missing-generator:logic:8e369a2f" tracker="#1575" reason="Implement ManagedService readiness and status projection for Deployment replicas and expose the shared operator run loop.">
use std::future::Future;

use anyhow::{anyhow, Context};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Pod, Secret};
use kube::api::ListParams;
use kube::{Api, Client, ResourceExt};
use service_k8s::service::ReconcilePlan;
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use crate::k8s::{
    ControlPlaneStatus, EndpointCapacity, EndpointControlStatus, PodControlPhase, PodControlStatus,
};
use crate::platform::{
    discover_connection_facts, EndpointProvider, EndpointRole, ProviderAdvisory, RemoteEndpoint,
};

use super::crd::{Pgpool, PgpoolEndpointBudgetSpec, PgpoolEndpointProvider, PgpoolEndpointRole};
use super::render;

#[derive(Clone, Debug)]
struct EndpointObservation {
    spec: PgpoolEndpointBudgetSpec,
    capacity: Option<EndpointCapacity>,
    error: Option<String>,
}

impl ManagedService for Pgpool {
    const MANAGER: &'static str = "pgpool-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn reconcile_plan(
        &self,
        client: Client,
    ) -> impl Future<Output = anyhow::Result<ReconcilePlan>> + Send {
        let instance = self.clone();
        async move { build_reconcile_plan(instance, client).await }
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "Deployment",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let desired_replicas = self.spec.replicas as i32;
        let mut status = self.status.clone().unwrap_or_default();
        status.observed_generation = self.metadata.generation.unwrap_or(0);
        status.ready_replicas = ready_replicas;
        status.desired_replicas = desired_replicas;
        status.phase = if status.blocked_scale_reason.is_some() {
            "Blocked".into()
        } else if desired_replicas > 0 && ready_replicas >= desired_replicas {
            "Ready".into()
        } else if ready_replicas > 0 {
            "Reconciling".into()
        } else {
            "Pending".into()
        };
        status.message = format!("{ready_replicas}/{desired_replicas} pgpool pods ready");
        json!({ "status": status })
    }

    fn status_patch_with_context(
        &self,
        ready: &ReadyFacts,
        context: &serde_json::Value,
    ) -> serde_json::Value {
        let Ok(control) = serde_json::from_value::<ControlPlaneStatus>(context.clone()) else {
            return self.status_patch(ready);
        };
        let ready_replicas = ready.get(&self.name_any()) as i32;
        let status = super::crd::PgpoolStatus::from_control_plane(
            &self.spec,
            self.metadata.generation.unwrap_or(0),
            ready_replicas,
            &control,
        );
        json!({ "status": status })
    }
}

async fn build_reconcile_plan(instance: Pgpool, client: Client) -> anyhow::Result<ReconcilePlan> {
    let namespace = instance
        .namespace()
        .ok_or_else(|| anyhow!("Pgpool metadata.namespace is required"))?;
    let name = instance.name_any();
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let current_target = deployments
        .get_opt(&name)
        .await?
        .and_then(|deployment| deployment.spec.and_then(|spec| spec.replicas))
        .unwrap_or(0)
        .max(0) as u32;
    let selector = format!(
        "app.kubernetes.io/name=pgpool,app.kubernetes.io/instance={name},app.kubernetes.io/component=pool"
    );
    let pods: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let actual_pods = pods
        .list(&ListParams::default().labels(&selector))
        .await?
        .items
        .len() as u32;

    let mut observations = Vec::with_capacity(instance.spec.endpoints.len());
    for endpoint in &instance.spec.endpoints {
        match discover_endpoint(&client, &namespace, endpoint).await {
            Ok(capacity) => observations.push(EndpointObservation {
                spec: endpoint.clone(),
                capacity: Some(capacity),
                error: None,
            }),
            Err(error) => observations.push(EndpointObservation {
                spec: endpoint.clone(),
                capacity: None,
                error: Some(format!(
                    "endpoint {} discovery failed: {error}",
                    endpoint.name
                )),
            }),
        }
    }
    if observations.is_empty() {
        observations.push(EndpointObservation {
            spec: instance.spec.primary().clone(),
            capacity: None,
            error: Some("no remote endpoint budgets configured".into()),
        });
    }

    let (applied_replicas, status) = plan_capacity(
        &name,
        instance.spec.replicas,
        current_target,
        actual_pods,
        &observations,
    );
    let mut admitted = instance;
    admitted.spec.replicas = applied_replicas;
    Ok(ReconcilePlan {
        children: render::render(&admitted),
        context: serde_json::to_value(status)?,
    })
}

async fn discover_endpoint(
    client: &Client,
    namespace: &str,
    endpoint: &PgpoolEndpointBudgetSpec,
) -> anyhow::Result<EndpointCapacity> {
    let mut postgres = tokio_postgres::Config::new();
    postgres
        .host(&endpoint.host)
        .port(endpoint.port)
        .application_name("pgpool-operator");
    if let Some(database) = &endpoint.database {
        postgres.dbname(database);
    }
    if let Some(user) = &endpoint.user {
        postgres.user(user);
    }
    if let Some(reference) = &endpoint.password_secret_ref {
        let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
        let secret = secrets
            .get(&reference.name)
            .await
            .with_context(|| format!("read Secret {}", reference.name))?;
        let password = secret
            .data
            .as_ref()
            .and_then(|data| data.get(&reference.key))
            .ok_or_else(|| anyhow!("Secret {}/{} is missing", reference.name, reference.key))?;
        let password = std::str::from_utf8(&password.0)
            .with_context(|| format!("Secret {}/{} is not UTF-8", reference.name, reference.key))?;
        postgres.password(password);
    }
    let facts = discover_connection_facts(
        RemoteEndpoint {
            name: endpoint.name.clone(),
            provider: provider(endpoint.provider),
            role: role(endpoint.role),
            configured_ceiling: endpoint.configured_ceiling,
        },
        postgres,
        ProviderAdvisory::default(),
    )
    .await?;
    Ok(EndpointCapacity::from_discovery(
        &facts,
        endpoint.reserve,
        endpoint.safety_headroom,
    ))
}

fn plan_capacity(
    instance: &str,
    desired: u32,
    current_target: u32,
    actual_pods: u32,
    observations: &[EndpointObservation],
) -> (u32, ControlPlaneStatus) {
    let requested_fits = observations.iter().all(|item| {
        item.capacity.is_some_and(|capacity| {
            desired.saturating_mul(item.spec.per_pod_quota) <= capacity.usable()
        })
    });
    let applied = if requested_fits {
        desired
    } else {
        current_target
    };
    let held_pods = actual_pods.max(current_target).max(applied);
    let endpoints: Vec<_> = observations
        .iter()
        .map(|item| {
            let capacity = item.capacity.unwrap_or(EndpointCapacity {
                effective_limit: 0,
                reserve: item.spec.reserve,
                non_pgpool_usage: 0,
                safety_headroom: item.spec.safety_headroom,
            });
            let allocated = held_pods.saturating_mul(item.spec.per_pod_quota);
            let requested = desired.saturating_mul(item.spec.per_pod_quota);
            let blocked_scale_reason = item.error.clone().or_else(|| {
                (requested > capacity.usable()).then(|| {
                    format!(
                        "endpoint {} scale blocked: requested={}, usable={}, held={allocated}",
                        item.spec.name,
                        requested,
                        capacity.usable()
                    )
                })
            });
            EndpointControlStatus {
                endpoint: item.spec.name.clone(),
                effective_limit: capacity.effective_limit,
                reserve: capacity.reserve,
                non_pgpool_usage: capacity.non_pgpool_usage,
                safety_headroom: capacity.safety_headroom,
                usable: capacity.usable(),
                allocated,
                available: capacity.usable().saturating_sub(allocated),
                blocked_scale_reason,
            }
        })
        .collect();
    let pods = observations
        .iter()
        .flat_map(|endpoint| {
            (0..held_pods).map(move |index| PodControlStatus {
                pod: format!("{instance}-{index}@{}", endpoint.spec.name),
                endpoint: endpoint.spec.name.clone(),
                quota: endpoint.spec.per_pod_quota,
                phase: if index >= applied {
                    PodControlPhase::Draining
                } else {
                    PodControlPhase::Pending
                },
                ready: false,
                drain_requested: index >= applied,
                drain_deadline_epoch_seconds: None,
                backend_active: 0,
                backend_idle: 0,
            })
        })
        .collect();
    let blocked_scale_reason = endpoints
        .iter()
        .find_map(|endpoint| endpoint.blocked_scale_reason.clone());
    (
        applied,
        ControlPlaneStatus {
            endpoints,
            pods,
            blocked_scale_reason,
        },
    )
}

fn provider(provider: PgpoolEndpointProvider) -> EndpointProvider {
    match provider {
        PgpoolEndpointProvider::PlainPostgres => EndpointProvider::PlainPostgres,
        PgpoolEndpointProvider::CloudSql => EndpointProvider::CloudSql,
        PgpoolEndpointProvider::AlloyDb => EndpointProvider::AlloyDb,
    }
}

fn role(role: PgpoolEndpointRole) -> EndpointRole {
    match role {
        PgpoolEndpointRole::Primary => EndpointRole::Primary,
        PgpoolEndpointRole::ReadPool => EndpointRole::ReadPool,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(usable_limit: u32) -> EndpointObservation {
        EndpointObservation {
            spec: PgpoolEndpointBudgetSpec {
                per_pod_quota: 40,
                ..PgpoolEndpointBudgetSpec::default()
            },
            capacity: Some(EndpointCapacity {
                effective_limit: usable_limit,
                reserve: 0,
                non_pgpool_usage: 0,
                safety_headroom: 0,
            }),
            error: None,
        }
    }

    #[test]
    fn scale_out_is_admitted_before_deployment_target_changes() {
        let (applied, status) = plan_capacity("pool", 3, 1, 1, &[observation(120)]);
        assert_eq!(applied, 3);
        assert_eq!(status.endpoints[0].allocated, 120);
        assert!(status.blocked_scale_reason.is_none());
    }

    #[test]
    fn scale_above_live_capacity_keeps_current_target_and_reports_blocked() {
        let (applied, status) = plan_capacity("pool", 4, 2, 2, &[observation(120)]);
        assert_eq!(applied, 2);
        assert_eq!(status.endpoints[0].allocated, 80);
        assert!(status.blocked_scale_reason.is_some());
    }

    #[test]
    fn scale_in_holds_quota_for_terminating_pods_until_they_disappear() {
        let (applied, draining) = plan_capacity("pool", 1, 3, 3, &[observation(120)]);
        assert_eq!(applied, 1);
        assert_eq!(draining.endpoints[0].allocated, 120);
        assert_eq!(
            draining
                .pods
                .iter()
                .filter(|pod| pod.phase == PodControlPhase::Draining)
                .count(),
            2
        );

        let (_, released) = plan_capacity("pool", 1, 1, 1, &[observation(120)]);
        assert_eq!(released.endpoints[0].allocated, 40);
    }

    #[test]
    fn discovery_failure_never_scales_a_new_deployment() {
        let failed = EndpointObservation {
            spec: PgpoolEndpointBudgetSpec::default(),
            capacity: None,
            error: Some("discovery unavailable".into()),
        };
        let (applied, status) = plan_capacity("pool", 3, 0, 0, &[failed]);
        assert_eq!(applied, 0);
        assert!(status.blocked_scale_reason.is_some());
    }
}

pub async fn run() -> anyhow::Result<()> {
    ::service_k8s::run::<Pgpool>().await
}
// </HANDWRITE>
