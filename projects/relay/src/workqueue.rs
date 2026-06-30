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
    /// Not-yet-visible entries (delayed / ETA / backoff), min-heap by visible-at
    /// millis. `promote_due` moves due ones onto `redeliver`.
    delayed: BinaryHeap<Reverse<(i64, Seq)>>,
    /// Seqs currently held back in `delayed` — `pick` skips these.
    delayed_set: HashSet<Seq>,
    /// Contiguous-acked watermark: every seq `< committed` has been acked.
    committed: Seq,
}

/// Classification of the next pickable seq (work-queue internal).
enum Pick {
    /// Ready to lease.
    Ready(Seq),
    /// Exhausted its `max_attempts` budget — must be dead-lettered, not leased.
    Dead(Seq),
    /// Nothing ready.
    Empty,
}

/// Outcome of a dead-letter-aware lease attempt. `Dead` carries the seq and its
/// delivered-attempt count so the engine can durably route it to the
/// dead-letter subject (a cross-subject op the engine owns) and then `discard`
/// it from this queue.
///
/// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
pub enum LeaseOrDead {
    Leased(Lease),
    Dead { seq: Seq, attempts: u32 },
    Empty,
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
            delayed: BinaryHeap::new(),
            delayed_set: HashSet::new(),
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
            // Skip a stale heap entry that was meanwhile acked, re-leased, or
            // (re)deferred into the delay index.
            if !self.acked.contains(&seq)
                && !self.leases.contains_key(&seq)
                && !self.delayed_set.contains(&seq)
            {
                return Some(seq);
            }
        }
        // Hand out fresh seqs in order, skipping any held back as not-yet-visible
        // (a skipped delayed seq returns only via `promote_due` -> `redeliver`).
        while self.next_offer < log_len {
            let seq = self.next_offer;
            self.next_offer += 1;
            if !self.delayed_set.contains(&seq) {
                return Some(seq);
            }
        }
        None
    }

    /// Hold `seq` back until `visible_at` (delayed / ETA / countdown / backoff).
    /// `pick` skips it until [`WorkQueue::promote_due`] releases it.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn register_delayed(&mut self, seq: Seq, visible_at: DateTime<Utc>) {
        if self.acked.contains(&seq) {
            return;
        }
        if self.delayed_set.insert(seq) {
            self.delayed.push(Reverse((visible_at.timestamp_millis(), seq)));
        }
    }

    /// Release every delayed entry whose visible-at is at or before `now` onto the
    /// redeliver heap (prefer-redeliver). Returns how many were promoted, so the
    /// caller can wake idle consumers.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn promote_due(&mut self, now: DateTime<Utc>) -> usize {
        let cutoff = now.timestamp_millis();
        let mut promoted = 0;
        while let Some(&Reverse((at, seq))) = self.delayed.peek() {
            if at > cutoff {
                break;
            }
            self.delayed.pop();
            if self.delayed_set.remove(&seq) && !self.acked.contains(&seq) {
                self.redeliver.push(Reverse(seq));
                promoted += 1;
            }
        }
        promoted
    }

    /// True when `seq` has exhausted its delivery budget and must be
    /// dead-lettered instead of re-leased. `max_attempts == 0` disables this
    /// (entries redeliver indefinitely). The check uses the *delivered* count, so
    /// the `max_attempts`-th delivery is the last actual attempt and the next
    /// pick is dead.
    fn is_dead(&self, seq: Seq) -> bool {
        self.max_attempts > 0 && self.attempts.get(&seq).copied().unwrap_or(0) >= self.max_attempts
    }

    /// Pick the next seq and classify it as ready-to-lease vs dead (exhausted).
    fn pick_classified(&mut self, log_len: Seq) -> Pick {
        match self.pick(log_len) {
            Some(seq) if self.is_dead(seq) => Pick::Dead(seq),
            Some(seq) => Pick::Ready(seq),
            None => Pick::Empty,
        }
    }

    /// Build and record a lease for `seq` (bumping its attempt/epoch).
    fn grant(&mut self, seq: Seq, consumer_id: &str, now: DateTime<Utc>) -> Lease {
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
        lease
    }

    /// Lease the next entry to `consumer_id` (preferring redelivery), ignoring
    /// the dead-letter cap. The grant carries a monotonic `epoch` (the attempt
    /// number) used to fence stale workers. Returns `None` when nothing is ready.
    /// Prefer [`WorkQueue::lease_or_dead`] on the engine path so exhausted entries
    /// are dead-lettered rather than re-offered forever.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn lease(&mut self, consumer_id: &str, log_len: Seq, now: DateTime<Utc>) -> Option<Lease> {
        self.promote_due(now);
        let seq = self.pick(log_len)?;
        Some(self.grant(seq, consumer_id, now))
    }

    /// Lease the next ready entry, or report the next entry that has exhausted
    /// `max_attempts` so the engine can dead-letter it. A `Dead` seq is removed
    /// from the pick rotation (popped) but NOT leased and NOT yet committed — the
    /// engine durably routes it to the dead-letter subject and then calls
    /// [`WorkQueue::discard`].
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn lease_or_dead(
        &mut self,
        consumer_id: &str,
        log_len: Seq,
        now: DateTime<Utc>,
    ) -> LeaseOrDead {
        self.promote_due(now);
        match self.pick_classified(log_len) {
            Pick::Ready(seq) => LeaseOrDead::Leased(self.grant(seq, consumer_id, now)),
            Pick::Dead(seq) => LeaseOrDead::Dead {
                seq,
                attempts: self.attempts.get(&seq).copied().unwrap_or(0),
            },
            Pick::Empty => LeaseOrDead::Empty,
        }
    }

    /// Drop a dead-lettered entry: remove its bookkeeping and advance the
    /// committed watermark (mirrors `ack`) so the source backlog reclaims it.
    /// Called by the engine AFTER the entry is durably on the dead-letter
    /// subject (hazard H2: DLQ-before-discard).
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn discard(&mut self, seq: Seq) {
        self.leases.remove(&seq);
        self.lease_index.retain(|_, s| *s != seq);
        self.attempts.remove(&seq);
        self.acked.insert(seq);
        while self.acked.contains(&self.committed) {
            self.committed += 1;
        }
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

    /// Release a held lease immediately (Nack): drop it and push its seq onto the
    /// redeliver heap so the next `lease` re-offers it at once — no TTL wait.
    /// Idempotent: returns `false` when `lease_id` is unknown. Unlike `ack` this
    /// does NOT commit the entry; the attempt count is preserved for retry caps.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn release(&mut self, lease_id: &str) -> bool {
        let Some(&seq) = self.lease_index.get(lease_id) else {
            return false;
        };
        self.lease_index.remove(lease_id);
        self.leases.remove(&seq);
        self.redeliver.push(Reverse(seq));
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
