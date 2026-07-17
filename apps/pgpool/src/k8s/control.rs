// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-drain-safe-control-plane-status.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-control-plane" tracker="#1573" reason="Reconciliation state-machine and metrics generation are not available.">
use std::collections::BTreeMap;

use metrics_prometheus::escape_label_value;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    AllocationError, GlobalConnectionBudget, ReserveLeaseError, ReserveLeaseGrant,
    ReserveLeaseLedger, ReserveLeaseRequest,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendPoolObservation {
    pub active: u32,
    pub idle: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PodControlPhase {
    Pending,
    Ready,
    Draining,
    Released,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodControlStatus {
    pub pod: String,
    pub endpoint: String,
    pub quota: u32,
    pub phase: PodControlPhase,
    pub ready: bool,
    pub drain_requested: bool,
    pub drain_deadline_epoch_seconds: Option<u64>,
    pub backend_active: u32,
    pub backend_idle: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointControlStatus {
    pub endpoint: String,
    pub effective_limit: u32,
    pub reserve: u32,
    pub non_pgpool_usage: u32,
    pub safety_headroom: u32,
    pub usable: u32,
    pub allocated: u32,
    pub available: u32,
    pub reserve_granted: u32,
    pub reserve_available: u32,
    /// True only when this status was derived from a reconciled reserve
    /// ledger, rather than a static capacity plan.
    pub reserve_accounting_available: bool,
    pub reserve_denials: u64,
    pub allocator_available: bool,
    pub blocked_scale_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPlaneStatus {
    pub endpoints: Vec<EndpointControlStatus>,
    pub pods: Vec<PodControlStatus>,
    pub blocked_scale_reason: Option<String>,
}

impl ControlPlaneStatus {
    // <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="Escape endpoint and Pod labels and reap Pod-owned reserve grants on completed drain.">
    pub fn prometheus(&self) -> String {
        let mut output = String::new();
        for endpoint in &self.endpoints {
            let label = format!("endpoint=\"{}\"", escape_label_value(&endpoint.endpoint));
            for (metric, value) in [
                ("effective_limit", endpoint.effective_limit),
                ("reserve", endpoint.reserve),
                ("non_pgpool_usage", endpoint.non_pgpool_usage),
                ("safety_headroom", endpoint.safety_headroom),
                ("usable", endpoint.usable),
                ("allocated", endpoint.allocated),
                ("available", endpoint.available),
                ("reserve_granted", endpoint.reserve_granted),
                ("reserve_available", endpoint.reserve_available),
            ] {
                output.push_str(&format!("pgpool_endpoint_{metric}{{{label}}} {value}\n"));
            }
            output.push_str(&format!(
                "pgpool_endpoint_reserve_denials{{{label}}} {}\n",
                endpoint.reserve_denials
            ));
            output.push_str(&format!(
                "pgpool_endpoint_reserve_accounting_available{{{label}}} {}\n",
                u8::from(endpoint.reserve_accounting_available)
            ));
            output.push_str(&format!(
                "pgpool_endpoint_scale_blocked{{{label}}} {}\n",
                u8::from(endpoint.blocked_scale_reason.is_some())
            ));
            output.push_str(&format!(
                "pgpool_endpoint_allocator_available{{{label}}} {}\n",
                u8::from(endpoint.allocator_available)
            ));
        }
        for pod in &self.pods {
            let labels = format!(
                "endpoint=\"{}\",pod=\"{}\"",
                escape_label_value(&pod.endpoint),
                escape_label_value(&pod.pod)
            );
            output.push_str(&format!("pgpool_pod_quota{{{labels}}} {}\n", pod.quota));
            output.push_str(&format!(
                "pgpool_pod_backend_active{{{labels}}} {}\n",
                pod.backend_active
            ));
            output.push_str(&format!(
                "pgpool_pod_backend_idle{{{labels}}} {}\n",
                pod.backend_idle
            ));
            output.push_str(&format!(
                "pgpool_pod_draining{{{labels}}} {}\n",
                u8::from(pod.phase == PodControlPhase::Draining)
            ));
        }
        output
    }
    // </HANDWRITE>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrainProgress {
    Held,
    Released,
}

#[derive(Debug, Error)]
pub enum ControlPlaneError {
    #[error("unknown endpoint {0}")]
    UnknownEndpoint(String),
    #[error("unknown pod {0}")]
    UnknownPod(String),
    #[error("pod {pod} is in phase {phase:?}, expected {expected:?}")]
    InvalidPodPhase {
        pod: String,
        phase: PodControlPhase,
        expected: PodControlPhase,
    },
    #[error(transparent)]
    Allocation(#[from] AllocationError),
    #[error(transparent)]
    Reserve(#[from] ReserveLeaseError),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgpoolControlPlane {
    pub budgets: GlobalConnectionBudget,
    reserve_ledgers: BTreeMap<String, ReserveLeaseLedger>,
    reserve_denials: BTreeMap<String, u64>,
    /// Pod control records are endpoint-scoped: one Deployment Pod may hold
    /// independent allocations for more than one remote endpoint.
    pods: BTreeMap<String, BTreeMap<String, PodControlStatus>>,
}

impl PgpoolControlPlane {
    pub fn new(budgets: GlobalConnectionBudget) -> Self {
        let reserve_ledgers = budgets
            .endpoints()
            .map(|(name, allocator)| {
                (
                    name.to_owned(),
                    ReserveLeaseLedger::new(
                        name,
                        allocator.capacity.usable(),
                        allocator.held_quota(),
                    ),
                )
            })
            .collect();
        Self {
            budgets,
            reserve_ledgers,
            reserve_denials: BTreeMap::new(),
            pods: BTreeMap::new(),
        }
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1888" reason="Pass outstanding reserve-grant units into static scale admission, reject instead of implicitly reclaiming grants, and cover admit/grant/release sequences.">
    /// Reserve desired and rollout-surge Pods in one transaction. The caller
    /// passes an empty surge set for the default no-surge Deployment policy.
    pub fn admit_scale<I, J, S, T>(
        &mut self,
        endpoint: &str,
        desired_new_pods: I,
        rollout_surge_pods: J,
        quota_per_pod: u32,
    ) -> Result<(), ControlPlaneError>
    where
        I: IntoIterator<Item = S>,
        J: IntoIterator<Item = T>,
        S: Into<String>,
        T: Into<String>,
    {
        let pods: Vec<String> = desired_new_pods
            .into_iter()
            .map(Into::into)
            .chain(rollout_surge_pods.into_iter().map(Into::into))
            .collect();
        let outstanding_reserve = self
            .reserve_ledgers
            .get(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?
            .held_reserve();
        {
            let allocator = self
                .budgets
                .endpoint_mut(endpoint)
                .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?;
            allocator.reserve_many_with_external_held(
                pods.clone(),
                quota_per_pod,
                outstanding_reserve,
            )?;
        }
        self.refresh_reserve_base(endpoint)?;
        let endpoint_pods = self.pods.entry(endpoint.into()).or_default();
        for pod in pods {
            endpoint_pods.insert(
                pod.clone(),
                PodControlStatus {
                    pod,
                    endpoint: endpoint.into(),
                    quota: quota_per_pod,
                    phase: PodControlPhase::Pending,
                    ready: false,
                    drain_requested: false,
                    drain_deadline_epoch_seconds: None,
                    backend_active: 0,
                    backend_idle: 0,
                },
            );
        }
        Ok(())
    }
    // </HANDWRITE>

    /// Mark exactly one endpoint allocation for a Pod as ready. The endpoint
    /// is explicit so a shared Pod name cannot route to the wrong allocator.
    pub fn mark_ready(&mut self, endpoint: &str, pod: &str) -> Result<(), ControlPlaneError> {
        self.pod(endpoint, pod)?;
        self.budgets
            .endpoint_mut(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?
            .mark_ready(pod)?;
        let status = self.pod_mut(endpoint, pod)?;
        status.phase = PodControlPhase::Ready;
        status.ready = true;
        Ok(())
    }

    /// First removes readiness, then records the drain request and deadline.
    pub fn begin_drain(
        &mut self,
        endpoint: &str,
        pod: &str,
        now_epoch_seconds: u64,
        timeout_seconds: u64,
    ) -> Result<(), ControlPlaneError> {
        self.pod(endpoint, pod)?;
        self.budgets
            .endpoint_mut(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?
            .begin_drain(pod)?;
        let status = self.pod_mut(endpoint, pod)?;
        status.ready = false;
        status.drain_requested = true;
        status.phase = PodControlPhase::Draining;
        status.drain_deadline_epoch_seconds =
            Some(now_epoch_seconds.saturating_add(timeout_seconds));
        Ok(())
    }

    pub fn observe_pool(
        &mut self,
        endpoint: &str,
        pod: &str,
        observation: BackendPoolObservation,
    ) -> Result<(), ControlPlaneError> {
        let status = self.pod_mut(endpoint, pod)?;
        status.backend_active = observation.active;
        status.backend_idle = observation.idle;
        Ok(())
    }

    pub fn reconcile_drain(
        &mut self,
        endpoint: &str,
        pod: &str,
        now_epoch_seconds: u64,
    ) -> Result<DrainProgress, ControlPlaneError> {
        let status = self.pod(endpoint, pod)?.clone();
        if status.phase != PodControlPhase::Draining {
            return Err(ControlPlaneError::InvalidPodPhase {
                pod: pod.into(),
                phase: status.phase,
                expected: PodControlPhase::Draining,
            });
        }
        let deadline_reached = status
            .drain_deadline_epoch_seconds
            .is_some_and(|deadline| now_epoch_seconds >= deadline);
        if status.backend_active > 0 && !deadline_reached {
            return Ok(DrainProgress::Held);
        }
        self.budgets
            .endpoint_mut(&status.endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(status.endpoint.clone()))?
            .release_after_drain(pod, true)?;
        self.refresh_reserve_base(&status.endpoint)?;
        self.reserve_ledgers
            .get_mut(&status.endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(status.endpoint.clone()))?
            .reap_pod_after_drain(pod);
        let status = self.pod_mut(&status.endpoint, pod)?;
        status.quota = 0;
        status.phase = PodControlPhase::Released;
        status.backend_active = 0;
        status.backend_idle = 0;
        Ok(DrainProgress::Released)
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1890" reason="Distinguish control-plane reserve ledger availability from endpoint discovery availability in the shared status context.">
    pub fn status(&self) -> ControlPlaneStatus {
        let endpoints: Vec<_> = self
            .budgets
            .endpoints()
            .map(|(name, allocator)| {
                let reserve = self.reserve_ledgers.get(name);
                EndpointControlStatus {
                    endpoint: name.into(),
                    effective_limit: allocator.capacity.effective_limit,
                    reserve: allocator.capacity.reserve,
                    non_pgpool_usage: allocator.capacity.non_pgpool_usage,
                    safety_headroom: allocator.capacity.safety_headroom,
                    usable: allocator.capacity.usable(),
                    allocated: allocator.held_quota(),
                    available: allocator.available_quota(),
                    reserve_granted: reserve.map(ReserveLeaseLedger::held_reserve).unwrap_or(0),
                    reserve_available: reserve.map(ReserveLeaseLedger::available).unwrap_or(0),
                    reserve_accounting_available: reserve.is_some(),
                    reserve_denials: self.reserve_denials.get(name).copied().unwrap_or(0),
                    allocator_available: reserve.is_some(),
                    blocked_scale_reason: allocator.blocked_reason().map(str::to_owned),
                }
            })
            .collect();
        let blocked_scale_reason = endpoints
            .iter()
            .find_map(|endpoint| endpoint.blocked_scale_reason.clone());
        ControlPlaneStatus {
            endpoints,
            pods: self
                .pods
                .values()
                .flat_map(|endpoint_pods| endpoint_pods.values().cloned())
                .collect(),
            blocked_scale_reason,
        }
    }
    // </HANDWRITE>

    fn pod(&self, endpoint: &str, pod: &str) -> Result<&PodControlStatus, ControlPlaneError> {
        self.pods
            .get(endpoint)
            .and_then(|endpoint_pods| endpoint_pods.get(pod))
            .ok_or_else(|| ControlPlaneError::UnknownPod(pod.into()))
    }

    fn pod_mut(
        &mut self,
        endpoint: &str,
        pod: &str,
    ) -> Result<&mut PodControlStatus, ControlPlaneError> {
        self.pods
            .get_mut(endpoint)
            .and_then(|endpoint_pods| endpoint_pods.get_mut(pod))
            .ok_or_else(|| ControlPlaneError::UnknownPod(pod.into()))
    }

    /// Atomically admit a reserve-grant chunk for one endpoint. This pure
    /// control-plane model is used by the operator reconciliation fixture;
    /// runtime Pods consume only the grants they receive from its background
    /// exchange.
    pub fn grant_reserve(
        &mut self,
        endpoint: &str,
        now_epoch_seconds: u64,
        requests: impl IntoIterator<Item = ReserveLeaseRequest>,
    ) -> Result<Vec<ReserveLeaseGrant>, ControlPlaneError> {
        let result = self
            .reserve_ledgers
            .get_mut(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?
            .grant_many(now_epoch_seconds, requests);
        match result {
            Ok(grants) => Ok(grants),
            Err(error) => {
                *self.reserve_denials.entry(endpoint.into()).or_default() += 1;
                Err(ControlPlaneError::Reserve(error))
            }
        }
    }

    pub fn reserve_ledger(&self, endpoint: &str) -> Option<&ReserveLeaseLedger> {
        self.reserve_ledgers.get(endpoint)
    }

    fn refresh_reserve_base(&mut self, endpoint: &str) -> Result<(), ControlPlaneError> {
        let allocator = self
            .budgets
            .endpoint(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?;
        let ledger = self
            .reserve_ledgers
            .get_mut(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?;
        ledger.usable = allocator.capacity.usable();
        ledger.base_held = allocator.held_quota();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k8s::{EndpointAllocator, EndpointCapacity};

    fn control_plane(usable: u32) -> PgpoolControlPlane {
        let mut budgets = GlobalConnectionBudget::default();
        budgets.insert(EndpointAllocator::new(
            "primary",
            EndpointCapacity {
                effective_limit: usable,
                reserve: 0,
                non_pgpool_usage: 0,
                safety_headroom: 0,
            },
        ));
        PgpoolControlPlane::new(budgets)
    }

    fn assert_combined_capacity_invariant(control: &PgpoolControlPlane) {
        let allocator = control.budgets.endpoint("primary").unwrap();
        let ledger = control.reserve_ledger("primary").unwrap();
        assert_eq!(ledger.base_held, allocator.held_quota());
        assert_eq!(
            ledger.held_total(),
            ledger.base_held + ledger.held_reserve()
        );
        assert!(ledger.held_total() <= allocator.capacity.usable());
    }

    #[test]
    fn desired_and_surge_pods_are_admitted_atomically_before_ready() {
        let mut control = control_plane(60);
        control
            .admit_scale("primary", ["pod-0", "pod-1"], ["pod-surge"], 20)
            .unwrap();
        assert_eq!(control.status().endpoints[0].allocated, 60);
        assert!(control.status().pods.iter().all(|pod| !pod.ready));
        assert!(control
            .admit_scale("primary", ["pod-2"], std::iter::empty::<&str>(), 1)
            .is_err());
        assert!(control.status().blocked_scale_reason.is_some());
        assert!(control.status().pods.iter().all(|pod| pod.pod != "pod-2"));
    }

    #[test]
    fn drain_removes_readiness_and_holds_until_sessions_finish() {
        let mut control = control_plane(20);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        control.mark_ready("primary", "pod-0").unwrap();
        control
            .observe_pool(
                "primary",
                "pod-0",
                BackendPoolObservation { active: 2, idle: 3 },
            )
            .unwrap();
        control.begin_drain("primary", "pod-0", 100, 30).unwrap();
        assert!(!control.status().pods[0].ready);
        assert_eq!(
            control.reconcile_drain("primary", "pod-0", 120).unwrap(),
            DrainProgress::Held
        );
        assert_eq!(control.status().endpoints[0].allocated, 20);
        control
            .observe_pool("primary", "pod-0", BackendPoolObservation::default())
            .unwrap();
        assert_eq!(
            control.reconcile_drain("primary", "pod-0", 121).unwrap(),
            DrainProgress::Released
        );
        assert_eq!(control.status().endpoints[0].allocated, 0);
    }

    #[test]
    fn drain_deadline_releases_even_with_remaining_sessions() {
        let mut control = control_plane(10);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 10)
            .unwrap();
        control.mark_ready("primary", "pod-0").unwrap();
        control
            .observe_pool(
                "primary",
                "pod-0",
                BackendPoolObservation { active: 1, idle: 0 },
            )
            .unwrap();
        control.begin_drain("primary", "pod-0", 100, 30).unwrap();
        assert_eq!(
            control.reconcile_drain("primary", "pod-0", 130).unwrap(),
            DrainProgress::Released
        );
    }

    #[test]
    fn status_and_metrics_expose_budget_activity_and_drain() {
        let mut control = control_plane(50);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        control.mark_ready("primary", "pod-0").unwrap();
        control
            .observe_pool(
                "primary",
                "pod-0",
                BackendPoolObservation { active: 3, idle: 7 },
            )
            .unwrap();
        control.begin_drain("primary", "pod-0", 0, 60).unwrap();
        let status = control.status();
        let metrics = status.prometheus();
        assert_eq!(status.endpoints[0].available, 30);
        assert!(metrics.contains("pgpool_endpoint_effective_limit{endpoint=\"primary\"} 50"));
        assert!(metrics.contains("pgpool_pod_backend_active{endpoint=\"primary\",pod=\"pod-0\"} 3"));
        assert!(metrics.contains("pgpool_pod_draining{endpoint=\"primary\",pod=\"pod-0\"} 1"));
    }

    #[test]
    fn control_plane_prometheus_escapes_hostile_labels() {
        let status = ControlPlaneStatus {
            endpoints: vec![EndpointControlStatus {
                endpoint: "primary\"\\\nnext".into(),
                effective_limit: 1,
                reserve: 0,
                non_pgpool_usage: 0,
                safety_headroom: 0,
                usable: 1,
                allocated: 0,
                available: 1,
                reserve_granted: 0,
                reserve_available: 0,
                reserve_accounting_available: false,
                reserve_denials: 0,
                allocator_available: true,
                blocked_scale_reason: None,
            }],
            pods: vec![PodControlStatus {
                pod: "pod\"\\\nnext".into(),
                endpoint: "primary\"\\\nnext".into(),
                quota: 1,
                phase: PodControlPhase::Pending,
                ready: false,
                drain_requested: false,
                drain_deadline_epoch_seconds: None,
                backend_active: 0,
                backend_idle: 0,
            }],
            blocked_scale_reason: None,
        };
        let rendered = status.prometheus();
        assert!(rendered.contains("endpoint=\"primary\\\"\\\\\\nnext\""));
        assert!(rendered.contains("pod=\"pod\\\"\\\\\\nnext\""));
        assert!(!rendered.contains("primary\"\\\nnext\""));
    }

    #[test]
    fn drain_completion_reaps_pod_reserve_grants() {
        let mut control = control_plane(30);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        control
            .grant_reserve(
                "primary",
                10,
                [ReserveLeaseRequest {
                    pod: "pod-0".into(),
                    token: "reserve-0".into(),
                    units: 5,
                    expires_at_epoch_seconds: 20,
                }],
            )
            .unwrap();
        control.mark_ready("primary", "pod-0").unwrap();
        control.begin_drain("primary", "pod-0", 10, 0).unwrap();
        assert_eq!(
            control.reconcile_drain("primary", "pod-0", 10).unwrap(),
            DrainProgress::Released
        );
        assert_eq!(control.reserve_ledger("primary").unwrap().held_reserve(), 0);
    }

    #[test]
    fn allocator_and_control_phase_stay_aligned() {
        let mut control = control_plane(10);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 10)
            .unwrap();
        control.mark_ready("primary", "pod-0").unwrap();
        let allocation = control
            .budgets
            .endpoint("primary")
            .unwrap()
            .allocations()
            .next()
            .unwrap();
        assert_eq!(allocation.state, crate::k8s::AllocationState::Ready);
    }

    #[test]
    fn reserve_aware_static_scale_admission_rejects_overcommit_without_mutation() {
        let mut control = control_plane(100);
        control
            .admit_scale("primary", ["pod-a"], std::iter::empty::<&str>(), 70)
            .unwrap();
        let grant = control
            .grant_reserve(
                "primary",
                10,
                [ReserveLeaseRequest {
                    pod: "pod-a".into(),
                    token: "reserve-a".into(),
                    units: 15,
                    expires_at_epoch_seconds: 20,
                }],
            )
            .unwrap()
            .pop()
            .unwrap();

        let error = control
            .admit_scale("primary", ["pod-b"], std::iter::empty::<&str>(), 30)
            .unwrap_err();
        assert!(matches!(
            error,
            ControlPlaneError::Allocation(AllocationError::InsufficientCapacity {
                held: 85,
                requested: 30,
                usable: 100,
                ..
            })
        ));
        assert_eq!(
            control.budgets.endpoint("primary").unwrap().held_quota(),
            70
        );
        assert_eq!(
            control.reserve_ledger("primary").unwrap().held_reserve(),
            15
        );
        assert_eq!(
            control.reserve_ledger("primary").unwrap().grants().count(),
            1
        );
        assert!(control.pod("primary", "pod-b").is_err());
        assert!(control.status().blocked_scale_reason.is_some());
        assert_combined_capacity_invariant(&control);

        control
            .reserve_ledgers
            .get_mut("primary")
            .unwrap()
            .release_after_close(&grant.key)
            .unwrap();
        assert_combined_capacity_invariant(&control);
    }

    #[test]
    fn reserve_and_static_sequence_never_exceeds_usable_capacity() {
        let mut control = control_plane(100);
        control
            .admit_scale("primary", ["pod-a"], std::iter::empty::<&str>(), 60)
            .unwrap();
        assert_combined_capacity_invariant(&control);

        control
            .grant_reserve(
                "primary",
                10,
                [ReserveLeaseRequest {
                    pod: "pod-a".into(),
                    token: "reserve-a".into(),
                    units: 20,
                    expires_at_epoch_seconds: 30,
                }],
            )
            .unwrap()
            .pop()
            .unwrap();
        assert_combined_capacity_invariant(&control);

        assert!(control
            .admit_scale("primary", ["pod-b"], std::iter::empty::<&str>(), 30)
            .is_err());
        assert_combined_capacity_invariant(&control);

        control.mark_ready("primary", "pod-a").unwrap();
        control.begin_drain("primary", "pod-a", 20, 0).unwrap();
        assert_eq!(
            control.reconcile_drain("primary", "pod-a", 20).unwrap(),
            DrainProgress::Released
        );
        assert_combined_capacity_invariant(&control);

        control
            .admit_scale("primary", ["pod-b"], std::iter::empty::<&str>(), 70)
            .unwrap();
        assert_combined_capacity_invariant(&control);

        control
            .admit_scale("primary", ["pod-c"], std::iter::empty::<&str>(), 30)
            .unwrap();
        assert_combined_capacity_invariant(&control);
    }

    #[test]
    fn endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations() {
        let mut budgets = GlobalConnectionBudget::default();
        for endpoint in ["primary", "read-pool"] {
            budgets.insert(EndpointAllocator::new(
                endpoint,
                EndpointCapacity {
                    effective_limit: 40,
                    reserve: 0,
                    non_pgpool_usage: 0,
                    safety_headroom: 0,
                },
            ));
        }
        let mut control = PgpoolControlPlane::new(budgets);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        control
            .admit_scale("read-pool", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        assert!(matches!(
            control.admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 1),
            Err(ControlPlaneError::Allocation(
                AllocationError::DuplicatePod { .. }
            ))
        ));

        for endpoint in ["primary", "read-pool"] {
            control.mark_ready(endpoint, "pod-0").unwrap();
            control.begin_drain(endpoint, "pod-0", 10, 0).unwrap();
            assert_eq!(
                control.reconcile_drain(endpoint, "pod-0", 10).unwrap(),
                DrainProgress::Released
            );
            assert_eq!(control.budgets.endpoint(endpoint).unwrap().held_quota(), 0);
        }
        assert_eq!(control.status().pods.len(), 2);
        assert!(control
            .status()
            .pods
            .iter()
            .all(|pod| pod.phase == PodControlPhase::Released));
    }
}
// </HANDWRITE>
