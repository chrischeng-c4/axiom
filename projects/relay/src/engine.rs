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

use std::collections::{BTreeMap, HashMap};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use chrono::{DateTime, Utc};

use crate::broadcast::BroadcastDelivery;
use crate::config::RelayCoreConfig;
use crate::log::Log;
use crate::shard::shard_for;
use crate::types::{
    AppendOutcome, CommittedOffset, DeliveryModel, Lease, LogEntry, Payload, Seq, ShardId,
};
use crate::workqueue::WorkQueue;
use tokio::sync::watch;

struct SubjectState {
    log: Log,
    broadcast: BroadcastDelivery,
    workqueue: WorkQueue,
    model: DeliveryModel,
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
            lease_cursor: AtomicU64::new(0),
        }
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
        let log = Log::open(&self.config, subject, shard)?;
        let mut workqueue = WorkQueue::new(
            subject,
            shard,
            self.config.work_queue.lease_ttl_ms,
            self.config.work_queue.max_attempts,
        );
        if let Some(watermark) = log.load_commit() {
            workqueue.recover(watermark);
        }
        let ss = Arc::new(Mutex::new(SubjectState {
            log,
            broadcast: BroadcastDelivery::new(),
            workqueue,
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
        let shard = self.route(message_id);
        let ss = self.shard_state(subject, shard)?;
        let outcome = {
            let mut g = ss.lock().expect("subject mutex");
            g.log.append(message_id, payload, headers, now)?
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
                g.log.append_many(msgs, now)?
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
    /// whole subject drains across shards.
    ///
    /// @spec projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#logic
    pub fn lease(
        &self,
        subject: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        let start = (self.lease_cursor.fetch_add(1, Ordering::Relaxed) % self.shards as u64) as u32;
        for off in 0..self.shards {
            let shard = (start + off) % self.shards;
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            let len = g.log.len();
            if let Some(l) = g.workqueue.lease(consumer_id, len, now) {
                return Ok(Some(l));
            }
        }
        Ok(None)
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
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            let len = g.log.len();
            out.extend(
                g.workqueue
                    .lease_batch(consumer_id, len, max - out.len(), now),
            );
        }
        Ok(out)
    }

    /// Acknowledge a lease (epoch-fenced); routed by scanning shards for the lease_id.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn ack(&self, subject: &str, lease_id: &str, epoch: Option<u64>) -> io::Result<bool> {
        for shard in 0..self.shards {
            let ss = self.shard_state(subject, shard)?;
            let mut g = ss.lock().expect("subject mutex");
            if g.workqueue.ack(lease_id, epoch) {
                let wm = g.workqueue.committed_watermark();
                g.log.persist_commit(wm)?;
                return Ok(true);
            }
        }
        Ok(false)
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
            let n = g.workqueue.ack_batch(acks);
            if n > 0 {
                let wm = g.workqueue.committed_watermark();
                g.log.persist_commit(wm)?;
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
            if let Some(exp) = g.workqueue.heartbeat(lease_id, epoch, now) {
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
            out.extend(
                ss.lock()
                    .expect("subject mutex")
                    .workqueue
                    .reclaim_expired(now),
            );
        }
        Ok(out)
    }

    /// The committed work-queue offset for `subject` (shard 0; offsets are per-shard).
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&self, subject: &str) -> io::Result<Option<CommittedOffset>> {
        let ss = self.shard_state(subject, 0)?;
        let c = ss
            .lock()
            .expect("subject mutex")
            .workqueue
            .committed_offset();
        Ok(c)
    }

    /// Sweep every `(subject, shard)`'s expired leases (frontier-only); returns
    /// the number reclaimed. Each shard is locked independently.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn reconcile(&self, now: DateTime<Utc>) -> usize {
        let states: Vec<Arc<Mutex<SubjectState>>> = {
            self.subjects
                .read()
                .expect("subjects rwlock")
                .values()
                .cloned()
                .collect()
        };
        states
            .iter()
            .map(|s| {
                s.lock()
                    .expect("subject mutex")
                    .workqueue
                    .reclaim_expired(now)
                    .len()
            })
            .sum()
    }
}
// HANDWRITE-END
