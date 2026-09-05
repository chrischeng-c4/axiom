//! Product-neutral collector runtime.
//!
//! A product provides sources, record decoding, and delivery. This crate owns
//! batching, retry, quarantine ordering, and checkpoint commit ordering.

use std::{
    fmt,
    fs::OpenOptions,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use storage_durable::{atomic_write, set_private_file_mode, FsyncPolicy};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CommitStats {
    pub accepted: u64,
    pub duplicates: u64,
    pub rejected: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourceProgress {
    pub start_offset: u64,
    pub final_offset: u64,
    pub lost_bytes: u64,
    pub lost_sources: u64,
}

pub trait CollectorRecord {
    type Cursor: Clone;

    fn cursor(&self) -> &Self::Cursor;
}

pub trait CollectorRejection {
    type Cursor: Clone;
    type Entry;

    fn into_parts(self) -> (Self::Entry, Self::Cursor);
}

pub enum ReadOutcome<R, Q> {
    Record(R),
    Rejection(Q),
    Pending,
    Exhausted,
}

pub trait CollectorSource {
    type Cursor: Clone;
    type Error: fmt::Display + Send + Sync + 'static;
    type Record: CollectorRecord<Cursor = Self::Cursor>;
    type Rejection: CollectorRejection<Cursor = Self::Cursor>;

    fn next_record(
        &mut self,
        max_bytes: usize,
    ) -> Result<ReadOutcome<Self::Record, Self::Rejection>, Self::Error>;
    fn commit(&mut self, cursors: &[Self::Cursor], stats: CommitStats) -> Result<(), Self::Error>;
    fn refresh(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn progress(&self) -> SourceProgress {
        SourceProgress::default()
    }
}

pub trait RecordDecoder<R> {
    type Item;
    type Rejection;

    fn decode(&self, record: R) -> Result<Self::Item, Self::Rejection>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeliveryReceipt {
    pub accepted: u64,
    pub duplicates: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct DeliveryFailure {
    retryable: bool,
    message: String,
}

impl DeliveryFailure {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            retryable: true,
            message: message.into(),
        }
    }

    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            retryable: false,
            message: message.into(),
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[async_trait]
pub trait BatchSink<T>: Send + Sync {
    async fn send(&self, records: &[T]) -> Result<DeliveryReceipt, DeliveryFailure>;
}

pub trait QuarantineSink<T> {
    type Error: fmt::Display + Send + Sync + 'static;

    fn append(&mut self, entries: &[T]) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl RetryPolicy {
    pub fn new(
        max_retries: usize,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Result<Self, ConfigError> {
        let policy = Self {
            max_retries,
            initial_backoff,
            max_backoff,
        };
        if initial_backoff.is_zero() {
            return Err(ConfigError::ZeroInitialBackoff);
        }
        if max_backoff < initial_backoff {
            return Err(ConfigError::InvalidMaxBackoff);
        }
        Ok(policy)
    }

    pub fn delay(&self, attempt: usize) -> Duration {
        let multiplier = 1_u32.checked_shl(attempt.min(6) as u32).unwrap_or(64);
        self.initial_backoff
            .saturating_mul(multiplier)
            .min(self.max_backoff)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub batch_size: usize,
    pub max_record_bytes: usize,
    pub retry: RetryPolicy,
    pub follow: bool,
    pub follow_poll_interval: Duration,
}

impl RuntimeConfig {
    pub fn validate(self) -> Result<Self, ConfigError> {
        if self.batch_size == 0 {
            return Err(ConfigError::ZeroBatchSize);
        }
        if self.max_record_bytes == 0 {
            return Err(ConfigError::ZeroRecordBytes);
        }
        if self.follow_poll_interval.is_zero() {
            return Err(ConfigError::ZeroFollowPoll);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ConfigError {
    #[error("collector batch size must be positive")]
    ZeroBatchSize,
    #[error("collector record byte limit must be positive")]
    ZeroRecordBytes,
    #[error("collector initial retry backoff must be positive")]
    ZeroInitialBackoff,
    #[error("collector maximum backoff must not be below its initial backoff")]
    InvalidMaxBackoff,
    #[error("collector follow poll interval must be positive")]
    ZeroFollowPoll,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RunReport {
    pub lines: u64,
    pub accepted: u64,
    pub duplicates: u64,
    pub rejected: u64,
    pub progress: SourceProgress,
}

/// Controls delivery only. Source polling still follows `RuntimeConfig::follow`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryRetryMode {
    /// Stop after `RetryPolicy::max_retries`, as the original API does.
    Bounded,
    /// Keep the same bounded batch on retryable failures, with capped backoff.
    /// Dropping the runtime future cancels delivery without committing its cursors.
    /// Permanent failures and invalid success receipts still stop the runtime.
    UntilCancelled,
}

pub async fn run_collector<S, D, B, Q>(
    source: &mut S,
    decoder: &D,
    sink: &B,
    quarantine: &mut Q,
    config: RuntimeConfig,
) -> Result<RunReport>
where
    S: CollectorSource + ?Sized,
    D: RecordDecoder<S::Record, Rejection = <S::Rejection as CollectorRejection>::Entry>,
    B: BatchSink<D::Item>,
    Q: QuarantineSink<<S::Rejection as CollectorRejection>::Entry>,
{
    run_collector_with_delivery_mode(
        source,
        decoder,
        sink,
        quarantine,
        config,
        DeliveryRetryMode::Bounded,
    )
    .await
}

pub async fn run_collector_with_delivery_mode<S, D, B, Q>(
    source: &mut S,
    decoder: &D,
    sink: &B,
    quarantine: &mut Q,
    config: RuntimeConfig,
    delivery_mode: DeliveryRetryMode,
) -> Result<RunReport>
where
    S: CollectorSource + ?Sized,
    D: RecordDecoder<S::Record, Rejection = <S::Rejection as CollectorRejection>::Entry>,
    B: BatchSink<D::Item>,
    Q: QuarantineSink<<S::Rejection as CollectorRejection>::Entry>,
{
    let config = config.validate()?;
    RetryPolicy::new(
        config.retry.max_retries,
        config.retry.initial_backoff,
        config.retry.max_backoff,
    )?;
    let mut report = RunReport {
        progress: source.progress(),
        ..RunReport::default()
    };

    loop {
        let mut records = Vec::with_capacity(config.batch_size);
        let mut rejections = Vec::new();
        let mut cursors = Vec::with_capacity(config.batch_size);
        let mut reached_end = false;

        for _ in 0..config.batch_size {
            match source
                .next_record(config.max_record_bytes)
                .map_err(|error| anyhow!("collector source read failed: {error}"))?
            {
                ReadOutcome::Record(record) => {
                    cursors.push(record.cursor().clone());
                    match decoder.decode(record) {
                        Ok(record) => records.push(record),
                        Err(rejection) => rejections.push(rejection),
                    }
                }
                ReadOutcome::Rejection(rejection) => {
                    let (entry, cursor) = rejection.into_parts();
                    rejections.push(entry);
                    cursors.push(cursor);
                }
                ReadOutcome::Pending | ReadOutcome::Exhausted => {
                    reached_end = true;
                    break;
                }
            }
        }

        if cursors.is_empty() {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
                source
                    .refresh()
                    .map_err(|error| anyhow!("collector source refresh failed: {error}"))?;
                continue;
            }
            break;
        }

        let delivered = deliver_with_retry(sink, &records, config.retry, delivery_mode).await?;
        quarantine
            .append(&rejections)
            .map_err(|error| anyhow!("collector quarantine append failed: {error}"))?;
        source
            .commit(
                &cursors,
                CommitStats {
                    accepted: delivered.accepted,
                    duplicates: delivered.duplicates,
                    rejected: rejections.len() as u64,
                },
            )
            .map_err(|error| anyhow!("collector checkpoint commit failed: {error}"))?;

        report.lines += cursors.len() as u64;
        report.accepted += delivered.accepted;
        report.duplicates += delivered.duplicates;
        report.rejected += rejections.len() as u64;
        report.progress = source.progress();

        if reached_end {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
                source
                    .refresh()
                    .map_err(|error| anyhow!("collector source refresh failed: {error}"))?;
            } else {
                break;
            }
        }
    }

    report.progress = source.progress();
    Ok(report)
}

async fn deliver_with_retry<B, T>(
    sink: &B,
    records: &[T],
    retry: RetryPolicy,
    delivery_mode: DeliveryRetryMode,
) -> Result<DeliveryReceipt>
where
    B: BatchSink<T>,
{
    if records.is_empty() {
        return Ok(DeliveryReceipt::default());
    }
    let mut attempt = 0_usize;
    loop {
        match sink.send(records).await {
            Ok(receipt) => {
                let covered = receipt
                    .accepted
                    .checked_add(receipt.duplicates)
                    .ok_or_else(|| anyhow!("collector delivery receipt counters overflowed"))?;
                let expected = u64::try_from(records.len())
                    .map_err(|_| anyhow!("collector batch size does not fit in u64"))?;
                if covered != expected {
                    return Err(anyhow!(
                        "collector delivery receipt covered {covered} of {expected} records \
                         (accepted={}, duplicates={}); refusing to commit source cursors",
                        receipt.accepted,
                        receipt.duplicates
                    ));
                }
                return Ok(receipt);
            }
            Err(error) if !error.is_retryable() => {
                return Err(anyhow!("collector delivery failed permanently: {error}"));
            }
            Err(error)
                if delivery_mode == DeliveryRetryMode::Bounded && attempt >= retry.max_retries =>
            {
                return Err(anyhow!(
                    "collector delivery exhausted after {} attempt(s): {error}",
                    retry.max_retries.saturating_add(1)
                ));
            }
            Err(_) => tokio::time::sleep(retry.delay(attempt)).await,
        }
        attempt = attempt.saturating_add(1);
    }
}

/// Append-only JSONL quarantine with an fsync before returning.
pub struct JsonlQuarantine<T> {
    path: PathBuf,
    marker: PhantomData<T>,
}

impl<T> JsonlQuarantine<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            marker: PhantomData,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<T: Serialize> QuarantineSink<T> for JsonlQuarantine<T> {
    type Error = anyhow::Error;

    fn append(&mut self, entries: &[T]) -> Result<()> {
        append_jsonl(&self.path, entries)
    }
}

pub fn append_jsonl<T: Serialize>(path: &Path, entries: &[T]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create quarantine directory {}", parent.display()))?;
        }
    }
    let mut bytes = Vec::new();
    for entry in entries {
        serde_json::to_writer(&mut bytes, entry)?;
        bytes.push(b'\n');
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open collector quarantine {}", path.display()))?;
    set_private_file_mode(path)?;
    file.write_all(&bytes)
        .with_context(|| format!("append collector quarantine {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("fsync collector quarantine {}", path.display()))
}

pub fn load_json_checkpoint<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("read collector checkpoint {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("decode collector checkpoint {}", path.display()))
        .map(Some)
}

pub fn save_json_checkpoint<T: Serialize>(path: &Path, checkpoint: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(checkpoint)?;
    atomic_write(path, &bytes, FsyncPolicy::Always)
        .with_context(|| format!("commit collector checkpoint {}", path.display()))?;
    set_private_file_mode(path)
}
