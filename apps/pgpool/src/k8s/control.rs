// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-drain-safe-control-plane-status.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-control-plane" tracker="#1573" reason="Reconciliation state-machine and metrics generation are not available.">
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{AllocationError, GlobalConnectionBudget};

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
    pub blocked_scale_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPlaneStatus {
    pub endpoints: Vec<EndpointControlStatus>,
    pub pods: Vec<PodControlStatus>,
    pub blocked_scale_reason: Option<String>,
}

impl ControlPlaneStatus {
    pub fn prometheus(&self) -> String {
        let mut output = String::new();
        for endpoint in &self.endpoints {
            let label = format!("endpoint=\"{}\"", endpoint.endpoint);
            for (metric, value) in [
                ("effective_limit", endpoint.effective_limit),
                ("reserve", endpoint.reserve),
                ("non_pgpool_usage", endpoint.non_pgpool_usage),
                ("safety_headroom", endpoint.safety_headroom),
                ("usable", endpoint.usable),
                ("allocated", endpoint.allocated),
                ("available", endpoint.available),
            ] {
                output.push_str(&format!("pgpool_endpoint_{metric}{{{label}}} {value}\n"));
            }
            output.push_str(&format!(
                "pgpool_endpoint_scale_blocked{{{label}}} {}\n",
                u8::from(endpoint.blocked_scale_reason.is_some())
            ));
        }
        for pod in &self.pods {
            let labels = format!("endpoint=\"{}\",pod=\"{}\"", pod.endpoint, pod.pod);
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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgpoolControlPlane {
    pub budgets: GlobalConnectionBudget,
    pods: BTreeMap<String, PodControlStatus>,
}

impl PgpoolControlPlane {
    pub fn new(budgets: GlobalConnectionBudget) -> Self {
        Self {
            budgets,
            pods: BTreeMap::new(),
        }
    }

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
        let allocator = self
            .budgets
            .endpoint_mut(endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.into()))?;
        allocator.reserve_many(pods.clone(), quota_per_pod)?;
        for pod in pods {
            self.pods.insert(
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

    pub fn mark_ready(&mut self, pod: &str) -> Result<(), ControlPlaneError> {
        let endpoint = self.pod(pod)?.endpoint.clone();
        self.budgets
            .endpoint_mut(&endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.clone()))?
            .mark_ready(pod)?;
        let status = self.pod_mut(pod)?;
        status.phase = PodControlPhase::Ready;
        status.ready = true;
        Ok(())
    }

    /// First removes readiness, then records the drain request and deadline.
    pub fn begin_drain(
        &mut self,
        pod: &str,
        now_epoch_seconds: u64,
        timeout_seconds: u64,
    ) -> Result<(), ControlPlaneError> {
        let endpoint = self.pod(pod)?.endpoint.clone();
        self.budgets
            .endpoint_mut(&endpoint)
            .ok_or_else(|| ControlPlaneError::UnknownEndpoint(endpoint.clone()))?
            .begin_drain(pod)?;
        let status = self.pod_mut(pod)?;
        status.ready = false;
        status.drain_requested = true;
        status.phase = PodControlPhase::Draining;
        status.drain_deadline_epoch_seconds =
            Some(now_epoch_seconds.saturating_add(timeout_seconds));
        Ok(())
    }

    pub fn observe_pool(
        &mut self,
        pod: &str,
        observation: BackendPoolObservation,
    ) -> Result<(), ControlPlaneError> {
        let status = self.pod_mut(pod)?;
        status.backend_active = observation.active;
        status.backend_idle = observation.idle;
        Ok(())
    }

    pub fn reconcile_drain(
        &mut self,
        pod: &str,
        now_epoch_seconds: u64,
    ) -> Result<DrainProgress, ControlPlaneError> {
        let status = self.pod(pod)?.clone();
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
        let status = self.pod_mut(pod)?;
        status.quota = 0;
        status.phase = PodControlPhase::Released;
        status.backend_active = 0;
        status.backend_idle = 0;
        Ok(DrainProgress::Released)
    }

    pub fn status(&self) -> ControlPlaneStatus {
        let endpoints: Vec<_> = self
            .budgets
            .endpoints()
            .map(|(name, allocator)| EndpointControlStatus {
                endpoint: name.into(),
                effective_limit: allocator.capacity.effective_limit,
                reserve: allocator.capacity.reserve,
                non_pgpool_usage: allocator.capacity.non_pgpool_usage,
                safety_headroom: allocator.capacity.safety_headroom,
                usable: allocator.capacity.usable(),
                allocated: allocator.held_quota(),
                available: allocator.available_quota(),
                blocked_scale_reason: allocator.blocked_reason().map(str::to_owned),
            })
            .collect();
        let blocked_scale_reason = endpoints
            .iter()
            .find_map(|endpoint| endpoint.blocked_scale_reason.clone());
        ControlPlaneStatus {
            endpoints,
            pods: self.pods.values().cloned().collect(),
            blocked_scale_reason,
        }
    }

    fn pod(&self, pod: &str) -> Result<&PodControlStatus, ControlPlaneError> {
        self.pods
            .get(pod)
            .ok_or_else(|| ControlPlaneError::UnknownPod(pod.into()))
    }

    fn pod_mut(&mut self, pod: &str) -> Result<&mut PodControlStatus, ControlPlaneError> {
        self.pods
            .get_mut(pod)
            .ok_or_else(|| ControlPlaneError::UnknownPod(pod.into()))
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
        control.mark_ready("pod-0").unwrap();
        control
            .observe_pool("pod-0", BackendPoolObservation { active: 2, idle: 3 })
            .unwrap();
        control.begin_drain("pod-0", 100, 30).unwrap();
        assert!(!control.status().pods[0].ready);
        assert_eq!(
            control.reconcile_drain("pod-0", 120).unwrap(),
            DrainProgress::Held
        );
        assert_eq!(control.status().endpoints[0].allocated, 20);
        control
            .observe_pool("pod-0", BackendPoolObservation::default())
            .unwrap();
        assert_eq!(
            control.reconcile_drain("pod-0", 121).unwrap(),
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
        control.mark_ready("pod-0").unwrap();
        control
            .observe_pool("pod-0", BackendPoolObservation { active: 1, idle: 0 })
            .unwrap();
        control.begin_drain("pod-0", 100, 30).unwrap();
        assert_eq!(
            control.reconcile_drain("pod-0", 130).unwrap(),
            DrainProgress::Released
        );
    }

    #[test]
    fn status_and_metrics_expose_budget_activity_and_drain() {
        let mut control = control_plane(50);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 20)
            .unwrap();
        control.mark_ready("pod-0").unwrap();
        control
            .observe_pool("pod-0", BackendPoolObservation { active: 3, idle: 7 })
            .unwrap();
        control.begin_drain("pod-0", 0, 60).unwrap();
        let status = control.status();
        let metrics = status.prometheus();
        assert_eq!(status.endpoints[0].available, 30);
        assert!(metrics.contains("pgpool_endpoint_effective_limit{endpoint=\"primary\"} 50"));
        assert!(metrics.contains("pgpool_pod_backend_active{endpoint=\"primary\",pod=\"pod-0\"} 3"));
        assert!(metrics.contains("pgpool_pod_draining{endpoint=\"primary\",pod=\"pod-0\"} 1"));
    }

    #[test]
    fn allocator_and_control_phase_stay_aligned() {
        let mut control = control_plane(10);
        control
            .admit_scale("primary", ["pod-0"], std::iter::empty::<&str>(), 10)
            .unwrap();
        control.mark_ready("pod-0").unwrap();
        let allocation = control
            .budgets
            .endpoint("primary")
            .unwrap()
            .allocations()
            .next()
            .unwrap();
        assert_eq!(allocation.state, crate::k8s::AllocationState::Ready);
    }
}
// </HANDWRITE>
