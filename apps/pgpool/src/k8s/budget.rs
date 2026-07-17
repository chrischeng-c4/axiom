// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-global-endpoint-quota-allocation.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-endpoint-budget" tracker="#1571" reason="Quota state-machine generation is not available.">
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::platform::ConnectionFacts;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationState {
    Pending,
    Ready,
    Draining,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodAllocation {
    pub pod: String,
    pub quota: u32,
    pub state: AllocationState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointCapacity {
    pub effective_limit: u32,
    pub reserve: u32,
    pub non_pgpool_usage: u32,
    pub safety_headroom: u32,
}

impl EndpointCapacity {
    pub fn usable(self) -> u32 {
        self.effective_limit
            .saturating_sub(self.reserve)
            .saturating_sub(self.non_pgpool_usage)
            .saturating_sub(self.safety_headroom)
    }

    pub fn from_discovery(facts: &ConnectionFacts, reserve: u32, safety_headroom: u32) -> Self {
        Self {
            effective_limit: facts.effective_max_connections,
            reserve,
            non_pgpool_usage: facts.non_pgpool_connections,
            safety_headroom,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointAllocator {
    pub endpoint: String,
    pub capacity: EndpointCapacity,
    allocations: BTreeMap<String, PodAllocation>,
    blocked_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum AllocationError {
    #[error(
        "endpoint {endpoint} has insufficient capacity: held={held}, requested={requested}, usable={usable}"
    )]
    InsufficientCapacity {
        endpoint: String,
        held: u32,
        requested: u32,
        usable: u32,
    },
    #[error("pod {pod} already holds an allocation on endpoint {endpoint}")]
    DuplicatePod { endpoint: String, pod: String },
    #[error("pod {pod} has no allocation on endpoint {endpoint}")]
    UnknownPod { endpoint: String, pod: String },
    #[error("pod {pod} must be draining before its quota can be released")]
    NotDraining { pod: String },
    #[error("pod {pod} drain has not completed; quota remains held")]
    DrainIncomplete { pod: String },
}

impl EndpointAllocator {
    pub fn new(endpoint: impl Into<String>, capacity: EndpointCapacity) -> Self {
        Self {
            endpoint: endpoint.into(),
            capacity,
            allocations: BTreeMap::new(),
            blocked_reason: None,
        }
    }

    pub fn held_quota(&self) -> u32 {
        self.allocations.values().map(|item| item.quota).sum()
    }

    pub fn available_quota(&self) -> u32 {
        self.capacity.usable().saturating_sub(self.held_quota())
    }

    /// Shared cap predicate for static Pod allocations and any subsequently
    /// admitted reserve ledger.  Callers must keep an unexpired reserve grant
    /// held until its physical backend is closed; this method never treats a
    /// failed probe or an expired token as free capacity by itself.
    pub fn can_hold_additional(&self, requested: u32) -> bool {
        self.held_quota().saturating_add(requested) <= self.capacity.usable()
    }

    pub fn allocations(&self) -> impl Iterator<Item = &PodAllocation> {
        self.allocations.values()
    }

    pub fn blocked_reason(&self) -> Option<&str> {
        self.blocked_reason.as_deref()
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1888" reason="Admit static Pod quota against the allocator quota plus externally-held reserve capacity, preserving the allocator's atomic error and blocked-scale status behavior.">
    pub fn reserve_many<I, S>(&mut self, pods: I, quota_per_pod: u32) -> Result<(), AllocationError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.reserve_many_with_external_held(pods, quota_per_pod, 0)
    }

    /// Atomically reserve static Pod quota while accounting for capacity that
    /// is held outside this allocator, such as outstanding reserve grants.
    /// The external hold is advisory input from the same control-plane
    /// transaction; it is never reclaimed as a side effect of static admission.
    pub fn reserve_many_with_external_held<I, S>(
        &mut self,
        pods: I,
        quota_per_pod: u32,
        external_held: u32,
    ) -> Result<(), AllocationError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let pods: Vec<String> = pods.into_iter().map(Into::into).collect();
        let mut seen = BTreeSet::new();
        for pod in &pods {
            if !seen.insert(pod) || self.allocations.contains_key(pod) {
                return Err(AllocationError::DuplicatePod {
                    endpoint: self.endpoint.clone(),
                    pod: pod.clone(),
                });
            }
        }
        let requested = quota_per_pod.saturating_mul(pods.len().try_into().unwrap_or(u32::MAX));
        let held = self.held_quota().saturating_add(external_held);
        let usable = self.capacity.usable();
        if held.saturating_add(requested) > usable {
            let error = AllocationError::InsufficientCapacity {
                endpoint: self.endpoint.clone(),
                held,
                requested,
                usable,
            };
            self.blocked_reason = Some(error.to_string());
            return Err(error);
        }
        for pod in pods {
            self.allocations.insert(
                pod.clone(),
                PodAllocation {
                    pod,
                    quota: quota_per_pod,
                    state: AllocationState::Pending,
                },
            );
        }
        self.blocked_reason = None;
        debug_assert!(self.held_quota().saturating_add(external_held) <= usable);
        Ok(())
    }
    // </HANDWRITE>

    pub fn mark_ready(&mut self, pod: &str) -> Result<(), AllocationError> {
        self.allocation_mut(pod)?.state = AllocationState::Ready;
        Ok(())
    }

    pub fn begin_drain(&mut self, pod: &str) -> Result<(), AllocationError> {
        self.allocation_mut(pod)?.state = AllocationState::Draining;
        Ok(())
    }

    pub fn release_after_drain(
        &mut self,
        pod: &str,
        drain_complete: bool,
    ) -> Result<PodAllocation, AllocationError> {
        let allocation = self
            .allocations
            .get(pod)
            .ok_or_else(|| AllocationError::UnknownPod {
                endpoint: self.endpoint.clone(),
                pod: pod.into(),
            })?;
        if allocation.state != AllocationState::Draining {
            return Err(AllocationError::NotDraining { pod: pod.into() });
        }
        if !drain_complete {
            return Err(AllocationError::DrainIncomplete { pod: pod.into() });
        }
        Ok(self
            .allocations
            .remove(pod)
            .expect("allocation still present"))
    }

    fn allocation_mut(&mut self, pod: &str) -> Result<&mut PodAllocation, AllocationError> {
        self.allocations
            .get_mut(pod)
            .ok_or_else(|| AllocationError::UnknownPod {
                endpoint: self.endpoint.clone(),
                pod: pod.into(),
            })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConnectionBudget {
    endpoints: BTreeMap<String, EndpointAllocator>,
}

impl GlobalConnectionBudget {
    pub fn insert(&mut self, allocator: EndpointAllocator) {
        self.endpoints.insert(allocator.endpoint.clone(), allocator);
    }

    pub fn endpoint(&self, name: &str) -> Option<&EndpointAllocator> {
        self.endpoints.get(name)
    }

    pub fn endpoint_mut(&mut self, name: &str) -> Option<&mut EndpointAllocator> {
        self.endpoints.get_mut(name)
    }

    pub fn endpoints(&self) -> impl Iterator<Item = (&str, &EndpointAllocator)> {
        self.endpoints
            .iter()
            .map(|(name, allocator)| (name.as_str(), allocator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allocator() -> EndpointAllocator {
        EndpointAllocator::new(
            "primary",
            EndpointCapacity {
                effective_limit: 100,
                reserve: 10,
                non_pgpool_usage: 20,
                safety_headroom: 10,
            },
        )
    }

    #[test]
    fn atomic_scale_out_never_partially_allocates() {
        let mut allocator = allocator();
        allocator.reserve_many(["pod-0", "pod-1"], 25).unwrap();
        let error = allocator.reserve_many(["pod-2", "pod-3"], 10).unwrap_err();
        assert!(matches!(
            error,
            AllocationError::InsufficientCapacity { .. }
        ));
        assert_eq!(allocator.held_quota(), 50);
        assert!(allocator.allocations().all(|item| item.pod != "pod-2"));
        assert!(allocator.blocked_reason().is_some());
    }

    #[test]
    fn duplicate_pod_batch_is_rejected_atomically() {
        let mut allocator = allocator();
        assert!(matches!(
            allocator.reserve_many(["pod-0", "pod-0"], 10),
            Err(AllocationError::DuplicatePod { .. })
        ));
        assert_eq!(allocator.held_quota(), 0);
        assert_eq!(allocator.allocations().count(), 0);
    }

    #[test]
    fn pending_ready_and_draining_all_hold_quota_until_completion() {
        let mut allocator = allocator();
        allocator.reserve_many(["pod-0"], 20).unwrap();
        allocator.mark_ready("pod-0").unwrap();
        allocator.begin_drain("pod-0").unwrap();
        assert_eq!(allocator.held_quota(), 20);
        assert!(matches!(
            allocator.release_after_drain("pod-0", false),
            Err(AllocationError::DrainIncomplete { .. })
        ));
        assert_eq!(
            allocator.release_after_drain("pod-0", true).unwrap().quota,
            20
        );
        assert_eq!(allocator.held_quota(), 0);
    }

    #[test]
    fn endpoints_are_isolated() {
        let mut global = GlobalConnectionBudget::default();
        global.insert(allocator());
        global.insert(EndpointAllocator::new(
            "read-pool",
            EndpointCapacity {
                effective_limit: 20,
                reserve: 0,
                non_pgpool_usage: 0,
                safety_headroom: 0,
            },
        ));
        global
            .endpoint_mut("primary")
            .unwrap()
            .reserve_many(["pod-0"], 50)
            .unwrap();
        assert_eq!(global.endpoint("read-pool").unwrap().available_quota(), 20);
    }

    #[test]
    fn usable_capacity_saturates_at_zero() {
        let capacity = EndpointCapacity {
            effective_limit: 5,
            reserve: 10,
            non_pgpool_usage: 10,
            safety_headroom: 10,
        };
        assert_eq!(capacity.usable(), 0);
    }
}
// </HANDWRITE>
