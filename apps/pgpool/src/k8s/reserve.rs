// HANDWRITE-BEGIN gap="missing-generator:logic:f8226085" tracker="#1731" reason="Define deterministic reserve grant requests, grant tokens, expiry and reconciliation transitions independently of Kubernetes transport."
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A lease token is scoped to one endpoint and one Pod.  The operator chooses
/// it, so retries of the same request can be idempotent without granting a
/// second chunk of remote-Postgres capacity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReserveLeaseKey {
    pub pod: String,
    pub token: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReserveLeaseState {
    Granted,
    Connecting,
    Active,
    Idle,
    Draining,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveLeaseGrant {
    pub key: ReserveLeaseKey,
    pub units: u32,
    pub expires_at_epoch_seconds: u64,
    pub state: ReserveLeaseState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveLeaseRequest {
    pub pod: String,
    pub token: String,
    pub units: u32,
    pub expires_at_epoch_seconds: u64,
}

impl ReserveLeaseRequest {
    fn key(&self) -> ReserveLeaseKey {
        ReserveLeaseKey {
            pod: self.pod.clone(),
            token: self.token.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ReserveLeaseError {
    #[error("reserve lease units must be positive")]
    ZeroUnits,
    #[error("reserve lease expiry must be in the future")]
    ExpiredAtGrant,
    #[error(
        "endpoint {endpoint} reserve capacity exceeded: base={base_held}, held={held}, requested={requested}, usable={usable}"
    )]
    InsufficientCapacity {
        endpoint: String,
        base_held: u32,
        held: u32,
        requested: u32,
        usable: u32,
    },
    #[error("reserve lease {token} for Pod {pod} conflicts with the existing grant")]
    ConflictingRetry { pod: String, token: String },
    #[error("unknown reserve lease {token} for Pod {pod}")]
    UnknownLease { pod: String, token: String },
}

/// Deterministic operator-side accounting for one endpoint.  `base_held` is
/// the statically admitted Pod allocation.  Every reserve state continues to
/// count until the physical backend is known closed and `release_after_close`
/// removes it, including `Expired` and `Draining`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveLeaseLedger {
    pub endpoint: String,
    pub usable: u32,
    pub base_held: u32,
    grants: BTreeMap<ReserveLeaseKey, ReserveLeaseGrant>,
}

impl ReserveLeaseLedger {
    pub fn new(endpoint: impl Into<String>, usable: u32, base_held: u32) -> Self {
        Self {
            endpoint: endpoint.into(),
            usable,
            base_held,
            grants: BTreeMap::new(),
        }
    }

    pub fn grants(&self) -> impl Iterator<Item = &ReserveLeaseGrant> {
        self.grants.values()
    }

    pub fn held_reserve(&self) -> u32 {
        self.grants.values().map(|grant| grant.units).sum()
    }

    pub fn held_total(&self) -> u32 {
        self.base_held.saturating_add(self.held_reserve())
    }

    pub fn available(&self) -> u32 {
        self.usable.saturating_sub(self.held_total())
    }

    /// Atomically accepts a batch.  Identical retry records are accepted as
    /// no-ops; conflicting retries and any batch that does not fit leave the
    /// ledger unchanged.
    pub fn grant_many(
        &mut self,
        now_epoch_seconds: u64,
        requests: impl IntoIterator<Item = ReserveLeaseRequest>,
    ) -> Result<Vec<ReserveLeaseGrant>, ReserveLeaseError> {
        let requests: Vec<_> = requests.into_iter().collect();
        let mut new_units = 0_u32;
        for request in &requests {
            if request.units == 0 {
                return Err(ReserveLeaseError::ZeroUnits);
            }
            if request.expires_at_epoch_seconds <= now_epoch_seconds {
                return Err(ReserveLeaseError::ExpiredAtGrant);
            }
            if let Some(existing) = self.grants.get(&request.key()) {
                if existing.units != request.units
                    || existing.expires_at_epoch_seconds != request.expires_at_epoch_seconds
                {
                    return Err(ReserveLeaseError::ConflictingRetry {
                        pod: request.pod.clone(),
                        token: request.token.clone(),
                    });
                }
            } else {
                new_units = new_units.saturating_add(request.units);
            }
        }
        let held = self.held_reserve();
        if self
            .base_held
            .saturating_add(held)
            .saturating_add(new_units)
            > self.usable
        {
            return Err(ReserveLeaseError::InsufficientCapacity {
                endpoint: self.endpoint.clone(),
                base_held: self.base_held,
                held,
                requested: new_units,
                usable: self.usable,
            });
        }
        let mut granted = Vec::with_capacity(requests.len());
        for request in requests {
            let key = request.key();
            let grant = self
                .grants
                .entry(key.clone())
                .or_insert_with(|| ReserveLeaseGrant {
                    key,
                    units: request.units,
                    expires_at_epoch_seconds: request.expires_at_epoch_seconds,
                    state: ReserveLeaseState::Granted,
                })
                .clone();
            granted.push(grant);
        }
        debug_assert!(self.held_total() <= self.usable);
        Ok(granted)
    }

    pub fn transition(
        &mut self,
        key: &ReserveLeaseKey,
        state: ReserveLeaseState,
    ) -> Result<(), ReserveLeaseError> {
        let grant = self
            .grants
            .get_mut(key)
            .ok_or_else(|| ReserveLeaseError::UnknownLease {
                pod: key.pod.clone(),
                token: key.token.clone(),
            })?;
        grant.state = state;
        Ok(())
    }

    /// Expiry is intentionally not a capacity release.  The caller must close
    /// or prove absence of the physical backend before `release_after_close`.
    pub fn mark_expired(&mut self, now_epoch_seconds: u64) -> Vec<ReserveLeaseKey> {
        self.grants
            .values_mut()
            .filter_map(|grant| {
                (grant.expires_at_epoch_seconds <= now_epoch_seconds
                    && grant.state != ReserveLeaseState::Expired)
                    .then(|| {
                        grant.state = ReserveLeaseState::Expired;
                        grant.key.clone()
                    })
            })
            .collect()
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="Add endpoint-ledger per-Pod reserve grant reaping after physical drain completion.">
    pub fn release_after_close(
        &mut self,
        key: &ReserveLeaseKey,
    ) -> Result<ReserveLeaseGrant, ReserveLeaseError> {
        self.grants
            .remove(key)
            .ok_or_else(|| ReserveLeaseError::UnknownLease {
                pod: key.pod.clone(),
                token: key.token.clone(),
            })
    }

    /// A completed Pod drain proves every backend owned by that Pod is gone,
    /// so its remaining reserve leases can no longer hold endpoint capacity.
    pub fn reap_pod_after_drain(&mut self, pod: &str) -> Vec<ReserveLeaseGrant> {
        let keys: Vec<_> = self
            .grants
            .keys()
            .filter(|key| key.pod == pod)
            .cloned()
            .collect();
        keys.into_iter()
            .filter_map(|key| self.grants.remove(&key))
            .collect()
    }
    // </HANDWRITE>
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(pod: &str, token: &str, units: u32, expiry: u64) -> ReserveLeaseRequest {
        ReserveLeaseRequest {
            pod: pod.into(),
            token: token.into(),
            units,
            expires_at_epoch_seconds: expiry,
        }
    }

    #[test]
    fn concurrent_chunks_are_atomic_and_never_exceed_usable_capacity() {
        let mut ledger = ReserveLeaseLedger::new("primary", 100, 70);
        ledger
            .grant_many(10, [request("pod-a", "a", 15, 20)])
            .unwrap();
        let error = ledger
            .grant_many(
                10,
                [request("pod-b", "b", 10, 20), request("pod-c", "c", 10, 20)],
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ReserveLeaseError::InsufficientCapacity { .. }
        ));
        assert_eq!(ledger.held_total(), 85);
        assert_eq!(ledger.grants().count(), 1);
    }

    #[test]
    fn identical_retry_is_idempotent_but_conflicting_retry_is_rejected() {
        let mut ledger = ReserveLeaseLedger::new("primary", 10, 0);
        let original = request("pod-a", "lease", 5, 20);
        ledger.grant_many(10, [original.clone()]).unwrap();
        ledger.grant_many(10, [original]).unwrap();
        assert_eq!(ledger.held_reserve(), 5);
        assert!(matches!(
            ledger.grant_many(10, [request("pod-a", "lease", 6, 20)]),
            Err(ReserveLeaseError::ConflictingRetry { .. })
        ));
    }

    #[test]
    fn expired_and_draining_grants_remain_held_until_close_is_reconciled() {
        let mut ledger = ReserveLeaseLedger::new("primary", 10, 0);
        let grant = ledger
            .grant_many(10, [request("pod-a", "lease", 5, 20)])
            .unwrap()
            .pop()
            .unwrap();
        ledger
            .transition(&grant.key, ReserveLeaseState::Draining)
            .unwrap();
        assert_eq!(ledger.mark_expired(20), vec![grant.key.clone()]);
        assert_eq!(ledger.held_reserve(), 5);
        ledger.release_after_close(&grant.key).unwrap();
        assert_eq!(ledger.held_reserve(), 0);
    }
}

// marker: missing-generator:logic:f8226085 (filled for #1731)
// HANDWRITE-END
