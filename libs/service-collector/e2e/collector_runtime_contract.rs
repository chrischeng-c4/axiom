use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use service_collector::{
    run_collector, BatchSink, CollectorRecord, CollectorRejection, CollectorSource, CommitStats,
    DeliveryFailure, DeliveryReceipt, JsonlQuarantine, ReadOutcome, RecordDecoder, RetryPolicy,
    RuntimeConfig,
};

#[derive(Clone)]
struct Raw {
    value: &'static str,
    cursor: u64,
}

impl CollectorRecord for Raw {
    type Cursor = u64;

    fn cursor(&self) -> &Self::Cursor {
        &self.cursor
    }
}

struct Rejected {
    entry: String,
    cursor: u64,
}

impl CollectorRejection for Rejected {
    type Cursor = u64;
    type Entry = String;

    fn into_parts(self) -> (Self::Entry, Self::Cursor) {
        (self.entry, self.cursor)
    }
}

struct Source {
    outcomes: VecDeque<ReadOutcome<Raw, Rejected>>,
    commits: Arc<Mutex<Vec<(Vec<u64>, CommitStats)>>>,
}

impl CollectorSource for Source {
    type Cursor = u64;
    type Error = anyhow::Error;
    type Record = Raw;
    type Rejection = Rejected;

    fn next_record(
        &mut self,
        _max_bytes: usize,
    ) -> Result<ReadOutcome<Self::Record, Self::Rejection>, Self::Error> {
        Ok(self.outcomes.pop_front().unwrap_or(ReadOutcome::Exhausted))
    }

    fn commit(&mut self, cursors: &[Self::Cursor], stats: CommitStats) -> Result<(), Self::Error> {
        self.commits.lock().unwrap().push((cursors.to_vec(), stats));
        Ok(())
    }
}

struct Decoder;

impl RecordDecoder<Raw> for Decoder {
    type Item = String;
    type Rejection = String;

    fn decode(&self, record: Raw) -> Result<Self::Item, Self::Rejection> {
        if record.value == "bad" {
            Err("invalid".into())
        } else {
            Ok(record.value.into())
        }
    }
}

struct Sink {
    attempts: Arc<Mutex<usize>>,
}

struct ShortReceiptSink;

#[async_trait::async_trait]
impl BatchSink<String> for ShortReceiptSink {
    async fn send(&self, _records: &[String]) -> Result<DeliveryReceipt, DeliveryFailure> {
        Ok(DeliveryReceipt {
            accepted: 1,
            duplicates: 0,
        })
    }
}

#[async_trait::async_trait]
impl BatchSink<String> for Sink {
    async fn send(&self, records: &[String]) -> Result<DeliveryReceipt, DeliveryFailure> {
        let mut attempts = self.attempts.lock().unwrap();
        *attempts += 1;
        if *attempts == 1 {
            return Err(DeliveryFailure::retryable("temporary"));
        }
        Ok(DeliveryReceipt {
            accepted: records.len() as u64,
            duplicates: 0,
        })
    }
}

#[tokio::test]
async fn generic_runtime_retries_quarantines_then_commits() {
    let commits = Arc::new(Mutex::new(Vec::new()));
    let attempts = Arc::new(Mutex::new(0));
    let mut source = Source {
        outcomes: VecDeque::from([
            ReadOutcome::Record(Raw {
                value: "good",
                cursor: 1,
            }),
            ReadOutcome::Record(Raw {
                value: "bad",
                cursor: 2,
            }),
            ReadOutcome::Exhausted,
        ]),
        commits: commits.clone(),
    };
    let temp = tempfile::tempdir().unwrap();
    let quarantine_path = temp.path().join("quarantine.jsonl");
    let mut quarantine = JsonlQuarantine::new(&quarantine_path);
    let report = run_collector(
        &mut source,
        &Decoder,
        &Sink {
            attempts: attempts.clone(),
        },
        &mut quarantine,
        RuntimeConfig {
            batch_size: 10,
            max_record_bytes: 1024,
            retry: RetryPolicy::new(1, Duration::from_millis(1), Duration::from_millis(1)).unwrap(),
            follow: false,
            follow_poll_interval: Duration::from_millis(1),
        },
    )
    .await
    .unwrap();

    assert_eq!(*attempts.lock().unwrap(), 2);
    assert_eq!(report.accepted, 1);
    assert_eq!(report.rejected, 1);
    assert_eq!(commits.lock().unwrap().len(), 1);
    assert_eq!(commits.lock().unwrap()[0].0, vec![1, 2]);
    assert_eq!(
        std::fs::read_to_string(quarantine_path).unwrap(),
        "\"invalid\"\n"
    );
}

#[tokio::test]
async fn incomplete_success_receipt_never_commits_source_cursors() {
    let commits = Arc::new(Mutex::new(Vec::new()));
    let mut source = Source {
        outcomes: VecDeque::from([
            ReadOutcome::Record(Raw {
                value: "first",
                cursor: 1,
            }),
            ReadOutcome::Record(Raw {
                value: "second",
                cursor: 2,
            }),
            ReadOutcome::Exhausted,
        ]),
        commits: commits.clone(),
    };
    let temp = tempfile::tempdir().unwrap();
    let mut quarantine = JsonlQuarantine::new(temp.path().join("quarantine.jsonl"));

    let error = run_collector(
        &mut source,
        &Decoder,
        &ShortReceiptSink,
        &mut quarantine,
        RuntimeConfig {
            batch_size: 10,
            max_record_bytes: 1024,
            retry: RetryPolicy::new(0, Duration::from_millis(1), Duration::from_millis(1)).unwrap(),
            follow: false,
            follow_poll_interval: Duration::from_millis(1),
        },
    )
    .await
    .expect_err("a short success receipt must fail closed");

    assert!(
        error
            .to_string()
            .contains("delivery receipt covered 1 of 2 records"),
        "unexpected error: {error:#}"
    );
    assert!(
        commits.lock().unwrap().is_empty(),
        "the source cursor must not advance after an incomplete receipt"
    );
}
