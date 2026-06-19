// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:464901bc" tracker="pending-tracker" reason="Relay core engine tying publish -> classify -> broadcast / work-queue delivery over one durable log."
//! The in-process relay core: one durable ordered log per subject, with both
//! broadcast fan-out and work-queue competing-consumer delivery reading from
//! that same log.
//!
//! Sequencing is per `(subject, shard)`; this core implements a single shard
//! (`0`) per subject. The engine is internally synchronized with **per-shard
//! locking** (#128): each subject's state sits behind its own `Mutex`, so
//! operations on different subjects run concurrently; only a brief `RwLock`
//! guards subject lookup/creation. All methods take `&self`.

use std::collections::{BTreeMap, HashMap};
use std::io;
use std::sync::{Arc, Mutex, RwLock};

use chrono::{DateTime, Utc};

use crate::broadcast::BroadcastDelivery;
use crate::config::RelayCoreConfig;
use crate::log::Log;
use crate::types::{
    AppendOutcome, CommittedOffset, DeliveryModel, Lease, LogEntry, Payload, Seq, ShardId,
};
use crate::workqueue::WorkQueue;

/// The single shard this core implements per subject.
const SHARD: ShardId = 0;

struct SubjectState {
    log: Log,
    broadcast: BroadcastDelivery,
    workqueue: WorkQueue,
    model: DeliveryModel,
}

/// In-process broker core. Internally synchronized (per-shard locking); share
/// it as `Arc<Relay>` across threads / tasks.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
pub struct Relay {
    config: RelayCoreConfig,
    subjects: RwLock<HashMap<String, Arc<Mutex<SubjectState>>>>,
}

impl Relay {
    /// Build a core over `config`. Subjects are opened lazily on first use.
    pub fn new(config: RelayCoreConfig) -> Self {
        Relay {
            config,
            subjects: RwLock::new(HashMap::new()),
        }
    }

    /// Resolve (and lazily open) a subject's shard, returning its lock handle.
    /// Only the subject map is guarded here; callers lock the returned shard.
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    fn subject_state(&self, subject: &str) -> io::Result<Arc<Mutex<SubjectState>>> {
        if let Some(s) = self.subjects.read().expect("subjects rwlock").get(subject) {
            return Ok(Arc::clone(s));
        }
        let mut map = self.subjects.write().expect("subjects rwlock");
        if let Some(s) = map.get(subject) {
            return Ok(Arc::clone(s));
        }
        let log = Log::open(&self.config, subject, SHARD)?;
        let mut workqueue = WorkQueue::new(
            subject,
            SHARD,
            self.config.work_queue.lease_ttl_ms,
            self.config.work_queue.max_attempts,
        );
        // Crash recovery: resume after the durably committed watermark.
        if let Some(watermark) = log.load_commit() {
            workqueue.recover(watermark);
        }
        let ss = Arc::new(Mutex::new(SubjectState {
            log,
            broadcast: BroadcastDelivery::new(),
            workqueue,
            model: DeliveryModel::Broadcast,
        }));
        map.insert(subject.to_string(), Arc::clone(&ss));
        Ok(ss)
    }

    /// Publish a message to `subject`. Idempotent on `message_id`: a repeated
    /// id returns the existing seq with `deduped = true`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn publish(
        &self,
        subject: &str,
        message_id: &str,
        payload: Payload,
        headers: BTreeMap<String, String>,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        g.log.append(message_id, payload, headers, now)
    }

    /// Publish a batch of messages with a single group-commit fsync (the
    /// durable high-throughput produce path).
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn publish_batch(
        &self,
        subject: &str,
        messages: Vec<(String, Payload, BTreeMap<String, String>)>,
        now: DateTime<Utc>,
    ) -> io::Result<Vec<AppendOutcome>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        g.log.append_many(messages, now)
    }

    /// Set the descriptive delivery model for a subject.
    pub fn set_delivery_model(&self, subject: &str, model: DeliveryModel) -> io::Result<()> {
        let ss = self.subject_state(subject)?;
        ss.lock().expect("subject mutex").model = model;
        Ok(())
    }

    /// The recorded delivery model for a subject, if it exists.
    pub fn delivery_model(&self, subject: &str) -> Option<DeliveryModel> {
        self.subjects
            .read()
            .expect("subjects rwlock")
            .get(subject)
            .map(|s| s.lock().expect("subject mutex").model)
    }

    /// Number of entries appended to `subject`'s log.
    pub fn log_len(&self, subject: &str) -> io::Result<Seq> {
        Ok(self
            .subject_state(subject)?
            .lock()
            .expect("subject mutex")
            .log
            .len())
    }

    /// Register (or re-position) a broadcast subscriber to start at `from_seq`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn subscribe(&self, subject: &str, subscriber_id: &str, from_seq: Seq) -> io::Result<()> {
        let ss = self.subject_state(subject)?;
        ss.lock()
            .expect("subject mutex")
            .broadcast
            .subscribe(subscriber_id, from_seq);
        Ok(())
    }

    /// Deliver all not-yet-delivered entries to a broadcast subscriber, in
    /// order, advancing its cursor.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn poll(&self, subject: &str, subscriber_id: &str) -> io::Result<Vec<LogEntry>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        let g = &mut *g;
        g.broadcast.poll(subscriber_id, &g.log)
    }

    /// Lease the next eligible entry of `subject` to a competing consumer.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn lease(
        &self,
        subject: &str,
        consumer_id: &str,
        now: DateTime<Utc>,
    ) -> io::Result<Option<Lease>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        let len = g.log.len();
        Ok(g.workqueue.lease(consumer_id, len, now))
    }

    /// Lease up to `max` eligible entries in one call.
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    pub fn lease_batch(
        &self,
        subject: &str,
        consumer_id: &str,
        max: usize,
        now: DateTime<Utc>,
    ) -> io::Result<Vec<Lease>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        let len = g.log.len();
        Ok(g.workqueue.lease_batch(consumer_id, len, max, now))
    }

    /// Acknowledge a work-queue lease. Idempotent and epoch-fenced.
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn ack(&self, subject: &str, lease_id: &str, epoch: Option<u64>) -> io::Result<bool> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        let ok = g.workqueue.ack(lease_id, epoch);
        if ok {
            let wm = g.workqueue.committed_watermark();
            g.log.persist_commit(wm)?;
        }
        Ok(ok)
    }

    /// Acknowledge many leases in one call; returns (accepted count, committed offset).
    ///
    /// @spec projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#logic
    pub fn ack_batch(
        &self,
        subject: &str,
        acks: &[(String, Option<u64>)],
    ) -> io::Result<(usize, Option<CommittedOffset>)> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        let n = g.workqueue.ack_batch(acks);
        let committed = g.workqueue.committed_offset();
        if n > 0 {
            let wm = g.workqueue.committed_watermark();
            g.log.persist_commit(wm)?;
        }
        Ok((n, committed))
    }

    /// Extend a held lease (heartbeat).
    ///
    /// @spec projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#logic
    pub fn heartbeat(
        &self,
        subject: &str,
        lease_id: &str,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> io::Result<Option<DateTime<Utc>>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        Ok(g.workqueue.heartbeat(lease_id, epoch, now))
    }

    /// Reclaim expired leases on `subject`.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn reclaim_expired(&self, subject: &str, now: DateTime<Utc>) -> io::Result<Vec<Seq>> {
        let ss = self.subject_state(subject)?;
        let mut g = ss.lock().expect("subject mutex");
        Ok(g.workqueue.reclaim_expired(now))
    }

    /// The committed work-queue offset for `subject`.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn committed_offset(&self, subject: &str) -> io::Result<Option<CommittedOffset>> {
        let ss = self.subject_state(subject)?;
        let c = ss
            .lock()
            .expect("subject mutex")
            .workqueue
            .committed_offset();
        Ok(c)
    }

    /// Sweep every subject/shard's expired leases (frontier-only) and return the
    /// number reclaimed. Each shard is locked independently.
    ///
    /// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
    pub fn reconcile(&self, now: DateTime<Utc>) -> usize {
        let shards: Vec<Arc<Mutex<SubjectState>>> = {
            self.subjects
                .read()
                .expect("subjects rwlock")
                .values()
                .cloned()
                .collect()
        };
        shards
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
