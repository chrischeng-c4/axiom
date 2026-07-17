// HANDWRITE-BEGIN gap="missing-generator:logic:e42a1f61" tracker="#1731" reason="Implement the asynchronous batched reserve lease cache/client, active-grant spend/reconcile rules, renewal, expiration, and diagnostic counters."
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::{Arc, Mutex};

use crate::k8s::{ReserveLeaseGrant, ReserveLeaseKey, ReserveLeaseRequest, ReserveLeaseState};

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="Represent reserve, queue, and idle timeouts as Duration and preserve sub-second elapsed time during local idle release.">
/// Runtime policy rendered by the operator.  The caller places
/// `reconcile_once` on a background task; acquire/relay code only calls
/// `try_spend`, which reads the local cache and never performs control-plane
/// I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveLeasePolicy {
    pub reserve_pool_timeout_seconds: u64,
    pub queue_wait_timeout_seconds: u64,
    pub reserve_idle_timeout_seconds: u64,
    pub lease_ttl_seconds: u64,
    pub request_chunk_size: u32,
}
// </HANDWRITE>

impl Default for ReserveLeasePolicy {
    fn default() -> Self {
        Self {
            reserve_pool_timeout_seconds: 1,
            queue_wait_timeout_seconds: 5,
            reserve_idle_timeout_seconds: 30,
            lease_ttl_seconds: 60,
            request_chunk_size: 1,
        }
    }
}

/// Process identity and policy for one endpoint's reserve worker.  The Pod
/// name is rendered through the Downward API so the allocator can make lease
/// ownership explicit without requiring StatefulSet identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReserveLeaseRuntimeConfig {
    pub endpoint: String,
    pub pod: String,
    pub policy: ReserveLeasePolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReserveLeaseDemand {
    pub endpoint: String,
    pub pod: String,
    pub units: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReserveLeaseBatch {
    pub endpoint: String,
    pub requests: Vec<ReserveLeaseRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReserveLeaseUse {
    pub endpoint: String,
    pub key: ReserveLeaseKey,
    spend_token: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReserveLeaseClientStats {
    pub queued_units: u32,
    pub granted_units: u32,
    pub spent_units: u32,
    pub denials: u64,
    pub expirations: u64,
    pub unavailable: u64,
}

#[derive(Clone, Debug)]
struct CachedGrant {
    grant: ReserveLeaseGrant,
    spent_tokens: BTreeSet<u64>,
    idle_since_epoch_seconds: Option<u64>,
}

#[derive(Debug, Default)]
struct ReserveLeaseClientState {
    queued: BTreeMap<(String, String), u32>,
    grants: BTreeMap<String, BTreeMap<ReserveLeaseKey, CachedGrant>>,
    next_token: u64,
    next_spend_token: u64,
    denials: u64,
    expirations: u64,
    unavailable: u64,
}

/// An endpoint-local, in-memory snapshot with a deliberately explicit
/// background reconciliation seam.  It does not know Kubernetes credentials
/// and therefore cannot accidentally put Kubernetes API I/O on the frontend
/// transaction hot path.
#[derive(Clone, Debug)]
pub struct ReserveLeaseClient {
    policy: ReserveLeasePolicy,
    state: Arc<Mutex<ReserveLeaseClientState>>,
}

impl ReserveLeaseClient {
    pub fn new(policy: ReserveLeasePolicy) -> Self {
        Self {
            policy,
            state: Arc::new(Mutex::new(ReserveLeaseClientState::default())),
        }
    }

    pub fn policy(&self) -> ReserveLeasePolicy {
        self.policy
    }

    /// Coalesce transaction waiters into a bounded endpoint+Pod demand.  The
    /// backend pool calls this only after reservePoolTimeout; it remains safe
    /// to call repeatedly while a batch is in flight.
    pub fn queue_demand(&self, demand: ReserveLeaseDemand) {
        if demand.units == 0 {
            return;
        }
        let mut state = self.state.lock().expect("reserve lease state lock");
        let key = (demand.endpoint, demand.pod);
        let pending = state.queued.entry(key).or_default();
        *pending = pending.saturating_add(demand.units);
    }

    /// Take one endpoint-local chunk for a background control-plane request.
    /// Requests are bounded and get stable tokens before dispatch, so a retry
    /// can be idempotent at the operator.
    pub fn take_next_batch(&self, now_epoch_seconds: u64) -> Option<ReserveLeaseBatch> {
        let mut state = self.state.lock().expect("reserve lease state lock");
        let ((endpoint, pod), requested_units) = state
            .queued
            .iter()
            .next()
            .map(|((endpoint, pod), units)| ((endpoint.clone(), pod.clone()), *units))?;
        state.queued.remove(&(endpoint.clone(), pod.clone()));
        let units = requested_units.min(self.policy.request_chunk_size.max(1));
        if requested_units > units {
            state
                .queued
                .insert((endpoint.clone(), pod.clone()), requested_units - units);
        }
        state.next_token = state.next_token.saturating_add(1);
        Some(ReserveLeaseBatch {
            endpoint,
            requests: vec![ReserveLeaseRequest {
                pod,
                token: format!("reserve-{}", state.next_token),
                units,
                expires_at_epoch_seconds: now_epoch_seconds
                    .saturating_add(self.policy.lease_ttl_seconds.max(1)),
            }],
        })
    }

    /// Run one batched control-plane exchange.  The supplied future belongs
    /// to the caller's background worker.  No state lock crosses `.await`,
    /// and a transport failure requeues the exact batch instead of allowing a
    /// speculative local grant.
    pub async fn reconcile_once<F, Fut, E>(
        &self,
        now_epoch_seconds: u64,
        dispatch: F,
    ) -> Result<usize, E>
    where
        F: FnOnce(ReserveLeaseBatch) -> Fut,
        Fut: Future<Output = Result<Vec<ReserveLeaseGrant>, E>>,
    {
        let Some(batch) = self.take_next_batch(now_epoch_seconds) else {
            return Ok(0);
        };
        let retry = batch.clone();
        match dispatch(batch).await {
            Ok(grants) => {
                let count = grants.len();
                self.install_grants(retry.endpoint, grants);
                Ok(count)
            }
            Err(error) => {
                self.requeue_batch(retry);
                self.record_unavailable();
                Err(error)
            }
        }
    }

    pub fn install_grants(&self, endpoint: impl Into<String>, grants: Vec<ReserveLeaseGrant>) {
        let endpoint = endpoint.into();
        let mut state = self.state.lock().expect("reserve lease state lock");
        let cached = state.grants.entry(endpoint).or_default();
        for grant in grants {
            cached.entry(grant.key.clone()).or_insert(CachedGrant {
                grant,
                spent_tokens: BTreeSet::new(),
                idle_since_epoch_seconds: None,
            });
        }
    }

    /// Hot-path lookup: returns one cached, unexpired grant unit or None.  It
    /// never refreshes, requests, or releases a Kubernetes resource.
    pub fn try_spend(&self, endpoint: &str, now_epoch_seconds: u64) -> Option<ReserveLeaseUse> {
        let mut state = self.state.lock().expect("reserve lease state lock");
        state.next_spend_token = state.next_spend_token.saturating_add(1);
        let spend_token = state.next_spend_token;
        let grants = state.grants.get_mut(endpoint)?;
        for (key, cached) in grants.iter_mut() {
            if cached.grant.expires_at_epoch_seconds <= now_epoch_seconds
                || cached.grant.state == ReserveLeaseState::Expired
                || cached.spent_tokens.len() >= cached.grant.units as usize
            {
                continue;
            }
            cached.spent_tokens.insert(spend_token);
            cached.idle_since_epoch_seconds = None;
            cached.grant.state = ReserveLeaseState::Connecting;
            return Some(ReserveLeaseUse {
                endpoint: endpoint.into(),
                key: key.clone(),
                spend_token,
            });
        }
        None
    }

    pub fn transition_use(&self, lease: &ReserveLeaseUse, state_kind: ReserveLeaseState) -> bool {
        let mut state = self.state.lock().expect("reserve lease state lock");
        let Some(cached) = state
            .grants
            .get_mut(&lease.endpoint)
            .and_then(|grants| grants.get_mut(&lease.key))
        else {
            return false;
        };
        cached.grant.state = state_kind;
        true
    }

    /// A close, failed connect, or failed reset returns the locally spent unit
    /// once.  Idle timeout collection may later return the whole unused grant
    /// to the operator.
    pub fn return_spend(&self, lease: &ReserveLeaseUse, now_epoch_seconds: u64) -> bool {
        let mut state = self.state.lock().expect("reserve lease state lock");
        let Some(cached) = state
            .grants
            .get_mut(&lease.endpoint)
            .and_then(|grants| grants.get_mut(&lease.key))
        else {
            return false;
        };
        if !cached.spent_tokens.remove(&lease.spend_token) {
            return false;
        }
        if cached.spent_tokens.is_empty() {
            cached.grant.state = ReserveLeaseState::Idle;
            cached
                .idle_since_epoch_seconds
                .get_or_insert(now_epoch_seconds);
        } else {
            cached.grant.state = ReserveLeaseState::Active;
        }
        true
    }

    /// Mark expired cache records unavailable to the data plane, but retain
    /// their accounting record for the background worker to safely reconcile.
    pub fn expire(&self, now_epoch_seconds: u64) -> Vec<(String, ReserveLeaseKey)> {
        let mut state = self.state.lock().expect("reserve lease state lock");
        let mut expired = Vec::new();
        for (endpoint, grants) in &mut state.grants {
            for (key, cached) in grants {
                if cached.grant.expires_at_epoch_seconds <= now_epoch_seconds
                    && cached.grant.state != ReserveLeaseState::Expired
                {
                    cached.grant.state = ReserveLeaseState::Expired;
                    expired.push((endpoint.clone(), key.clone()));
                }
            }
        }
        state.expirations = state.expirations.saturating_add(expired.len() as u64);
        expired
    }

    /// Remove only an unused and idle cached grant after the configured TTL;
    /// the caller sends the resulting key to the allocator asynchronously.
    pub fn collect_idle_releases(&self, now_epoch_seconds: u64) -> Vec<(String, ReserveLeaseKey)> {
        let mut state = self.state.lock().expect("reserve lease state lock");
        let mut releases = Vec::new();
        for (endpoint, grants) in &mut state.grants {
            let keys: Vec<_> = grants
                .iter()
                .filter_map(|(key, cached)| {
                    (cached.spent_tokens.is_empty()
                        && cached.idle_since_epoch_seconds.is_some_and(|idle_since| {
                            now_epoch_seconds
                                >= idle_since
                                    .saturating_add(self.policy.reserve_idle_timeout_seconds)
                        }))
                    .then(|| key.clone())
                })
                .collect();
            for key in keys {
                grants.remove(&key);
                releases.push((endpoint.clone(), key));
            }
        }
        releases
    }

    pub fn record_denial(&self) {
        let mut state = self.state.lock().expect("reserve lease state lock");
        state.denials = state.denials.saturating_add(1);
    }

    pub fn record_unavailable(&self) {
        let mut state = self.state.lock().expect("reserve lease state lock");
        state.unavailable = state.unavailable.saturating_add(1);
    }

    pub fn stats(&self) -> ReserveLeaseClientStats {
        let state = self.state.lock().expect("reserve lease state lock");
        ReserveLeaseClientStats {
            queued_units: state.queued.values().copied().sum(),
            granted_units: state
                .grants
                .values()
                .flat_map(|grants| grants.values())
                .map(|cached| cached.grant.units)
                .sum(),
            spent_units: state
                .grants
                .values()
                .flat_map(|grants| grants.values())
                .map(|cached| cached.spent_tokens.len() as u32)
                .sum(),
            denials: state.denials,
            expirations: state.expirations,
            unavailable: state.unavailable,
        }
    }

    fn requeue_batch(&self, batch: ReserveLeaseBatch) {
        let mut state = self.state.lock().expect("reserve lease state lock");
        for request in batch.requests {
            let pending = state
                .queued
                .entry((batch.endpoint.clone(), request.pod))
                .or_default();
            *pending = pending.saturating_add(request.units);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant(token: &str, units: u32, expiry: u64) -> ReserveLeaseGrant {
        ReserveLeaseGrant {
            key: ReserveLeaseKey {
                pod: "pod-a".into(),
                token: token.into(),
            },
            units,
            expires_at_epoch_seconds: expiry,
            state: ReserveLeaseState::Granted,
        }
    }

    #[test]
    fn demand_is_chunked_and_spending_uses_only_cached_grants() {
        let client = ReserveLeaseClient::new(ReserveLeasePolicy {
            request_chunk_size: 2,
            ..ReserveLeasePolicy::default()
        });
        client.queue_demand(ReserveLeaseDemand {
            endpoint: "primary".into(),
            pod: "pod-a".into(),
            units: 3,
        });
        let batch = client.take_next_batch(10).unwrap();
        assert_eq!(batch.requests[0].units, 2);
        assert!(client.try_spend("primary", 10).is_none());
        client.install_grants("primary", vec![grant("g", 1, 20)]);
        assert!(client.try_spend("primary", 10).is_some());
        assert_eq!(client.stats().queued_units, 1);
    }

    #[test]
    fn expiration_fails_closed_and_close_returns_spent_unit_once() {
        let client = ReserveLeaseClient::new(ReserveLeasePolicy::default());
        client.install_grants("primary", vec![grant("g", 1, 20)]);
        let lease = client.try_spend("primary", 10).unwrap();
        assert!(client.return_spend(&lease, 11));
        assert!(!client.return_spend(&lease, 11));
        assert_eq!(client.collect_idle_releases(41).len(), 1);
        client.install_grants("primary", vec![grant("expired", 1, 20)]);
        assert_eq!(client.expire(20).len(), 1);
        assert!(client.try_spend("primary", 20).is_none());
    }

    #[test]
    fn each_spend_has_a_distinct_return_token() {
        let client = ReserveLeaseClient::new(ReserveLeasePolicy::default());
        client.install_grants("primary", vec![grant("g", 2, 20)]);
        let first = client.try_spend("primary", 10).unwrap();
        let second = client.try_spend("primary", 10).unwrap();

        assert!(client.return_spend(&first, 11));
        assert!(!client.return_spend(&first, 11));
        assert_eq!(client.stats().spent_units, 1);
        assert!(client.return_spend(&second, 11));
        assert_eq!(client.stats().spent_units, 0);
    }
}

// marker: missing-generator:logic:e42a1f61 (filled for #1731)
// HANDWRITE-END
