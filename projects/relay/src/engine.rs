// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:464901bc" tracker="pending-tracker" reason="Relay core engine tying publish -> classify -> broadcast / work-queue delivery over one durable log."
//! The in-process relay core: a durable ordered log per `(subject, shard)`, with
//! both broadcast fan-out and work-queue competing-consumer delivery over it.
//!
//! State is keyed by `(subject, shard)` and a subject is partitioned into
//! `default_shards` shards (#132): `publish` routes by `crc32(message_id) %
//! shards`, each shard has its own log / seq space / lock, so a hot subject
//! scales across cores. `lease` scans shards; `ack` / `heartbeat` route to the
//! owning shard; `subscribe` / `poll` span all shards. `default_shards = 1`
//! routes everything to shard 0 and is identical to a single-shard engine.
//!
//! Internally synchronized (per-shard `Mutex` behind an `RwLock` map, #128); all
//! methods take `&self`. Share it as `Arc<Relay>`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use chrono::{DateTime, Utc};

use crate::broadcast::BroadcastDelivery;
use crate::config::{RelayCoreConfig, RetentionMode};
use crate::log::Log;
use crate::shard::shard_for;
use crate::types::{
    AppendOutcome, CommittedOffset, DeliveryModel, Lease, LogEntry, Payload, Seq, ShardId,
};
use crate::workqueue::{LeaseOrDead, WorkQueue};
use tokio::sync::watch;

struct SubjectState {
    log: Log,
    broadcast: BroadcastDelivery,
    /// Named consumer groups (multicast): each is an independent competing-consumer
    /// work-queue over the shared `log`. The default group `""` is the single-queue
    /// path; each additional group receives every message once. GC truncates below
    /// the MIN committed watermark across all groups (a lagging group pins the head).
    groups: HashMap<String, WorkQueue>,
    // Parameters to lazily spawn a new group's work-queue.
    wq_subject: String,
    wq_shard: ShardId,
    lease_ttl_ms: u64,
    max_attempts: u32,
    redeliver_backoff_ms: u64,
    model: DeliveryModel,
}

impl SubjectState {
    /// Get or lazily create a consumer group's work-queue, initialized to see the
    /// currently-retained log tail. The default group `""` resumes from the
    /// persisted `.commit` watermark (back-compat single-queue durability); a
    /// named group starts at `start_seq` (the truncated prefix is treated as
    /// already-committed for it). Lazy creation is load-bearing: an unused group
    /// must not exist, else its committed=0 would pin delete-on-ack GC forever.
    fn group(&mut self, group: &str) -> io::Result<&mut WorkQueue> {
        if !self.groups.contains_key(group) {
            let mut wq = WorkQueue::new(
                &self.wq_subject,
                self.wq_shard,
                self.lease_ttl_ms,
                self.max_attempts,
                self.redeliver_backoff_ms,
            );
            let from = if group.is_empty() {
                let wm = self.log.load_commit();
                if let Some(wm) = wm {
                    wq.recover(wm);
                }
                wm.unwrap_or(0)
            } else {
                let start = self.log.start_seq();
                wq.recover(start);
                start
            };
            for e in self.log.range(from)? {
                match e.not_before {
                    Some(t) => wq.register_delayed(e.seq, t, e.priority),
                    None => wq.offer_fresh(e.seq, e.priority),
                }
            }
            self.groups.insert(group.to_string(), wq);
        }
        Ok(self.groups.get_mut(group).expect("group present"))
    }

    /// The minimum committed watermark across all groups — the safe-to-reclaim
    /// point for delete-on-ack truncation (one lagging group pins the head).
    fn min_committed(&self) -> Seq {
        self.groups
            .values()
            .map(|w| w.committed_watermark())
            .min()
            .unwrap_or(0)
    }
}

#[derive(Clone)]
struct SubjectWake {
    tx: watch::Sender<u64>,
    rev: Arc<AtomicU64>,
}

/// In-process broker core. Internally synchronized; share it as `Arc<Relay>`.
///
/// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
pub struct Relay {
    config: RelayCoreConfig,
    shards: u32,
    subjects: RwLock<HashMap<(String, ShardId), Arc<Mutex<SubjectState>>>>,
    subject_wakes: RwLock<HashMap<String, SubjectWake>>,
    /// Per-subject retention override (delete-on-ack vs age/size). Absent =
    /// `config.retention.mode`. Applies to every shard of the subject.
    subject_modes: RwLock<HashMap<String, RetentionMode>>,
    /// Rotating start shard for `lease`, to spread consumers across shards.
    lease_cursor: AtomicU64,
}

impl Relay {
    /// Build a core over `config`. Shards/subjects open lazily on first use.
    pub fn new(config: RelayCoreConfig) -> Self {
        let shards = config.default_shards.max(1);
        Relay {
            config,
            shards,
            subjects: RwLock::new(HashMap::new()),
            subject_wakes: RwLock::new(HashMap::new()),
            subject_modes: RwLock::new(HashMap::new()),
            lease_cursor: AtomicU64::new(0),
        }
    }

    /// Set a subject's retention mode (delete-on-ack vs age/size pruning),
    /// applying to every shard — already-open shards are updated in place and
    /// lazily-opened ones resolve it on open. Mark task-queue subjects `Ack` so
    /// their storage tracks backlog depth; leave broadcast/replay subjects `Age`.
    ///
    /// @spec projects/relay/tech-design/logic/log-segment-rotation-retention-full-log-lifecycle.md#logic
    pub fn set_retention_mode(&self, subject: &str, mode: RetentionMode) -> io::Result<()> {
        self.subject_modes
            .write()
            .expect("subject modes rwlock")
            .insert(subject.to_string(), mode);
        for shard in 0..self.shards {
            if let Some(ss) = self
                .subjects
                .read()
                .expect("subjects rwlock")
                .get(&(subject.to_string(), shard))
                .cloned()
            {
                ss.lock().expect("subject mutex").log.set_retention_mode(mode);
            }
        }
        Ok(())
    }

    /// Shard a routing key falls in: `crc32(key) % shards`.
    fn route(&self, key: &str) -> ShardId {
        shard_for(key, self.shards)
    }

    /// Resolve (and lazily open) the state for `(subject, shard)`.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    fn shard_state(&self, subject: &str, shard: ShardId) -> io::Result<Arc<Mutex<SubjectState>>> {
        let key = (subject.to_string(), shard);
        if let Some(s) = self.subjects.read().expect("subjects rwlock").get(&key) {
            return Ok(Arc::clone(s));
        }
        let mut map = self.subjects.write().expect("subjects rwlock");
        if let Some(s) = map.get(&key) {
            return Ok(Arc::clone(s));
        }
        let mut log = Log::open(&self.config, subject, shard)?;
        if let Some(&mode) = self
            .subject_modes
            .read()
            .expect("subject modes rwlock")
            .get(subject)
        {
            log.set_retention_mode(mode);
        }
        // Dead-letter subjects ({subject}{dlq_suffix}) open with max_attempts=0
        // so a consumer draining the DLQ never re-dead-letters into {subject}.dlq.dlq.
        let dlq_suffix = &self.config.work_queue.dlq_suffix;
        let max_attempts = if !dlq_suffix.is_empty() && subject.ends_with(dlq_suffix.as_str()) {
            0
        } else {
            self.config.work_queue.max_attempts
        };
        // Groups (incl. the default "") are created lazily on first use — each
        // rebuilds its ready set from the retained log tail at that point, so the
        // backlog (and delayed / ETA entries + priorities) survives restarts.
        let ss = Arc::new(Mutex::new(SubjectState {
            log,
            broadcast: BroadcastDelivery::new(),
            groups: HashMap::new(),
            wq_subject: subject.to_string(),
            wq_shard: shard,
            lease_ttl_ms: self.config.work_queue.lease_ttl_ms,
            max_attempts,
            redeliver_backoff_ms: self.config.work_queue.redeliver_backoff_ms,
            model: DeliveryModel::Broadcast,
        }));
        map.insert(key, Arc::clone(&ss));
        Ok(ss)
    }

    fn ensure_subject_wake(&self, subject: &str) -> SubjectWake {
        if let Some(wake) = self
            .subject_wakes
            .read()
            .expect("subject wake registry rwlock")
            .get(subject)
            .cloned()
        {
            return wake;
        }
        let mut wakes = self
            .subject_wakes
            .write()
            .expect("subject wake registry rwlock");
        wakes
            .entry(subject.to_string())
            .or_insert_with(|| {
                let (tx, _) = watch::channel(0);
                SubjectWake {
                    tx,
                    rev: Arc::new(AtomicU64::new(0)),
                }
            })
            .clone()
    }

    fn wake_subscribers(&self, subject: &str) {
        let Some(wake) = self
            .subject_wakes
            .read()
            .expect("subject wake registry rwlock")
            .get(subject)
            .cloned()
        else {
            return;
        };
        let rev = wake.rev.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = wake.tx.send(rev);
    }

    pub fn subscribe_wake(&self, subject: &str) -> watch::Receiver<u64> {
        self.ensure_subject_wake(subject).tx.subscribe()
    }

    /// Publish a message; routed to `crc32(message_id) % shards`. Idempotent per id.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn publish(
        &self,
        subject: &str,
        message_id: &str,
        payload: Payload,
        headers: BTreeMap<String, String>,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        self.publish_at(subject, message_id, payload, headers, None, 0, now)
    }

    /// Publish with an optional `not_before` work-queue visibility gate (delayed /
    /// ETA / countdown delivery). The entry is durably appended immediately (and
    /// is visible to broadcast subscribers at once), but is not leasable by a
    /// work-queue consumer until `not_before`. Idempotent per id.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn publish_at(
        &self,
        subject: &str,
        message_id: &str,
        payload: Payload,
        headers: BTreeMap<String, String>,
        not_before: Option<DateTime<Utc>>,
        priority: u8,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        let shard = self.route(message_id);
        let ss = self.shard_state(subject, shard)?;
        let outcome = {
            let mut g = ss.lock().expect("subject mutex");
            let outcome = g
                .log
                .append_at(message_id, payload, headers, not_before, priority, now)?;
            // Offer the entry into EVERY group's priority band (multicast: each
            // group gets every message once) — held in the delay index if
            // future-dated (broadcast still sees it immediately — it is in the log).
            if !outcome.deduped {
                for wq in g.groups.values_mut() {
                    match not_before {
                        Some(t) if t > now => wq.register_delayed(outcome.seq, t, priority),
                        _ => wq.offer_fresh(outcome.seq, priority),
                    }
                }
            }
            outcome
        };
        if !outcome.deduped {
            self.wake_subscribers(subject);
        }
        Ok(outcome)
    }

    /// Publish a batch (group commit); each message routes to its shard, one
    /// group-commit fsync per touched shard. Outcomes are returned in input order.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn publish_batch(
        &self,
        subject: &str,
        messages: Vec<(String, Payload, BTreeMap<String, String>)>,
        now: DateTime<Utc>,
    ) -> io::Result<Vec<AppendOutcome>> {
        // Partition by shard, preserving the original index.
        let mut buckets: HashMap<
            ShardId,
            Vec<(usize, (String, Payload, BTreeMap<String, String>))>,
        > = HashMap::new();
        for (i, msg) in messages.into_iter().enumerate() {
            let shard = self.route(&msg.0);
            buckets.entry(shard).or_default().push((i, msg));
        }
        let mut out: Vec<Option<AppendOutcome>> = (0..buckets.values().map(|v| v.len()).sum())
            .map(|_| None)
            .collect();
        for (shard, items) in buckets {
            let (idxs, msgs): (Vec<usize>, Vec<_>) = items.into_iter().unzip();
            let ss = self.shard_state(subject, shard)?;
            let outcomes = {
                let mut g = ss.lock().expect("subject mutex");
                let outcomes = g.log.append_many(msgs, now)?;
                // Offer each new entry into every group (batch = priority 0,
                // immediate). Without this the entries would never be leasable.
                for oc in &outcomes {
                    if !oc.deduped {
                        for wq in g.groups.values_mut() {
                            wq.offer_fresh(oc.seq, 0);
                        }
                    }
                }
                outcomes
            };
            for (idx, oc) in idxs.into_iter().zip(outcomes) {
                out[idx] = Some(oc);
            }
        }
        let out: Vec<AppendOutcome> = out
            .into_iter()
            .map(|o| o.expect("every index filled"))
            .collect();
        if out.iter().any(|outcome| !outcome.deduped) {
            self.wake_subscribers(subject);
        }
        Ok(out)
    }

    /// Set the descriptive delivery model for a subject (recorded on shard 0).
    pub fn set_delivery_model(&self, subject: &str, model: DeliveryModel) -> io::Result<()> {
        let ss = self.shard_state(subject, 0)?;
        ss.lock().expect("subject mutex").model = model;
        Ok(())
    }

    /// The recorded delivery model for a subject's shard 0, if it exists.
    pub fn delivery_model(&self, subject: &str) -> Option<DeliveryModel> {
        self.subjects
            .read()
            .expect("subjects rwlock")
            .get(&(subject.to_string(), 0))
            .map(|s| s.lock().expect("subject mutex").model)
    }

    /// Total entries across all of `subject`'s shards.
    pub fn log_len(&self, subject: &str) -> io::Result<Seq> {
        let mut total = 0;
        for shard in 0..self.shards {
            total += self
                .shard_state(subject, shard)?
                .lock()
                .expect("subject mutex")
                .log
                .len();
        }
        Ok(total)
    }

    /// Register a broadcast subscriber on every shard, starting at `from_seq`.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn subscribe(&self, subject: &str, subscriber_id: &str, from_seq: Seq) -> io::Result<()> {
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            ss.lock()
                .expect("subject mutex")
                .broadcast
                .subscribe(subscriber_id, from_seq);
        }
        Ok(())
    }

    /// Deliver not-yet-delivered entries to a broadcast subscriber, merged across
    /// shards (per-shard order preserved).
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn poll(&self, subject: &str, subscriber_id: &str) -> io::Result<Vec<LogEntry>> {
        let mut out = Vec::new();
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            let g = &mut *g;
            out.extend(g.broadcast.poll(subscriber_id, &g.log)?);
        }
        Ok(out)
    }

    /// Lease the next ready entry, scanning shards from a rotating start so the
    /// whole subject drains across shards. Entries that have exhausted
    /// `max_attempts` are dead-lettered (durably routed to `{subject}{dlq_suffix}`)
    /// and skipped, never re-offered.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn lease(
        &self,
        subject: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        self.lease_in(subject, "", consumer_id, now)
    }

    /// Lease the next ready entry for a named consumer group (multicast). Each
    /// group competes within itself and independently receives every message;
    /// the default group `""` is the single-queue path.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn lease_in(
        &self,
        subject: &str,
        group: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        let start = (self.lease_cursor.fetch_add(1, Ordering::Relaxed) % self.shards as u64) as u32;
        for off in 0..self.shards {
            let shard = (start + off) % self.shards;
            if let Some(l) = self.lease_one(subject, shard, group, consumer_id, now)? {
                return Ok(Some(l));
            }
        }
        Ok(None)
    }

    /// Lease one entry from a single shard, dead-lettering any exhausted entries
    /// encountered first. Returns `Ok(None)` when the shard's queue is drained.
    ///
    /// Dead-lettering crosses subjects (publish to `{subject}{dlq_suffix}`), so it
    /// runs WITHOUT holding the source shard lock — and the DLQ append is made
    /// durable BEFORE the source entry is discarded (hazard H2), so a crash
    /// mid-way at worst re-delivers (at-least-once), never loses, the task.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    fn lease_one(
        &self,
        subject: &str,
        shard: ShardId,
        group: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        loop {
            // Phase A: under the source lock, lease or detect a dead entry and
            // read its body (the dead seq is popped out of the pick rotation).
            let (dead_seq, dead_entry, attempts) = {
                let ss = self.shard_state(subject, shard)?;
                let mut g = ss.lock().expect("subject mutex");
                match g.group(group)?.lease_or_dead(consumer_id, now) {
                    LeaseOrDead::Leased(l) => return Ok(Some(l)),
                    LeaseOrDead::Empty => return Ok(None),
                    LeaseOrDead::Dead { seq, attempts } => (seq, g.log.entry(seq)?, attempts),
                }
            };
            // Phase B (no source lock held): durably route to the DLQ, then
            // discard from the source so its watermark/storage advances.
            if let Some(entry) = dead_entry {
                let dlq = format!("{subject}{}", self.config.work_queue.dlq_suffix);
                let mut headers = entry.headers.clone();
                headers.insert("x-relay-dlq-reason".to_string(), "max-attempts".to_string());
                headers.insert("x-relay-dlq-attempts".to_string(), attempts.to_string());
                headers.insert("x-relay-origin-subject".to_string(), subject.to_string());
                headers.insert("x-relay-origin-seq".to_string(), entry.seq.to_string());
                let dlq_id = format!("{}:dlq", entry.message_id);
                self.publish(&dlq, &dlq_id, entry.payload, headers, now)?;
            }
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            g.group(group)?.discard(dead_seq);
            self.persist_and_truncate(&mut g)?;
            // loop: re-pick from this shard (next entry may be ready or dead).
        }
    }

    /// Read a leased entry's stored body (`message_id` + `payload` + `headers`)
    /// by its `(subject, shard, seq)`, so a work-queue consumer knows what it
    /// leased and can fetch claim-check input / dispatch on the task (#166).
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn entry(&self, subject: &str, shard: ShardId, seq: Seq) -> io::Result<Option<LogEntry>> {
        let ss = self.shard_state(subject, shard)?;
        let g = ss.lock().expect("subject mutex");
        g.log.entry(seq)
    }

    /// Lease up to `max` entries, accumulating across shards.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn lease_batch(
        &self,
        subject: &str,
        consumer_id: &str,
        max: usize,
        now: DateTime<Utc>,
    ) -> io::Result<Vec<Lease>> {
        let start = (self.lease_cursor.fetch_add(1, Ordering::Relaxed) % self.shards as u64) as u32;
        let mut out = Vec::new();
        for off in 0..self.shards {
            if out.len() >= max {
                break;
            }
            let shard = (start + off) % self.shards;
            // Dead-letter-aware: drain this shard one entry at a time so exhausted
            // entries are routed to the DLQ rather than re-offered.
            while out.len() < max {
                match self.lease_one(subject, shard, "", consumer_id, now)? {
                    Some(l) => out.push(l),
                    None => break,
                }
            }
        }
        Ok(out)
    }

    /// Persist the default group's committed watermark (for recovery) and, in
    /// Ack-retention mode, truncate the log below the MIN committed watermark
    /// across ALL groups — a lagging group pins the head. H1: persist-before-truncate.
    fn persist_and_truncate(&self, g: &mut SubjectState) -> io::Result<()> {
        // Persist the default group's watermark for recovery — only if it exists,
        // so a named-group ack never clobbers the `.commit` file with 0.
        if let Some(default_wm) = g.groups.get("").map(|w| w.committed_watermark()) {
            g.log.persist_commit(default_wm)?;
        }
        if g.log.retention_mode() == RetentionMode::Ack {
            let min_wm = g.min_committed();
            g.log.truncate_below_acked(min_wm)?;
        }
        Ok(())
    }

    /// Acknowledge a lease (epoch-fenced) on the default group.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn ack(&self, subject: &str, lease_id: &str, epoch: Option<u64>) -> io::Result<bool> {
        self.ack_in(subject, "", lease_id, epoch)
    }

    /// Acknowledge a lease on a named consumer group; routed by scanning shards.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn ack_in(
        &self,
        subject: &str,
        group: &str,
        lease_id: &str,
        epoch: Option<u64>,
    ) -> io::Result<bool> {
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            if g.group(group)?.ack(lease_id, epoch) {
                self.persist_and_truncate(&mut g)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Release (Nack) a held lease for immediate redelivery on the default group.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn release(&self, subject: &str, lease_id: &str) -> io::Result<bool> {
        self.release_in(subject, "", lease_id)
    }

    /// Release (Nack) a held lease on a named consumer group; routed by scanning
    /// shards. Wakes the subject's consumers so an idle `/consume` stream re-leases
    /// the entry at once instead of waiting for TTL.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn release_in(&self, subject: &str, group: &str, lease_id: &str) -> io::Result<bool> {
        let mut released = false;
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            if g.group(group)?.release(lease_id) {
                released = true;
                break;
            }
        }
        if released {
            self.wake_subscribers(subject);
        }
        Ok(released)
    }

    /// Acknowledge many leases; each shard acks the ones it owns. Returns
    /// (accepted count, committed offset of shard 0).
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    pub fn ack_batch(
        &self,
        subject: &str,
        acks: &[(String, Option<u64>)],
    ) -> io::Result<(usize, Option<CommittedOffset>)> {
        let mut total = 0;
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            let n = g.group("")?.ack_batch(acks);
            if n > 0 {
                self.persist_and_truncate(&mut g)?;
            }
            total += n;
        }
        let committed = self.committed_offset(subject)?;
        Ok((total, committed))
    }

    /// Extend a held lease (heartbeat); routed by scanning shards for the lease_id.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn heartbeat(
        &self,
        subject: &str,
        lease_id: &str,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> io::Result<Option<DateTime<Utc>>> {
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            if let Some(exp) = g.group("")?.heartbeat(lease_id, epoch, now) {
                return Ok(Some(exp));
            }
        }
        Ok(None)
    }

    /// Reclaim expired leases on `subject` (all shards); returns reclaimed seqs.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn reclaim_expired(&self, subject: &str, now: DateTime<Utc>) -> io::Result<Vec<Seq>> {
        let mut out = Vec::new();
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            out.extend(g.group("")?.reclaim_expired(now));
        }
        Ok(out)
    }

    /// The committed work-queue offset for `subject` (shard 0; offsets are per-shard).
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&self, subject: &str) -> io::Result<Option<CommittedOffset>> {
        let ss = self.shard_state(subject, 0)?;
        let mut g = ss.lock().expect("subject mutex");
        Ok(g.group("")?.committed_offset())
    }

    /// Sweep every `(subject, shard)`'s expired leases (frontier-only); returns
    /// the number reclaimed. Each shard is locked independently.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn reconcile(&self, now: DateTime<Utc>) -> usize {
        let states: Vec<(String, Arc<Mutex<SubjectState>>)> = {
            self.subjects
                .read()
                .expect("subjects rwlock")
                .iter()
                .map(|((subject, _shard), s)| (subject.clone(), Arc::clone(s)))
                .collect()
        };
        let mut total = 0;
        // Subjects with ≥1 reclaimed lease: wake their consumers so a redelivered
        // entry is re-leased immediately (matches `release`; #465 wake-based push).
        let mut woken: HashSet<String> = HashSet::new();
        for (subject, s) in &states {
            let mut g = s.lock().expect("subject mutex");
            // Sweep every group: reclaim expired leases + promote due delays.
            let (mut n, mut promoted) = (0usize, 0usize);
            for wq in g.groups.values_mut() {
                n += wq.reclaim_expired(now).len();
                promoted += wq.promote_due(now);
            }
            total += n;
            if n > 0 || promoted > 0 {
                woken.insert(subject.clone());
            }
        }
        for subject in woken {
            self.wake_subscribers(&subject);
        }
        total
    }
}
// HANDWRITE-END
