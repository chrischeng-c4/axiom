// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:baf16980" tracker="pending-tracker" reason="Work-queue competing-consumer delivery: lease / ack / redeliver and committed offset (standard at-least-once lease / retry semantics)."
//! Work-queue / competing-consumer delivery over a single log.
//!
//! Each entry is leased to exactly one consumer until it acks or the lease
//! expires; an expired lease makes the entry redelivery-eligible (with the
//! attempt count carried forward — standard at-least-once retry semantics).
//! The committed offset is the highest contiguous acked seq.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use crate::types::{CommittedOffset, Lease, Seq, ShardId};

/// Per-`(subject, shard)` competing-consumer delivery state.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
pub struct WorkQueue {
    subject: String,
    shard: ShardId,
    lease_ttl_ms: u64,
    max_attempts: u32,
    leases: HashMap<Seq, Lease>,
    lease_index: HashMap<String, Seq>,
    acked: HashSet<Seq>,
    attempts: HashMap<Seq, u32>,
}

impl WorkQueue {
    pub fn new(subject: &str, shard: ShardId, lease_ttl_ms: u64, max_attempts: u32) -> Self {
        WorkQueue {
            subject: subject.to_string(),
            shard,
            lease_ttl_ms,
            max_attempts,
            leases: HashMap::new(),
            lease_index: HashMap::new(),
            acked: HashSet::new(),
            attempts: HashMap::new(),
        }
    }

    /// The configured cap on delivery attempts before an entry is considered
    /// exhausted (revocation / dead-letter is the caller's policy).
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Lease the next eligible entry (smallest seq that is neither acked nor
    /// currently leased) to `consumer_id`. Returns `None` when nothing is
    /// available up to `log_len`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn lease(&mut self, consumer_id: &str, log_len: Seq, now: DateTime<Utc>) -> Option<Lease> {
        let mut seq = 0u64;
        while seq < log_len {
            if !self.acked.contains(&seq) && !self.leases.contains_key(&seq) {
                break;
            }
            seq += 1;
        }
        if seq >= log_len {
            return None;
        }

        let attempt = self.attempts.get(&seq).copied().unwrap_or(0) + 1;
        self.attempts.insert(seq, attempt);

        let lease_id = format!("{}#{}:{}:a{}", self.subject, self.shard, seq, attempt);
        let lease = Lease {
            lease_id: lease_id.clone(),
            seq,
            subject: self.subject.clone(),
            shard: self.shard,
            consumer_id: consumer_id.to_string(),
            granted_at: now,
            expires_at: now + Duration::milliseconds(self.lease_ttl_ms as i64),
            attempt,
        };
        self.leases.insert(seq, lease.clone());
        self.lease_index.insert(lease_id, seq);
        Some(lease)
    }

    /// Acknowledge a lease, marking its entry delivered. Returns `false` if the
    /// lease is unknown (already acked, expired, or never issued).
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn ack(&mut self, lease_id: &str) -> bool {
        if let Some(seq) = self.lease_index.remove(lease_id) {
            self.leases.remove(&seq);
            self.acked.insert(seq);
            true
        } else {
            false
        }
    }

    /// Reclaim every lease whose expiry is at or before `now`, making those
    /// entries redelivery-eligible. Returns the reclaimed seqs in order.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn reclaim_expired(&mut self, now: DateTime<Utc>) -> Vec<Seq> {
        let mut expired: Vec<Seq> = self
            .leases
            .iter()
            .filter(|(_, lease)| lease.expires_at <= now)
            .map(|(&seq, _)| seq)
            .collect();
        expired.sort_unstable();
        for seq in &expired {
            if let Some(lease) = self.leases.remove(seq) {
                self.lease_index.remove(&lease.lease_id);
            }
        }
        expired
    }

    /// Highest seq such that every entry `0..=committed_seq` has been acked, or
    /// `None` when entry `0` has not been acked yet.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&self) -> Option<CommittedOffset> {
        let mut c = 0u64;
        while self.acked.contains(&c) {
            c += 1;
        }
        if c == 0 {
            None
        } else {
            Some(CommittedOffset {
                subject: self.subject.clone(),
                shard: self.shard,
                committed_seq: c - 1,
            })
        }
    }
}
// HANDWRITE-END
