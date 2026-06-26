//! Minimal Lease-based leader election (coordination.k8s.io/v1).
//!
//! kube-rs 0.98 ships no built-in elector, so this is a small hand-rolled one:
//! every operator replica runs the watch + reconcile loop, but only the replica
//! that currently holds the `<manager>` Lease actually applies changes (the
//! reconcile loop gates on [`Election::is_leader`]). A background task
//! acquires/renews the Lease; if the holder's renewal lapses past the lease
//! duration, another replica takes over. This makes `replicas > 1` safe (no two
//! reconcilers fighting) without an external dependency.
//!
//! Lifted from lumen's operator; the Lease name is now a parameter (the
//! service's `MANAGER`) so two different operators never share one Lease.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use k8s_openapi::api::coordination::v1::{Lease, LeaseSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use kube::api::{Api, ObjectMeta, PostParams};
use kube::Client;

/// Lease timing. The renew interval is comfortably under the duration so a
/// healthy leader never lets the lease lapse.
const LEASE_DURATION_SECS: i32 = 15;
const RENEW_INTERVAL: Duration = Duration::from_secs(5);

/// Shared leadership flag, flipped by the background election task and read by
/// the reconcile loop.
pub struct Election {
    pub is_leader: AtomicBool,
    pub identity: String,
}

impl Election {
    pub fn new(identity: String) -> Arc<Self> {
        Arc::new(Self {
            is_leader: AtomicBool::new(false),
            identity,
        })
    }
}

/// Pure leadership decision: may `identity` hold the lease now? True when the
/// lease is unheld, already held by us, or expired (renewal lapsed past the
/// duration). False only when a *different* identity holds a still-fresh lease.
/// Factored out for unit testing — no cluster, no clock.
fn may_acquire(
    holder: Option<&str>,
    renew_epoch_secs: Option<i64>,
    lease_dur_secs: i64,
    identity: &str,
    now_epoch_secs: i64,
) -> bool {
    match holder {
        None => true,
        Some(h) if h == identity => true,
        Some(_) => match renew_epoch_secs {
            Some(r) => now_epoch_secs.saturating_sub(r) > lease_dur_secs,
            None => true,
        },
    }
}

/// Try to acquire or renew the lease once. Returns whether we hold it after the
/// attempt. Any API error → treat as "not leader" (fail safe: a follower never
/// applies).
async fn acquire_or_renew(api: &Api<Lease>, lease_name: &str, identity: &str) -> bool {
    let now = chrono::Utc::now();
    let now_secs = now.timestamp();
    match api.get_opt(lease_name).await {
        Ok(Some(mut lease)) => {
            let spec = lease.spec.get_or_insert_with(LeaseSpec::default);
            let holder = spec.holder_identity.clone();
            let renew_secs = spec.renew_time.as_ref().map(|MicroTime(t)| t.timestamp());
            let dur = spec.lease_duration_seconds.unwrap_or(LEASE_DURATION_SECS) as i64;
            if !may_acquire(holder.as_deref(), renew_secs, dur, identity, now_secs) {
                return false;
            }
            let renewing = holder.as_deref() == Some(identity);
            spec.holder_identity = Some(identity.to_string());
            spec.renew_time = Some(MicroTime(now));
            spec.lease_duration_seconds = Some(LEASE_DURATION_SECS);
            if !renewing {
                spec.acquire_time = Some(MicroTime(now));
            }
            api.replace(lease_name, &PostParams::default(), &lease)
                .await
                .is_ok()
        }
        Ok(None) => {
            let lease = Lease {
                metadata: ObjectMeta {
                    name: Some(lease_name.to_string()),
                    ..Default::default()
                },
                spec: Some(LeaseSpec {
                    holder_identity: Some(identity.to_string()),
                    acquire_time: Some(MicroTime(now)),
                    renew_time: Some(MicroTime(now)),
                    lease_duration_seconds: Some(LEASE_DURATION_SECS),
                    ..Default::default()
                }),
            };
            api.create(&PostParams::default(), &lease).await.is_ok()
        }
        Err(_) => false,
    }
}

/// Spawn the background election loop. The returned [`Election`] is shared with
/// the reconcile context; its `is_leader` flag tracks whether this replica
/// currently holds the `lease_name` Lease (in `namespace`).
pub fn spawn(client: Client, namespace: String, lease_name: String, election: Arc<Election>) {
    tokio::spawn(async move {
        let api: Api<Lease> = Api::namespaced(client, &namespace);
        loop {
            let leader = acquire_or_renew(&api, &lease_name, &election.identity).await;
            let was = election.is_leader.swap(leader, Ordering::Relaxed);
            if leader != was {
                if leader {
                    tracing::info!(identity = %election.identity, lease = %lease_name, "acquired leadership");
                } else {
                    tracing::warn!(identity = %election.identity, lease = %lease_name, "lost leadership");
                }
            }
            tokio::time::sleep(RENEW_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::may_acquire;

    #[test]
    fn unheld_lease_is_acquirable() {
        assert!(may_acquire(None, None, 15, "me", 1000));
    }

    #[test]
    fn own_lease_is_renewable_even_when_fresh() {
        assert!(may_acquire(Some("me"), Some(999), 15, "me", 1000));
    }

    #[test]
    fn other_holder_fresh_lease_blocks() {
        assert!(!may_acquire(Some("other"), Some(999), 15, "me", 1000));
    }

    #[test]
    fn other_holder_expired_lease_is_taken_over() {
        assert!(may_acquire(Some("other"), Some(980), 15, "me", 1000));
        // exactly at the boundary (== duration) is NOT yet expired.
        assert!(!may_acquire(Some("other"), Some(985), 15, "me", 1000));
    }

    #[test]
    fn other_holder_missing_renew_time_is_acquirable() {
        assert!(may_acquire(Some("other"), None, 15, "me", 1000));
    }
}
