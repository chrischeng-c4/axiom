// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:baf16980" tracker="pending-tracker" reason="Work-queue competing-consumer delivery: lease / ack / redeliver and committed offset (standard at-least-once lease / retry semantics)."
//! Work-queue / competing-consumer delivery over a single log.
//!
//! Each entry is leased to exactly one consumer until it acks or the lease
//! expires; an expired lease makes the entry redelivery-eligible (with the
//! attempt count carried forward — standard at-least-once retry semantics).
//! The committed offset is the highest contiguous acked seq.
//!
//! Picking the next entry is O(1) (#128): a `next_offer` cursor hands out
//! never-offered seqs in order, and a redeliver min-heap re-offers reclaimed
//! seqs first. The committed watermark advances incrementally on ack.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

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
    /// Next never-offered seq (O(1) fresh pick).
    next_offer: Seq,
    /// Reclaimed seqs to re-offer first (smallest-first); preserves prefer-redeliver.
    redeliver: BinaryHeap<Reverse<Seq>>,
    /// Contiguous-acked watermark: every seq `< committed` has been acked.
    committed: Seq,
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
            next_offer: 0,
            redeliver: BinaryHeap::new(),
            committed: 0,
        }
    }

    /// The configured cap on delivery attempts before an entry is considered
    /// exhausted (revocation / dead-letter is the caller's policy).
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// O(1) next entry to offer: pop the redeliver min-heap first (prefer
    /// redeliver), otherwise take `next_offer` if the log has it.
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    fn pick(&mut self, log_len: Seq) -> Option<Seq> {
        while let Some(&Reverse(seq)) = self.redeliver.peek() {
            self.redeliver.pop();
            // Skip a stale heap entry that was meanwhile acked or re-leased.
            if !self.acked.contains(&seq) && !self.leases.contains_key(&seq) {
                return Some(seq);
            }
        }
        if self.next_offer < log_len {
            let seq = self.next_offer;
            self.next_offer += 1;
            return Some(seq);
        }
        None
    }

    /// Lease the next entry to `consumer_id` (preferring redelivery). The grant
    /// carries a monotonic `epoch` (the attempt number) used to fence stale
    /// workers on ack / heartbeat. Returns `None` when nothing is ready.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn lease(&mut self, consumer_id: &str, log_len: Seq, now: DateTime<Utc>) -> Option<Lease> {
        let seq = self.pick(log_len)?;

        let attempt = self.attempts.get(&seq).copied().unwrap_or(0) + 1;
        self.attempts.insert(seq, attempt);
        let epoch = attempt as u64;

        let lease_id = format!("{}#{}:{}:e{}", self.subject, self.shard, seq, epoch);
        let lease = Lease {
            lease_id: lease_id.clone(),
            seq,
            subject: self.subject.clone(),
            shard: self.shard,
            consumer_id: consumer_id.to_string(),
            granted_at: now,
            expires_at: now + Duration::milliseconds(self.lease_ttl_ms as i64),
            attempt,
            epoch,
        };
        self.leases.insert(seq, lease.clone());
        self.lease_index.insert(lease_id, seq);
        Some(lease)
    }

    /// Lease up to `max` entries in one call (amortizes a worker's round-trips).
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    pub fn lease_batch(
        &mut self,
        consumer_id: &str,
        log_len: Seq,
        max: usize,
        now: DateTime<Utc>,
    ) -> Vec<Lease> {
        let mut out = Vec::with_capacity(max.min(64));
        for _ in 0..max {
            match self.lease(consumer_id, log_len, now) {
                Some(l) => out.push(l),
                None => break,
            }
        }
        out
    }

    /// Acknowledge a lease, marking its entry delivered. Idempotent and fenced:
    /// returns `false` (no-op) when the `lease_id` is unknown or, if `epoch` is
    /// given, when it does not match the live lease. Passing `None` for `epoch`
    /// falls back to lease_id-only fencing.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn ack(&mut self, lease_id: &str, epoch: Option<u64>) -> bool {
        let Some(&seq) = self.lease_index.get(lease_id) else {
            return false;
        };
        if let (Some(want), Some(lease)) = (epoch, self.leases.get(&seq)) {
            if lease.epoch != want {
                return false;
            }
        }
        self.lease_index.remove(lease_id);
        self.leases.remove(&seq);
        self.acked.insert(seq);
        // Advance the contiguous-acked watermark (amortized O(1)).
        while self.acked.contains(&self.committed) {
            self.committed += 1;
        }
        true
    }

    /// Acknowledge many leases in one call; returns how many were accepted.
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    pub fn ack_batch(&mut self, acks: &[(String, Option<u64>)]) -> usize {
        let mut n = 0;
        for (lease_id, epoch) in acks {
            if self.ack(lease_id, *epoch) {
                n += 1;
            }
        }
        n
    }

    /// Extend a held lease if `lease_id` is known and `epoch` matches the live
    /// lease; returns the new expiry, or `None` when fenced / unknown.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn heartbeat(
        &mut self,
        lease_id: &str,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let &seq = self.lease_index.get(lease_id)?;
        let lease = self.leases.get_mut(&seq)?;
        if lease.epoch != epoch {
            return None;
        }
        lease.expires_at = now + Duration::milliseconds(self.lease_ttl_ms as i64);
        Some(lease.expires_at)
    }

    /// Reclaim every lease whose expiry is at or before `now`, pushing those
    /// seqs onto the redeliver heap so the next lease re-offers them first.
    /// Returns the reclaimed seqs in order.
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
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
                self.redeliver.push(Reverse(*seq));
            }
        }
        expired
    }

    /// Highest seq such that every entry `0..=committed_seq` has been acked, or
    /// `None` when entry `0` has not been acked yet.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&self) -> Option<CommittedOffset> {
        if self.committed == 0 {
            None
        } else {
            Some(CommittedOffset {
                subject: self.subject.clone(),
                shard: self.shard,
                committed_seq: self.committed - 1,
            })
        }
    }

    /// The committed watermark: the count of contiguous acked entries from 0
    /// (so `committed_seq = watermark - 1`). Persisted for crash recovery.
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn committed_watermark(&self) -> Seq {
        self.committed
    }

    /// Recover on open from a persisted watermark: entries `< watermark` are
    /// treated as committed and are never re-offered; uncommitted entries
    /// redeliver (at-least-once).
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn recover(&mut self, watermark: Seq) {
        self.committed = watermark;
        self.next_offer = watermark;
    }
}
// HANDWRITE-END
