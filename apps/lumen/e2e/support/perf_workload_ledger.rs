//! Standalone evidence ledger for the approved #4246 release workload.
//!
//! Behavior: Index items are fields, not documents. A document operation is
//! complete only after all of its required fields finish successfully, even
//! when fields span batches. Replace counts one document and unindex counts
//! one external ID.
//!
//! Security: event input is untrusted. Duplicate IDs, duplicate fields,
//! mixed endpoints, unknown IDs, and missing end times reject the ledger.
//!
//! Performance: the fixed plan is 30 minutes, 100 actual full-document body
//! starts/s over that complete window at >=95%, 10 combined hot/idle QPS,
//! p99 <=1s, max <=5s, zero errors/timeouts, drain <=60s, RSS <=12 GiB,
//! plus checkpoint and merge. The scheduler still creates the mixed 100-op
//! turn each second, while fixed 1,000-item builders may form bounded bursts.
//! This module validates evidence. It does not measure a benchmark.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

pub type RequestId = u64;
pub type OperationId = u64;

const REPLACE_ITEM: &str = "$document";
const UNINDEX_ITEM: &str = "$external_id";
const GIB: u64 = 1024 * 1024 * 1024;
/// `drive_workload` rotates add, replace, and delete turns in this fixed
/// order. The bound below comes from that existing three-second rotation and
/// the selected item batch capacity; it is not an endpoint throughput target.
const MUTATION_ROTATION_SECONDS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endpoint {
    Index,
    Replace,
    Unindex,
}

/// The two query populations in the fixed durable workload. Semantic
/// readbacks deliberately do not enter this evidence stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryClass {
    Hot,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    FlatCpu,
    HnswCpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct WorkloadCase {
    /// The endpoint/batch cell this run measures. The workload still drives
    /// all three mutation endpoints: add uses Index, update uses Replace, and
    /// delete uses Unindex. The non-primary endpoints use their largest
    /// approved batch so a primary cell cannot omit a mutation class.
    pub primary_endpoint: Endpoint,
    pub primary_batch_size: usize,
    pub backend: Backend,
}

impl WorkloadCase {
    pub fn new(primary_endpoint: Endpoint, primary_batch_size: usize, backend: Backend) -> Self {
        Self {
            primary_endpoint,
            primary_batch_size,
            backend,
        }
    }

    pub fn batch_size_for(&self, endpoint: Endpoint) -> usize {
        if endpoint == self.primary_endpoint {
            self.primary_batch_size
        } else {
            match endpoint {
                Endpoint::Index => 1_000,
                Endpoint::Replace => 32,
                Endpoint::Unindex => 1_000,
            }
        }
    }

    fn is_approved_matrix_cell(&self) -> bool {
        matches!(
            (self.primary_endpoint, self.primary_batch_size),
            (Endpoint::Index, 1 | 100 | 1_000)
                | (Endpoint::Replace, 1 | 32)
                | (Endpoint::Unindex, 1 | 100 | 1_000)
        )
    }
}

#[derive(Debug, Clone)]
pub struct ApprovedLimits {
    pub input_duration: Duration,
    pub docops_per_second: u64,
    pub query_qps: u64,
    pub completed_percent: u64,
    pub p99_limit: Duration,
    pub max_query_limit: Duration,
    pub drain_limit: Duration,
    pub rss_limit_bytes: u64,
}

impl ApprovedLimits {
    pub fn approved() -> Self {
        Self {
            input_duration: Duration::from_secs(30 * 60),
            docops_per_second: 100,
            query_qps: 10,
            completed_percent: 95,
            p99_limit: Duration::from_secs(1),
            max_query_limit: Duration::from_secs(5),
            drain_limit: Duration::from_secs(60),
            rss_limit_bytes: 12 * GIB,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentOperation {
    pub id: OperationId,
    pub endpoint: Endpoint,
    pub document_id: String,
    pub required_items: Vec<String>,
}

impl DocumentOperation {
    pub fn index(
        id: OperationId,
        document_id: impl Into<String>,
        required_fields: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            id,
            endpoint: Endpoint::Index,
            document_id: document_id.into(),
            required_items: required_fields.into_iter().collect(),
        }
    }

    pub fn replace(id: OperationId, document_id: impl Into<String>) -> Self {
        Self {
            id,
            endpoint: Endpoint::Replace,
            document_id: document_id.into(),
            required_items: vec![REPLACE_ITEM.to_owned()],
        }
    }

    pub fn unindex(id: OperationId, external_id: impl Into<String>) -> Self {
        Self {
            id,
            endpoint: Endpoint::Unindex,
            document_id: external_id.into(),
            required_items: vec![UNINDEX_ITEM.to_owned()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestItem {
    pub operation_id: OperationId,
    pub item: String,
}

impl RequestItem {
    pub fn new(operation_id: OperationId, item: impl Into<String>) -> Self {
        Self {
            operation_id,
            item: item.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub endpoint: Endpoint,
    /// Generator scheduling time. It is distinct from body_started_at so a
    /// blocked client cannot claim offered load it never sent.
    pub scheduled_at: Duration,
    /// The time the client starts the HTTP body write.
    pub body_started_at: Duration,
    pub items: Vec<RequestItem>,
}

impl Request {
    pub fn new(
        id: RequestId,
        endpoint: Endpoint,
        scheduled_at: Duration,
        body_started_at: Duration,
        items: Vec<RequestItem>,
    ) -> Self {
        Self {
            id,
            endpoint,
            scheduled_at,
            body_started_at,
            items,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceFault {
    EmptyBatch,
    BatchTooLarge(RequestId),
    RequestOperationEndpointMismatch(RequestId, OperationId),
    InvalidRequestSchedule(RequestId),
    RequestQueueDelayTooLong(RequestId, Duration, Duration),
    OperationBodyStartTooLate(OperationId, Duration, Duration),
    ScheduledInputRequestMissedInput(RequestId),
    InvalidQuerySchedule,
    ScheduledInputQueryMissedInput,
    DuplicateOperation(OperationId),
    DuplicateRequest(RequestId),
    EmptyRequiredItems(OperationId),
    DuplicateRequiredItem(OperationId, String),
    DuplicateRequestItem(RequestId, OperationId, String),
    DuplicateRequestDocumentTarget(RequestId, String),
    DuplicateItemResult(RequestId, OperationId, String),
    UnknownOperation(OperationId),
    UnknownRequest(RequestId),
    UnexpectedItem(OperationId, String),
    MissingItemResult(RequestId, OperationId, String),
    InvalidInputWindow,
    InvalidRequestLatency(RequestId),
    IncompleteQueryLatency,
    InvalidQueryLatency,
    UnderfilledBatchBeforeFinalTail(Endpoint),
    MultipleUnderfilledBatches(Endpoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFailure {
    Evidence(EvidenceFault),
    MissingInputWindow,
    InputDurationTooShort(Duration, Duration),
    GeneratorUnderload(u64, u64),
    /// The generator did not create the mixed document schedule for this
    /// second. This is schedule evidence only: document throughput itself is
    /// counted from all required HTTP body starts over the full window, so
    /// 1,000-item requests may burst naturally.
    GeneratorScheduleUnderloadBucket(u64, u64, u64),
    CompletionRateTooLow(u64, u64, u64),
    MissingRequiredItemSubmissions(u64),
    MissingMutationEndpoint(Endpoint),
    MissingQueryClass(QueryClass),
    PendingRequestsWithoutEndTime(usize),
    RequestErrors(u64),
    ClientCancellations(u64),
    MissingRequestLatency,
    RequestP99TooHigh(Duration, Duration),
    RequestMaxTooHigh(Duration, Duration),
    QueryUnderload(u64, u64),
    QueryUnderloadBucket(u64, u64, u64),
    QueryErrorsOrTimeouts(u64),
    MissingQueryLatency,
    QueryP99TooHigh(Duration, Duration),
    QueryMaxTooHigh(Duration, Duration),
    DrainTooSlow(Duration, Duration),
    InputRequestsNotDrained(u64),
    InputQueriesNotDrained(u64),
    MissingCheckpoint,
    MissingMerge,
    PeakRssMissing,
    PeakRssTooHigh(u64, u64),
}

#[derive(Debug, Clone)]
pub struct WorkloadReport {
    pub case: WorkloadCase,
    pub requests_offered: u64,
    pub requests_finished: u64,
    pub requests_completed: u64,
    pub requests_started_in_input: u64,
    pub requests_finished_in_input: u64,
    pub requests_completed_in_input: u64,
    pub request_errors: u64,
    pub client_cancellations: u64,
    pub items_offered: u64,
    pub items_completed: u64,
    pub items_failed: u64,
    pub items_started_in_input: u64,
    pub items_completed_in_input: u64,
    pub docops_offered: u64,
    pub docops_completed: u64,
    pub docops_offered_in_input: u64,
    pub docops_completed_in_input: u64,
    pub index_requests_started_in_input: u64,
    pub replace_requests_started_in_input: u64,
    pub unindex_requests_started_in_input: u64,
    pub input_duration: Option<Duration>,
    /// Observed from the last request that began in the input window. This
    /// makes a receipt describe the drain that occurred, rather than merely
    /// repeating the configured 60-second upper limit.
    pub request_drain: Option<Duration>,
    pub request_latency_p99: Option<Duration>,
    pub request_latency_max: Option<Duration>,
    pub queries_offered: u64,
    pub queries_completed: u64,
    pub queries_started_in_input: u64,
    pub queries_completed_in_input: u64,
    pub hot_queries_started_in_input: u64,
    pub idle_queries_started_in_input: u64,
    /// Observed from the last query that began in the input window.
    pub query_drain: Option<Duration>,
    pub query_errors_or_timeouts: u64,
    pub query_latency_p99: Option<Duration>,
    pub query_latency_max: Option<Duration>,
    pub checkpoints: u64,
    pub merges: u64,
    pub peak_rss_bytes: Option<u64>,
}

#[derive(Debug)]
struct ActiveOperation {
    endpoint: Endpoint,
    document_id: String,
    prepared_at: Duration,
    prepared_in_input: bool,
    required: BTreeSet<String>,
    submitted: BTreeSet<String>,
    submitted_in_input: BTreeSet<String>,
    started_at: Option<Duration>,
    started_in_input: bool,
    successful: BTreeSet<String>,
    failed: bool,
    completed: bool,
}

#[derive(Debug)]
struct PendingItem {
    operation_id: OperationId,
    item: String,
    outcome: Option<Outcome>,
}

#[derive(Debug)]
struct PendingRequest {
    body_started_at: Duration,
    body_started_in_input: bool,
    items: Vec<PendingItem>,
    client_cancelled: bool,
}

#[derive(Debug, Default)]
struct BatchEvidence {
    saw_underfilled: bool,
}

pub struct WorkloadLedger {
    case: WorkloadCase,
    limits: ApprovedLimits,
    faults: Vec<EvidenceFault>,
    operations: HashMap<OperationId, ActiveOperation>,
    seen_operations: BTreeSet<OperationId>,
    requests: HashMap<RequestId, PendingRequest>,
    seen_requests: BTreeSet<RequestId>,
    input_window: Option<(Duration, Duration)>,
    requests_offered: u64,
    requests_finished: u64,
    requests_completed: u64,
    requests_started_in_input: u64,
    requests_finished_in_input: u64,
    requests_completed_in_input: u64,
    client_cancellations: u64,
    request_errors: u64,
    items_offered: u64,
    items_completed: u64,
    items_failed: u64,
    items_started_in_input: u64,
    items_completed_in_input: u64,
    docops_offered: u64,
    docops_completed: u64,
    docops_offered_in_input: u64,
    docops_completed_in_input: u64,
    request_latencies: Vec<Duration>,
    queries_offered: u64,
    queries_completed: u64,
    queries_started_in_input: u64,
    queries_completed_in_input: u64,
    query_errors_or_timeouts: u64,
    query_latencies: Vec<Duration>,
    checkpoints: u64,
    merges: u64,
    peak_rss_bytes: Option<u64>,
    last_input_request_completion: Option<Duration>,
    last_input_query_completion: Option<Duration>,
    // Kept under this historical name because the release inventory reads
    // it. These are scheduled mixed-operation turns, never actual throughput
    // credit; actual full-document throughput is `docops_offered_in_input`.
    input_docop_offer_buckets: Vec<u64>,
    input_query_start_buckets: Vec<u64>,
    input_requests_submitted: u64,
    input_requests_finished: u64,
    input_queries_submitted: u64,
    input_queries_finished: u64,
    input_endpoint_requests: BTreeMap<Endpoint, u64>,
    input_query_classes: BTreeMap<QueryClass, u64>,
    batch_evidence: HashMap<Endpoint, BatchEvidence>,
}

impl WorkloadLedger {
    pub fn new(case: WorkloadCase) -> Self {
        Self::with_limits(case, ApprovedLimits::approved())
    }

    fn with_limits(case: WorkloadCase, limits: ApprovedLimits) -> Self {
        Self {
            case,
            limits,
            faults: Vec::new(),
            operations: HashMap::new(),
            seen_operations: BTreeSet::new(),
            requests: HashMap::new(),
            seen_requests: BTreeSet::new(),
            input_window: None,
            requests_offered: 0,
            requests_finished: 0,
            requests_completed: 0,
            requests_started_in_input: 0,
            requests_finished_in_input: 0,
            requests_completed_in_input: 0,
            client_cancellations: 0,
            request_errors: 0,
            items_offered: 0,
            items_completed: 0,
            items_failed: 0,
            items_started_in_input: 0,
            items_completed_in_input: 0,
            docops_offered: 0,
            docops_completed: 0,
            docops_offered_in_input: 0,
            docops_completed_in_input: 0,
            request_latencies: Vec::new(),
            queries_offered: 0,
            queries_completed: 0,
            queries_started_in_input: 0,
            queries_completed_in_input: 0,
            query_errors_or_timeouts: 0,
            query_latencies: Vec::new(),
            checkpoints: 0,
            merges: 0,
            peak_rss_bytes: None,
            last_input_request_completion: None,
            last_input_query_completion: None,
            input_docop_offer_buckets: Vec::new(),
            input_query_start_buckets: Vec::new(),
            input_requests_submitted: 0,
            input_requests_finished: 0,
            input_queries_submitted: 0,
            input_queries_finished: 0,
            input_endpoint_requests: BTreeMap::new(),
            input_query_classes: BTreeMap::new(),
            batch_evidence: HashMap::new(),
        }
    }

    fn input_bucket(&self, at: Duration) -> Option<usize> {
        let (started_at, finished_at) = self.input_window?;
        (at >= started_at && at < finished_at).then(|| (at - started_at).as_secs() as usize)
    }

    fn input_deadline(&self) -> Option<Duration> {
        self.input_window
            .map(|(_, finished_at)| finished_at + self.limits.drain_limit)
    }

    /// A builder may retain an item only until enough work from the existing
    /// rotating scheduler can fill its selected fixed-size batch. The durable
    /// drain allowance then bounds a formed request's driver queue time.
    fn maximum_operation_body_delay(
        batch_items: usize,
        required_items: usize,
        docops_per_second: u64,
        drain_limit: Duration,
    ) -> Duration {
        let items_per_turn = u64::try_from(required_items)
            .unwrap_or(u64::MAX)
            .saturating_mul(docops_per_second)
            .max(1);
        let batch_items = u64::try_from(batch_items).unwrap_or(u64::MAX);
        let turns_to_fill =
            batch_items.saturating_add(items_per_turn.saturating_sub(1)) / items_per_turn;
        Duration::from_secs(
            turns_to_fill
                .saturating_mul(MUTATION_ROTATION_SECONDS)
                .saturating_add(drain_limit.as_secs()),
        )
    }

    /// Records planned work only. It cannot add rate evidence until every
    /// required item reaches an actual HTTP body start in `submit_request`.
    pub fn begin_operation(&mut self, prepared_at: Duration, operation: DocumentOperation) {
        if !self.seen_operations.insert(operation.id) {
            self.faults
                .push(EvidenceFault::DuplicateOperation(operation.id));
            return;
        }

        let prepared_in_input = self.input_bucket(prepared_at).is_some();
        if let Some(bucket) = self.input_bucket(prepared_at) {
            self.input_docop_offer_buckets[bucket] += 1;
        }
        let mut required = BTreeSet::new();
        let mut failed = false;
        for item in operation.required_items {
            if !required.insert(item.clone()) {
                self.faults
                    .push(EvidenceFault::DuplicateRequiredItem(operation.id, item));
                failed = true;
            }
        }
        if required.is_empty() {
            self.faults
                .push(EvidenceFault::EmptyRequiredItems(operation.id));
            failed = true;
        }
        self.operations.insert(
            operation.id,
            ActiveOperation {
                endpoint: operation.endpoint,
                document_id: operation.document_id,
                prepared_at,
                prepared_in_input,
                required,
                submitted: BTreeSet::new(),
                submitted_in_input: BTreeSet::new(),
                started_at: None,
                started_in_input: false,
                successful: BTreeSet::new(),
                failed,
                completed: false,
            },
        );
    }

    pub fn submit_request(&mut self, request: Request) {
        if !self.seen_requests.insert(request.id) {
            self.faults
                .push(EvidenceFault::DuplicateRequest(request.id));
            return;
        }
        self.requests_offered += 1;
        if request.body_started_at < request.scheduled_at {
            self.faults
                .push(EvidenceFault::InvalidRequestSchedule(request.id));
        } else {
            let queue_delay = request.body_started_at - request.scheduled_at;
            if queue_delay > self.limits.drain_limit {
                self.faults.push(EvidenceFault::RequestQueueDelayTooLong(
                    request.id,
                    queue_delay,
                    self.limits.drain_limit,
                ));
            }
        }
        let scheduled_in_input = self.input_bucket(request.scheduled_at).is_some();
        let body_started_bucket = self.input_bucket(request.body_started_at);
        let body_started_in_input = body_started_bucket.is_some();
        if scheduled_in_input && !body_started_in_input {
            self.faults
                .push(EvidenceFault::ScheduledInputRequestMissedInput(request.id));
        }
        if body_started_in_input {
            self.input_requests_submitted += 1;
            self.requests_started_in_input += 1;
            *self
                .input_endpoint_requests
                .entry(request.endpoint)
                .or_default() += 1;
            let expected = self.case.batch_size_for(request.endpoint);
            if request.items.len() < expected {
                let evidence = self.batch_evidence.entry(request.endpoint).or_default();
                if evidence.saw_underfilled {
                    self.faults
                        .push(EvidenceFault::MultipleUnderfilledBatches(request.endpoint));
                }
                evidence.saw_underfilled = true;
            } else if self
                .batch_evidence
                .get(&request.endpoint)
                .is_some_and(|evidence| evidence.saw_underfilled)
            {
                self.faults
                    .push(EvidenceFault::UnderfilledBatchBeforeFinalTail(
                        request.endpoint,
                    ));
            }
        }
        self.items_offered += request.items.len() as u64;
        if body_started_in_input {
            self.items_started_in_input += request.items.len() as u64;
        }
        if request.items.is_empty() {
            self.faults.push(EvidenceFault::EmptyBatch);
        }
        if request.items.len() > self.case.batch_size_for(request.endpoint) {
            self.faults.push(EvidenceFault::BatchTooLarge(request.id));
        }

        let mut seen_items = BTreeSet::new();
        let mut seen_operations = BTreeSet::new();
        let mut seen_document_targets = BTreeSet::new();
        let mut pending_items = Vec::with_capacity(request.items.len());
        let index_batch_size = self.case.batch_size_for(Endpoint::Index);
        let replace_batch_size = self.case.batch_size_for(Endpoint::Replace);
        let unindex_batch_size = self.case.batch_size_for(Endpoint::Unindex);
        let docops_per_second = self.limits.docops_per_second;
        let drain_limit = self.limits.drain_limit;
        for item in request.items {
            let mut started_operation = false;
            let mut started_in_input = false;
            let mut document_target = None;
            let mut body_start_fault = None;
            match self.operations.get_mut(&item.operation_id) {
                None => self
                    .faults
                    .push(EvidenceFault::UnknownOperation(item.operation_id)),
                Some(operation) if operation.endpoint != request.endpoint => {
                    self.faults
                        .push(EvidenceFault::RequestOperationEndpointMismatch(
                            request.id,
                            item.operation_id,
                        ))
                }
                Some(operation) => {
                    let first_item_for_operation = seen_operations.insert(item.operation_id);
                    if first_item_for_operation {
                        document_target = Some(operation.document_id.clone());
                    }
                    if request.body_started_at < operation.prepared_at {
                        body_start_fault = Some(EvidenceFault::InvalidRequestSchedule(request.id));
                    }
                    if operation.required.contains(&item.item) {
                        if operation.submitted.insert(item.item.clone()) {
                            if body_started_in_input {
                                operation.submitted_in_input.insert(item.item.clone());
                            }
                            if operation.started_at.is_none()
                                && operation.submitted.len() == operation.required.len()
                            {
                                let batch_size = match operation.endpoint {
                                    Endpoint::Index => index_batch_size,
                                    Endpoint::Replace => replace_batch_size,
                                    Endpoint::Unindex => unindex_batch_size,
                                };
                                let allowed = Self::maximum_operation_body_delay(
                                    batch_size,
                                    operation.required.len(),
                                    docops_per_second,
                                    drain_limit,
                                );
                                let body_delay = request
                                    .body_started_at
                                    .saturating_sub(operation.prepared_at);
                                if body_delay > allowed {
                                    body_start_fault =
                                        Some(EvidenceFault::OperationBodyStartTooLate(
                                            item.operation_id,
                                            body_delay,
                                            allowed,
                                        ));
                                }
                                operation.started_at = Some(request.body_started_at);
                                operation.started_in_input = body_started_in_input
                                    && operation.submitted_in_input.len()
                                        == operation.required.len();
                                started_operation = true;
                                started_in_input = operation.started_in_input;
                            }
                        } else {
                            operation.failed = true;
                            self.faults.push(EvidenceFault::DuplicateRequiredItem(
                                item.operation_id,
                                item.item.clone(),
                            ));
                        }
                    }
                }
            }
            if let Some(fault) = body_start_fault {
                self.faults.push(fault);
            }
            if let Some(document_target) = document_target {
                if !seen_document_targets.insert(document_target.clone()) {
                    self.faults
                        .push(EvidenceFault::DuplicateRequestDocumentTarget(
                            request.id,
                            document_target,
                        ));
                }
            }
            if started_operation {
                self.docops_offered += 1;
                if started_in_input {
                    self.docops_offered_in_input += 1;
                }
            }
            if !seen_items.insert((item.operation_id, item.item.clone())) {
                self.faults.push(EvidenceFault::DuplicateRequestItem(
                    request.id,
                    item.operation_id,
                    item.item.clone(),
                ));
            }
            pending_items.push(PendingItem {
                operation_id: item.operation_id,
                item: item.item,
                outcome: None,
            });
        }
        self.requests.insert(
            request.id,
            PendingRequest {
                body_started_at: request.body_started_at,
                body_started_in_input,
                items: pending_items,
                client_cancelled: false,
            },
        );
    }

    pub fn record_item_result(
        &mut self,
        request_id: RequestId,
        operation_id: OperationId,
        item: &str,
        outcome: Outcome,
    ) {
        let Some(request) = self.requests.get_mut(&request_id) else {
            self.faults.push(EvidenceFault::UnknownRequest(request_id));
            return;
        };
        let Some(pending) = request
            .items
            .iter_mut()
            .find(|pending| pending.operation_id == operation_id && pending.item == item)
        else {
            self.faults
                .push(EvidenceFault::UnexpectedItem(operation_id, item.to_owned()));
            return;
        };
        if pending.outcome.is_some() {
            self.faults.push(EvidenceFault::DuplicateItemResult(
                request_id,
                operation_id,
                item.to_owned(),
            ));
        } else {
            pending.outcome = Some(outcome);
        }
    }

    pub fn record_client_cancelled(&mut self, request_id: RequestId) {
        let Some(request) = self.requests.get_mut(&request_id) else {
            self.faults.push(EvidenceFault::UnknownRequest(request_id));
            return;
        };
        if !request.client_cancelled {
            request.client_cancelled = true;
            self.client_cancellations += 1;
        }
    }

    pub fn finish_request(
        &mut self,
        request_id: RequestId,
        finished_at: Duration,
        request_outcome: Outcome,
    ) {
        let Some(request) = self.requests.remove(&request_id) else {
            self.faults.push(EvidenceFault::UnknownRequest(request_id));
            return;
        };
        self.requests_finished += 1;
        let finished_in_input = self.input_bucket(finished_at).is_some();
        if finished_at < request.body_started_at {
            self.faults
                .push(EvidenceFault::InvalidRequestLatency(request_id));
        } else {
            self.request_latencies
                .push(finished_at - request.body_started_at);
        }
        if request.body_started_in_input {
            self.input_requests_finished += 1;
            if finished_in_input {
                self.requests_finished_in_input += 1;
            }
            self.last_input_request_completion = Some(
                self.last_input_request_completion
                    .map_or(finished_at, |prior| prior.max(finished_at)),
            );
        }
        if request_outcome == Outcome::Succeeded {
            self.requests_completed += 1;
            if request.body_started_in_input && finished_in_input {
                self.requests_completed_in_input += 1;
            }
        } else {
            self.request_errors += 1;
        }

        for pending in request.items {
            match (request_outcome, pending.outcome) {
                (Outcome::Succeeded, Some(Outcome::Succeeded)) => {
                    self.apply_success(
                        pending.operation_id,
                        &pending.item,
                        finished_at,
                        request.body_started_in_input,
                    );
                }
                (_, Some(Outcome::Succeeded)) => {
                    self.items_failed += 1;
                    self.mark_failed(pending.operation_id);
                }
                (_, Some(Outcome::Failed | Outcome::TimedOut)) => {
                    self.items_failed += 1;
                    self.request_errors += 1;
                    self.mark_failed(pending.operation_id);
                }
                (_, None) => {
                    self.faults.push(EvidenceFault::MissingItemResult(
                        request_id,
                        pending.operation_id,
                        pending.item,
                    ));
                    self.mark_failed(pending.operation_id);
                }
            }
        }
    }

    fn apply_success(
        &mut self,
        operation_id: OperationId,
        item: &str,
        finished_at: Duration,
        body_started_in_input: bool,
    ) {
        let mut fault = None;
        let mut counted = false;
        let mut completed = false;
        let mut started_in_input = false;
        match self.operations.get_mut(&operation_id) {
            None => fault = Some(EvidenceFault::UnknownOperation(operation_id)),
            Some(operation) => {
                if !operation.required.contains(item) {
                    operation.failed = true;
                    fault = Some(EvidenceFault::UnexpectedItem(operation_id, item.to_owned()));
                } else if !operation.successful.insert(item.to_owned()) {
                    operation.failed = true;
                    fault = Some(EvidenceFault::DuplicateRequiredItem(
                        operation_id,
                        item.to_owned(),
                    ));
                } else {
                    counted = true;
                    completed = !operation.completed
                        && !operation.failed
                        && operation.successful.len() == operation.required.len();
                    started_in_input = operation.started_in_input;
                    if completed {
                        operation.completed = true;
                    }
                }
            }
        }
        if let Some(fault) = fault {
            self.faults.push(fault);
            return;
        }
        if counted {
            self.items_completed += 1;
            if body_started_in_input && self.input_bucket(finished_at).is_some() {
                self.items_completed_in_input += 1;
            }
        }
        if completed {
            self.docops_completed += 1;
            if started_in_input && self.input_bucket(finished_at).is_some() {
                self.docops_completed_in_input += 1;
            }
        }
    }

    fn mark_failed(&mut self, operation_id: OperationId) {
        match self.operations.get_mut(&operation_id) {
            Some(operation) => operation.failed = true,
            None => self
                .faults
                .push(EvidenceFault::UnknownOperation(operation_id)),
        }
    }

    pub fn set_input_window(&mut self, started_at: Duration, finished_at: Duration) {
        if finished_at <= started_at {
            self.faults.push(EvidenceFault::InvalidInputWindow);
        } else {
            self.input_window = Some((started_at, finished_at));
            let buckets = (finished_at - started_at).as_secs() as usize;
            if buckets == 0 {
                self.faults.push(EvidenceFault::InvalidInputWindow);
            } else {
                self.input_docop_offer_buckets = vec![0; buckets];
                self.input_query_start_buckets = vec![0; buckets];
            }
        }
    }

    pub fn record_query(
        &mut self,
        scheduled_at: Duration,
        body_started_at: Duration,
        finished_at: Option<Duration>,
        outcome: Outcome,
    ) {
        self.record_classified_query(
            QueryClass::Hot,
            scheduled_at,
            body_started_at,
            finished_at,
            outcome,
        );
    }

    pub fn record_classified_query(
        &mut self,
        class: QueryClass,
        scheduled_at: Duration,
        body_started_at: Duration,
        finished_at: Option<Duration>,
        outcome: Outcome,
    ) {
        self.queries_offered += 1;
        if body_started_at < scheduled_at {
            self.faults.push(EvidenceFault::InvalidQuerySchedule);
        }
        let scheduled_in_input = self.input_bucket(scheduled_at).is_some();
        let body_started_in_input = self.input_bucket(body_started_at).is_some();
        if scheduled_in_input && !body_started_in_input {
            self.faults
                .push(EvidenceFault::ScheduledInputQueryMissedInput);
        }
        if let Some(bucket) = self.input_bucket(body_started_at) {
            self.queries_started_in_input += 1;
            self.input_queries_submitted += 1;
            self.input_query_start_buckets[bucket] += 1;
            *self.input_query_classes.entry(class).or_default() += 1;
        }
        let Some(finished_at) = finished_at else {
            self.faults.push(EvidenceFault::IncompleteQueryLatency);
            return;
        };
        if finished_at < body_started_at {
            self.faults.push(EvidenceFault::InvalidQueryLatency);
            return;
        }
        let finished_in_input = self.input_bucket(finished_at).is_some();
        if body_started_in_input {
            self.query_latencies.push(finished_at - body_started_at);
            self.input_queries_finished += 1;
            self.last_input_query_completion = Some(
                self.last_input_query_completion
                    .map_or(finished_at, |prior| prior.max(finished_at)),
            );
        }
        if outcome == Outcome::Succeeded {
            self.queries_completed += 1;
            if body_started_in_input && finished_in_input {
                self.queries_completed_in_input += 1;
            }
        } else {
            self.query_errors_or_timeouts += 1;
        }
    }

    pub fn record_checkpoint_completion(&mut self) {
        self.checkpoints += 1;
    }

    pub fn record_merge_completion(&mut self) {
        self.merges += 1;
    }

    pub fn observe_peak_rss_bytes(&mut self, bytes: u64) {
        self.peak_rss_bytes = Some(self.peak_rss_bytes.map_or(bytes, |old| old.max(bytes)));
    }

    pub fn report(&self) -> WorkloadReport {
        WorkloadReport {
            case: self.case.clone(),
            requests_offered: self.requests_offered,
            requests_finished: self.requests_finished,
            requests_completed: self.requests_completed,
            requests_started_in_input: self.requests_started_in_input,
            requests_finished_in_input: self.requests_finished_in_input,
            requests_completed_in_input: self.requests_completed_in_input,
            request_errors: self.request_errors,
            client_cancellations: self.client_cancellations,
            items_offered: self.items_offered,
            items_completed: self.items_completed,
            items_failed: self.items_failed,
            items_started_in_input: self.items_started_in_input,
            items_completed_in_input: self.items_completed_in_input,
            docops_offered: self.docops_offered,
            docops_completed: self.docops_completed,
            docops_offered_in_input: self.docops_offered_in_input,
            docops_completed_in_input: self.docops_completed_in_input,
            index_requests_started_in_input: self
                .input_endpoint_requests
                .get(&Endpoint::Index)
                .copied()
                .unwrap_or_default(),
            replace_requests_started_in_input: self
                .input_endpoint_requests
                .get(&Endpoint::Replace)
                .copied()
                .unwrap_or_default(),
            unindex_requests_started_in_input: self
                .input_endpoint_requests
                .get(&Endpoint::Unindex)
                .copied()
                .unwrap_or_default(),
            input_duration: self
                .input_window
                .map(|(started_at, finished_at)| finished_at - started_at),
            request_drain: observed_drain(self.input_window, self.last_input_request_completion),
            request_latency_p99: percentile_99(&self.request_latencies),
            request_latency_max: self.request_latencies.iter().copied().max(),
            queries_offered: self.queries_offered,
            queries_completed: self.queries_completed,
            queries_started_in_input: self.queries_started_in_input,
            queries_completed_in_input: self.queries_completed_in_input,
            hot_queries_started_in_input: self
                .input_query_classes
                .get(&QueryClass::Hot)
                .copied()
                .unwrap_or_default(),
            idle_queries_started_in_input: self
                .input_query_classes
                .get(&QueryClass::Idle)
                .copied()
                .unwrap_or_default(),
            query_drain: observed_drain(self.input_window, self.last_input_query_completion),
            query_errors_or_timeouts: self.query_errors_or_timeouts,
            query_latency_p99: percentile_99(&self.query_latencies),
            query_latency_max: self.query_latencies.iter().copied().max(),
            checkpoints: self.checkpoints,
            merges: self.merges,
            peak_rss_bytes: self.peak_rss_bytes,
        }
    }

    pub fn validate(&self) -> Result<WorkloadReport, Vec<GateFailure>> {
        let report = self.report();
        let mut failures: Vec<GateFailure> = self
            .faults
            .iter()
            .cloned()
            .map(GateFailure::Evidence)
            .collect();
        if !self.requests.is_empty() {
            failures.push(GateFailure::PendingRequestsWithoutEndTime(
                self.requests.len(),
            ));
        }
        let Some(input_duration) = report.input_duration else {
            failures.push(GateFailure::MissingInputWindow);
            return Err(failures);
        };
        if input_duration < self.limits.input_duration {
            failures.push(GateFailure::InputDurationTooShort(
                input_duration,
                self.limits.input_duration,
            ));
        }

        let seconds = input_duration.as_secs();
        let required_docops = self.limits.docops_per_second * seconds;
        if report.docops_offered_in_input < required_docops {
            failures.push(GateFailure::GeneratorUnderload(
                report.docops_offered_in_input,
                required_docops,
            ));
        }
        for (second, offered) in self.input_docop_offer_buckets.iter().copied().enumerate() {
            if offered < self.limits.docops_per_second {
                failures.push(GateFailure::GeneratorScheduleUnderloadBucket(
                    second as u64,
                    offered,
                    self.limits.docops_per_second,
                ));
            }
        }
        let missing_submissions = self
            .operations
            .values()
            .filter(|operation| {
                operation.prepared_in_input
                    && self.input_bucket(operation.prepared_at).is_some()
                    && operation
                        .required
                        .iter()
                        .any(|item| !operation.submitted_in_input.contains(item))
            })
            .count() as u64;
        if missing_submissions > 0 {
            failures.push(GateFailure::MissingRequiredItemSubmissions(
                missing_submissions,
            ));
        }
        if report.docops_offered_in_input == 0
            || u128::from(report.docops_completed_in_input) * 100
                < u128::from(report.docops_offered_in_input)
                    * u128::from(self.limits.completed_percent)
        {
            failures.push(GateFailure::CompletionRateTooLow(
                report.docops_offered_in_input,
                report.docops_completed_in_input,
                self.limits.completed_percent,
            ));
        }
        if self.request_errors > 0 {
            failures.push(GateFailure::RequestErrors(self.request_errors));
        }
        if self.client_cancellations > 0 {
            failures.push(GateFailure::ClientCancellations(self.client_cancellations));
        }
        match report.request_latency_p99 {
            None => failures.push(GateFailure::MissingRequestLatency),
            Some(observed) if observed > self.limits.p99_limit => {
                failures.push(GateFailure::RequestP99TooHigh(
                    observed,
                    self.limits.p99_limit,
                ));
            }
            Some(_) => {}
        }
        if let Some(observed) = report.request_latency_max {
            if observed > self.limits.max_query_limit {
                failures.push(GateFailure::RequestMaxTooHigh(
                    observed,
                    self.limits.max_query_limit,
                ));
            }
        }

        if self.case.is_approved_matrix_cell() {
            for endpoint in [Endpoint::Index, Endpoint::Replace, Endpoint::Unindex] {
                if self
                    .input_endpoint_requests
                    .get(&endpoint)
                    .copied()
                    .unwrap_or_default()
                    == 0
                {
                    failures.push(GateFailure::MissingMutationEndpoint(endpoint));
                }
            }
            for class in [QueryClass::Hot, QueryClass::Idle] {
                if self
                    .input_query_classes
                    .get(&class)
                    .copied()
                    .unwrap_or_default()
                    == 0
                {
                    failures.push(GateFailure::MissingQueryClass(class));
                }
            }
        }

        let required_queries = self.limits.query_qps * seconds;
        if report.queries_completed_in_input < required_queries {
            failures.push(GateFailure::QueryUnderload(
                report.queries_completed_in_input,
                required_queries,
            ));
        }
        for (second, offered) in self.input_query_start_buckets.iter().copied().enumerate() {
            if offered < self.limits.query_qps {
                failures.push(GateFailure::QueryUnderloadBucket(
                    second as u64,
                    offered,
                    self.limits.query_qps,
                ));
            }
        }
        if report.query_errors_or_timeouts > 0 {
            failures.push(GateFailure::QueryErrorsOrTimeouts(
                report.query_errors_or_timeouts,
            ));
        }
        match report.query_latency_p99 {
            None => failures.push(GateFailure::MissingQueryLatency),
            Some(observed) if observed > self.limits.p99_limit => {
                failures.push(GateFailure::QueryP99TooHigh(
                    observed,
                    self.limits.p99_limit,
                ));
            }
            Some(_) => {}
        }
        if let Some(observed) = report.query_latency_max {
            if observed > self.limits.max_query_limit {
                failures.push(GateFailure::QueryMaxTooHigh(
                    observed,
                    self.limits.max_query_limit,
                ));
            }
        }

        if self.input_requests_finished != self.input_requests_submitted {
            failures.push(GateFailure::InputRequestsNotDrained(
                self.input_requests_submitted
                    .saturating_sub(self.input_requests_finished),
            ));
        }
        if self.input_queries_finished != self.input_queries_submitted {
            failures.push(GateFailure::InputQueriesNotDrained(
                self.input_queries_submitted
                    .saturating_sub(self.input_queries_finished),
            ));
        }
        if let (Some((_, input_finished)), Some(deadline), Some(last)) = (
            self.input_window,
            self.input_deadline(),
            self.last_input_request_completion,
        ) {
            if last > deadline {
                failures.push(GateFailure::DrainTooSlow(
                    last - input_finished,
                    self.limits.drain_limit,
                ));
            }
        }
        if let (Some((_, input_finished)), Some(deadline), Some(last)) = (
            self.input_window,
            self.input_deadline(),
            self.last_input_query_completion,
        ) {
            if last > deadline {
                failures.push(GateFailure::DrainTooSlow(
                    last - input_finished,
                    self.limits.drain_limit,
                ));
            }
        }
        if report.checkpoints == 0 {
            failures.push(GateFailure::MissingCheckpoint);
        }
        if report.merges == 0 {
            failures.push(GateFailure::MissingMerge);
        }
        match report.peak_rss_bytes {
            None => failures.push(GateFailure::PeakRssMissing),
            Some(observed) if observed > self.limits.rss_limit_bytes => {
                failures.push(GateFailure::PeakRssTooHigh(
                    observed,
                    self.limits.rss_limit_bytes,
                ));
            }
            Some(_) => {}
        }

        if failures.is_empty() {
            Ok(report)
        } else {
            Err(failures)
        }
    }
}

fn observed_drain(
    input_window: Option<(Duration, Duration)>,
    last_completion: Option<Duration>,
) -> Option<Duration> {
    let (_, input_finished) = input_window?;
    let last_completion = last_completion?;
    Some(if last_completion > input_finished {
        last_completion - input_finished
    } else {
        Duration::ZERO
    })
}

fn percentile_99(samples: &[Duration]) -> Option<Duration> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 99 + 99) / 100;
    sorted.get(rank.saturating_sub(1)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    fn test_limits() -> ApprovedLimits {
        ApprovedLimits {
            input_duration: at(1),
            docops_per_second: 1,
            query_qps: 1,
            completed_percent: 95,
            p99_limit: at(1),
            max_query_limit: at(5),
            drain_limit: at(60),
            rss_limit_bytes: 1_024,
        }
    }

    fn ledger(endpoint: Endpoint, batch_size: usize) -> WorkloadLedger {
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(endpoint, batch_size, Backend::FlatCpu),
            test_limits(),
        );
        ledger.set_input_window(at(0), at(1));
        ledger
    }

    fn record_valid_non_document_evidence(ledger: &mut WorkloadLedger) {
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            Some(Duration::from_millis(20)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(512);
    }

    fn fields_14() -> Vec<String> {
        (0..14).map(|number| format!("field-{number:02}")).collect()
    }

    fn complete_replace(ledger: &mut WorkloadLedger, operation: u64, request: u64) {
        ledger.begin_operation(
            at(0),
            DocumentOperation::replace(operation, format!("doc-{operation}")),
        );
        ledger.submit_request(Request::new(
            request,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(operation, REPLACE_ITEM)],
        ));
        ledger.record_item_result(request, operation, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(request, Duration::from_millis(1), Outcome::Succeeded);
    }

    fn record_query_series(ledger: &mut WorkloadLedger, seconds: u64) {
        for second in 0..seconds {
            let started_at = at(second);
            let class = if second % 2 == 0 {
                QueryClass::Hot
            } else {
                QueryClass::Idle
            };
            ledger.record_classified_query(
                class,
                started_at,
                started_at,
                Some(started_at + Duration::from_millis(1)),
                Outcome::Succeeded,
            );
        }
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);
    }

    #[test]
    fn fixed_batch_builder_bounds_allow_normal_largest_batch_retention() {
        // Index produces 1,400 field items on a selected turn, so a 1,000
        // field request can form on that turn. Unindex produces 100 IDs on a
        // selected turn, so its 1,000-ID request may retain the oldest item
        // across ten three-second rotation turns. The existing 60-second
        // backlog allowance is then added to both derived bounds.
        assert_eq!(
            WorkloadLedger::maximum_operation_body_delay(1_000, 14, 100, at(60)),
            at(63)
        );
        assert_eq!(
            WorkloadLedger::maximum_operation_body_delay(1_000, 1, 100, at(60)),
            at(90)
        );
        assert_eq!(
            WorkloadLedger::maximum_operation_body_delay(32, 1, 100, at(60)),
            at(63)
        );
    }

    #[test]
    fn late_request_and_query_completion_are_not_in_window_evidence() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(1, "late-completion"));
        ledger.submit_request(Request::new(
            1,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(1, REPLACE_ITEM)],
        ));
        ledger.record_item_result(1, 1, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(1, at(1), Outcome::Succeeded);
        ledger.record_query(at(0), at(0), Some(at(1)), Outcome::Succeeded);

        let report = ledger.report();
        assert_eq!(
            (
                report.docops_completed_in_input,
                report.queries_completed_in_input,
            ),
            (0, 0),
            "a response that finishes at or after the input end cannot prove in-window completion"
        );
    }

    #[test]
    fn prepared_replace_operations_without_timed_body_starts_are_not_sustained_input() {
        let mut limits = test_limits();
        limits.input_duration = at(3);
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Replace, 1, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(3));
        for operation in 1..=3 {
            ledger.begin_operation(
                at(operation - 1),
                DocumentOperation::replace(operation, format!("replace-{operation}")),
            );
        }
        record_query_series(&mut ledger, 3);

        let failures = ledger
            .validate()
            .expect_err("generator preparation cannot substitute for unsent request bodies");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::GeneratorUnderload(0, 3)) }));
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::MissingRequiredItemSubmissions(3)) }));
    }

    #[test]
    fn split_index_document_starts_when_all_fourteen_fields_body_start() {
        let mut limits = test_limits();
        limits.input_duration = at(3);
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Index, 13, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(3));
        let fields = fields_14();
        for operation in 1..=3 {
            ledger.begin_operation(
                at(operation - 1),
                DocumentOperation::index(operation, format!("index-{operation}"), fields.clone()),
            );
        }
        for operation in 1..=3 {
            let request = 200 + operation;
            ledger.submit_request(Request::new(
                request,
                Endpoint::Index,
                at(operation - 1),
                at(operation - 1),
                fields
                    .iter()
                    .take(13)
                    .map(|field| RequestItem::new(operation, field.clone()))
                    .collect(),
            ));
            for field in fields.iter().take(13) {
                ledger.record_item_result(request, operation, field, Outcome::Succeeded);
            }
            ledger.finish_request(
                request,
                at(operation - 1) + Duration::from_millis(1),
                Outcome::Succeeded,
            );
        }
        record_query_series(&mut ledger, 3);

        let failures = ledger
            .validate()
            .expect_err("thirteen Index fields cannot start a full document operation");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::GeneratorUnderload(0, 3)) }));
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::MissingRequiredItemSubmissions(3)) }));
    }

    #[test]
    fn late_fourteenth_index_field_cannot_start_all_documents_at_input_edge() {
        let mut limits = test_limits();
        limits.input_duration = at(10);
        limits.drain_limit = at(1);
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Index, 1, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(10));
        let fields = fields_14();
        let mut request = 1;

        for operation in 0..10 {
            ledger.begin_operation(
                at(operation),
                DocumentOperation::index(
                    operation + 1,
                    format!("index-{operation}"),
                    fields.clone(),
                ),
            );
            for field in fields.iter().take(13) {
                ledger.submit_request(Request::new(
                    request,
                    Endpoint::Index,
                    at(operation),
                    at(operation),
                    vec![RequestItem::new(operation + 1, field.clone())],
                ));
                ledger.record_item_result(request, operation + 1, field, Outcome::Succeeded);
                ledger.finish_request(
                    request,
                    at(operation) + Duration::from_millis(1),
                    Outcome::Succeeded,
                );
                request += 1;
            }
        }
        for operation in 0..10 {
            let final_field = fields
                .last()
                .expect("fourteen-field fixture has a final field");
            ledger.submit_request(Request::new(
                request,
                Endpoint::Index,
                at(9),
                at(9),
                vec![RequestItem::new(operation + 1, final_field.clone())],
            ));
            ledger.record_item_result(request, operation + 1, final_field, Outcome::Succeeded);
            ledger.finish_request(
                request,
                at(9) + Duration::from_millis(1),
                Outcome::Succeeded,
            );
            request += 1;
        }
        record_query_series(&mut ledger, 10);

        let failures = ledger.validate().expect_err(
            "a final Index field held until the input edge cannot prove a sustained workload",
        );
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::OperationBodyStartTooLate(
                    1,
                    observed,
                    allowed,
                )) if *observed == at(9) && *allowed == at(4)
            )
        }));
    }

    #[test]
    fn replace_only_with_queries_cannot_prove_required_mutation_and_query_mix() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(1, "replace-only"));
        ledger.submit_request(Request::new(
            1,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(1, REPLACE_ITEM)],
        ));
        ledger.record_item_result(1, 1, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(1, Duration::from_millis(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        let failures = ledger.validate().expect_err(
            "one mutation endpoint and hot-only queries cannot prove the mixed workload",
        );
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::MissingMutationEndpoint(Endpoint::Index)
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::MissingMutationEndpoint(Endpoint::Unindex)
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(failure, GateFailure::MissingQueryClass(QueryClass::Idle))
        }));
    }

    #[test]
    fn duplicate_logical_target_in_one_batch_is_not_two_scheduled_documents() {
        let mut limits = test_limits();
        limits.docops_per_second = 2;
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Replace, 2, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(1));
        ledger.begin_operation(at(0), DocumentOperation::replace(1, "same-target"));
        ledger.begin_operation(at(0), DocumentOperation::replace(2, "same-target"));
        ledger.submit_request(Request::new(
            1,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![
                RequestItem::new(1, REPLACE_ITEM),
                RequestItem::new(2, REPLACE_ITEM),
            ],
        ));
        ledger.record_item_result(1, 1, REPLACE_ITEM, Outcome::Succeeded);
        ledger.record_item_result(1, 2, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(1, Duration::from_millis(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        assert!(
            ledger.validate().is_err(),
            "a duplicate logical target in one scheduled batch cannot substitute for two documents"
        );
    }

    #[test]
    fn report_records_observed_request_and_query_drain_not_the_configured_limit() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(1, "drain-doc"));
        ledger.submit_request(Request::new(
            1,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(1, REPLACE_ITEM)],
        ));
        ledger.record_item_result(1, 1, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(1, at(3), Outcome::Succeeded);
        ledger.record_query(at(0), at(0), Some(at(4)), Outcome::Succeeded);

        let report = ledger.report();
        assert_eq!(report.request_drain, Some(at(2)));
        assert_eq!(report.query_drain, Some(at(3)));
    }

    #[test]
    fn split_fourteen_index_fields_are_one_document_operation_not_fourteen() {
        let mut ledger = ledger(Endpoint::Index, 5);
        let fields = fields_14();
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(1, "doc-split", fields.clone()),
        );

        for (offset, batch) in fields.chunks(5).enumerate() {
            let request = 10 + offset as u64;
            let items = batch
                .iter()
                .map(|field| RequestItem::new(1, field.clone()))
                .collect();
            ledger.submit_request(Request::new(request, Endpoint::Index, at(0), at(0), items));
            for field in batch {
                ledger.record_item_result(request, 1, field, Outcome::Succeeded);
            }
            ledger.finish_request(request, Duration::from_millis(1), Outcome::Succeeded);
        }
        record_valid_non_document_evidence(&mut ledger);

        let report = ledger.validate().expect("complete split Index operation");
        assert_eq!(report.items_completed, 14, "Index items are fields");
        assert_eq!(
            report.docops_completed, 1,
            "14 successful fields are one complete document operation"
        );
        assert_eq!(report.requests_completed, 3);
        assert_eq!(report.docops_offered, 1);
    }

    #[test]
    fn one_request_can_complete_fields_for_different_documents() {
        let mut ledger = ledger(Endpoint::Index, 2);
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(1, "doc-a", vec!["title".to_owned()]),
        );
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(2, "doc-b", vec!["body".to_owned()]),
        );
        ledger.submit_request(Request::new(
            11,
            Endpoint::Index,
            at(0),
            at(0),
            vec![RequestItem::new(1, "title"), RequestItem::new(2, "body")],
        ));
        ledger.record_item_result(11, 1, "title", Outcome::Succeeded);
        ledger.record_item_result(11, 2, "body", Outcome::Succeeded);
        ledger.finish_request(11, Duration::from_millis(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        let report = ledger.validate().expect("both operations complete");
        assert_eq!(report.items_completed, 2);
        assert_eq!(report.docops_completed, 2);
        assert_eq!(report.requests_completed, 1);
    }

    #[test]
    fn missing_index_field_never_counts_a_document_as_complete() {
        let mut ledger = ledger(Endpoint::Index, 14);
        let fields = fields_14();
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(2, "doc-missing", fields.clone()),
        );
        ledger.submit_request(Request::new(
            12,
            Endpoint::Index,
            at(0),
            at(0),
            fields[..13]
                .iter()
                .map(|field| RequestItem::new(2, field.clone()))
                .collect(),
        ));
        for field in &fields[..13] {
            ledger.record_item_result(12, 2, field, Outcome::Succeeded);
        }
        ledger.finish_request(12, at(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        assert_eq!(ledger.report().items_completed, 13);
        assert_eq!(ledger.report().docops_completed, 0);
        let failures = ledger
            .validate()
            .expect_err("missing field rejects completion");
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::CompletionRateTooLow(..))));
    }

    #[test]
    fn failed_request_is_not_a_completed_document_operation() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(3, "doc-failed"));
        ledger.submit_request(Request::new(
            13,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(3, REPLACE_ITEM)],
        ));
        ledger.record_item_result(13, 3, REPLACE_ITEM, Outcome::Failed);
        ledger.finish_request(13, at(1), Outcome::Failed);
        record_valid_non_document_evidence(&mut ledger);

        assert_eq!(ledger.report().docops_completed, 0);
        let failures = ledger.validate().expect_err("request error rejects gate");
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::RequestErrors(..))));
    }

    #[test]
    fn duplicate_index_field_evidence_is_rejected_not_double_counted() {
        let mut ledger = ledger(Endpoint::Index, 3);
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(
                4,
                "doc-duplicate",
                vec!["title".to_owned(), "body".to_owned()],
            ),
        );
        ledger.submit_request(Request::new(
            14,
            Endpoint::Index,
            at(0),
            at(0),
            vec![
                RequestItem::new(4, "title"),
                RequestItem::new(4, "title"),
                RequestItem::new(4, "body"),
            ],
        ));
        ledger.record_item_result(14, 4, "title", Outcome::Succeeded);
        ledger.record_item_result(14, 4, "title", Outcome::Succeeded);
        ledger.record_item_result(14, 4, "body", Outcome::Succeeded);
        ledger.finish_request(14, at(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        assert_eq!(ledger.report().docops_completed, 0);
        let failures = ledger.validate().expect_err("duplicate evidence rejects");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::DuplicateRequestItem(..))
                    | GateFailure::Evidence(EvidenceFault::DuplicateItemResult(..))
            )
        }));
    }

    #[test]
    fn cancelled_await_can_later_commit_and_complete() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(5, "doc-cancelled"));
        ledger.submit_request(Request::new(
            15,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(5, REPLACE_ITEM)],
        ));
        ledger.record_client_cancelled(15);
        ledger.record_item_result(15, 5, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(15, Duration::from_millis(1), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        let report = ledger.report();
        assert_eq!(report.client_cancellations, 1);
        assert_eq!(report.docops_completed, 1);
        assert_eq!(report.requests_completed, 1);
        let failures = ledger
            .validate()
            .expect_err("a qualifying workload cannot contain a client cancellation");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::ClientCancellations(1)) }));
    }

    #[test]
    fn replace_and_unindex_count_one_operation_per_document_or_id() {
        for (endpoint, marker) in [
            (Endpoint::Replace, REPLACE_ITEM),
            (Endpoint::Unindex, UNINDEX_ITEM),
        ] {
            let mut ledger = ledger(endpoint, 2);
            let first = if endpoint == Endpoint::Replace {
                DocumentOperation::replace(6, "doc-a")
            } else {
                DocumentOperation::unindex(6, "doc-a")
            };
            let second = if endpoint == Endpoint::Replace {
                DocumentOperation::replace(7, "doc-b")
            } else {
                DocumentOperation::unindex(7, "doc-b")
            };
            ledger.begin_operation(at(0), first);
            ledger.begin_operation(at(0), second);
            ledger.submit_request(Request::new(
                16,
                endpoint,
                at(0),
                at(0),
                vec![RequestItem::new(6, marker), RequestItem::new(7, marker)],
            ));
            ledger.record_item_result(16, 6, marker, Outcome::Succeeded);
            ledger.record_item_result(16, 7, marker, Outcome::Succeeded);
            ledger.finish_request(16, Duration::from_millis(1), Outcome::Succeeded);
            record_valid_non_document_evidence(&mut ledger);

            let report = ledger.validate().expect("complete doc or ID operations");
            assert_eq!(report.items_completed, 2);
            assert_eq!(report.docops_completed, 2);
        }
    }

    #[test]
    fn only_twenty_nine_minutes_is_rejected() {
        let mut ledger =
            WorkloadLedger::new(WorkloadCase::new(Endpoint::Replace, 1, Backend::HnswCpu));
        ledger.set_input_window(at(0), at(29 * 60));

        let failures = ledger.validate().expect_err("29 minutes cannot pass");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::InputDurationTooShort(observed, required)
                    if *observed == at(29 * 60) && *required == at(30 * 60)
            )
        }));
    }

    #[test]
    fn generator_underload_is_rejected_even_for_a_thirty_minute_window() {
        let mut ledger =
            WorkloadLedger::new(WorkloadCase::new(Endpoint::Replace, 1, Backend::FlatCpu));
        ledger.set_input_window(at(0), at(30 * 60));
        complete_replace(&mut ledger, 8, 18);
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("one docop is underloaded");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::GeneratorUnderload(1, 180_000)) }));
    }

    #[test]
    fn checkpoint_merge_and_peak_rss_are_required() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        complete_replace(&mut ledger, 9, 19);
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            Some(Duration::from_millis(20)),
            Outcome::Succeeded,
        );

        let failures = ledger
            .validate()
            .expect_err("missing operational evidence rejects");
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::MissingCheckpoint)));
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::MissingMerge)));
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::PeakRssMissing)));
    }

    #[test]
    fn query_without_end_time_is_not_silently_dropped() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        complete_replace(&mut ledger, 10, 20);
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            None,
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("missing query end rejects");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::IncompleteQueryLatency)
            )
        }));
    }

    #[test]
    fn slow_and_timed_out_queries_are_separate_rejections() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        complete_replace(&mut ledger, 11, 21);
        ledger.record_query(at(0), at(0), Some(at(2)), Outcome::Succeeded);
        ledger.record_query(
            at(0),
            at(0),
            Some(Duration::from_millis(50)),
            Outcome::TimedOut,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("slow and timeout reject");
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::QueryP99TooHigh(..))));
        assert!(failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::QueryErrorsOrTimeouts(..))));
    }

    #[test]
    fn request_p99_and_maximum_are_both_enforced() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(12, "slow-request"));
        ledger.submit_request(Request::new(
            22,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(12, REPLACE_ITEM)],
        ));
        ledger.record_item_result(22, 12, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(22, at(6), Outcome::Succeeded);
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            Some(Duration::from_millis(20)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger
            .validate()
            .expect_err("six-second request cannot satisfy p99 or maximum latency");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::RequestP99TooHigh(observed, limit)
                    if *observed == at(6) && *limit == at(1)
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::RequestMaxTooHigh(observed, limit)
                    if *observed == at(6) && *limit == at(5)
            )
        }));
    }

    #[test]
    fn request_without_a_finish_time_is_rejected() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(12, "doc-pending"));
        ledger.submit_request(Request::new(
            22,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(12, REPLACE_ITEM)],
        ));
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            Some(Duration::from_millis(20)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("pending request rejects");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::PendingRequestsWithoutEndTime(1)) }));
    }

    #[test]
    fn rss_above_the_limit_is_rejected() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        complete_replace(&mut ledger, 13, 23);
        record_valid_non_document_evidence(&mut ledger);
        ledger.observe_peak_rss_bytes(1_025);

        let failures = ledger.validate().expect_err("RSS over limit rejects");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::PeakRssTooHigh(1_025, 1_024)) }));
    }

    #[test]
    fn drain_over_sixty_seconds_is_rejected() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(14, "doc-slow-drain"));
        ledger.submit_request(Request::new(
            24,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(14, REPLACE_ITEM)],
        ));
        ledger.record_item_result(24, 14, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(24, at(62), Outcome::Succeeded);
        ledger.record_query(
            Duration::from_millis(10),
            Duration::from_millis(10),
            Some(Duration::from_millis(20)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("slow drain rejects");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::DrainTooSlow(observed, limit)
                    if *observed == at(61) && *limit == at(60)
            )
        }));
    }

    #[test]
    fn query_above_five_seconds_is_rejected_even_if_it_finishes() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        complete_replace(&mut ledger, 15, 25);
        ledger.record_query(at(0), at(0), Some(at(6)), Outcome::Succeeded);
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger.validate().expect_err("six-second query rejects");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::QueryMaxTooHigh(observed, limit)
                    if *observed == at(6) && *limit == at(5)
            )
        }));
    }

    #[test]
    fn duplicate_or_unknown_event_ids_and_mixed_endpoints_are_rejected() {
        let mut ledger = ledger(Endpoint::Index, 1);
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(16, "doc-integrity", vec!["title".to_owned()]),
        );
        ledger.begin_operation(
            at(0),
            DocumentOperation::index(16, "doc-duplicate-id", vec!["body".to_owned()]),
        );
        ledger.submit_request(Request::new(
            26,
            Endpoint::Index,
            at(0),
            at(0),
            vec![RequestItem::new(999, "unknown")],
        ));
        ledger.submit_request(Request::new(
            26,
            Endpoint::Index,
            at(0),
            at(0),
            vec![RequestItem::new(16, "title")],
        ));
        ledger.submit_request(Request::new(
            27,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(16, "title")],
        ));

        let failures = ledger
            .validate()
            .expect_err("untrusted IDs and endpoint reject");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::DuplicateOperation(16))
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::UnknownOperation(999))
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::DuplicateRequest(26))
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::RequestOperationEndpointMismatch(27, 16))
            )
        }));
    }

    #[test]
    fn a_declared_three_second_window_rejects_every_offer_in_its_first_second() {
        let mut limits = test_limits();
        limits.input_duration = at(3);
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Replace, 3, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(3));
        for operation in 1..=3 {
            ledger.begin_operation(
                at(0),
                DocumentOperation::replace(operation, format!("d{operation}")),
            );
        }
        ledger.submit_request(Request::new(
            100,
            Endpoint::Replace,
            at(0),
            at(0),
            (1..=3)
                .map(|operation| RequestItem::new(operation, REPLACE_ITEM))
                .collect(),
        ));
        for operation in 1..=3 {
            ledger.record_item_result(100, operation, REPLACE_ITEM, Outcome::Succeeded);
        }
        ledger.finish_request(100, at(1), Outcome::Succeeded);
        for _ in 0..3 {
            ledger.record_query(
                at(0),
                at(0),
                Some(Duration::from_millis(1)),
                Outcome::Succeeded,
            );
        }
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger
            .validate()
            .expect_err("a burst cannot prove a sustained input rate");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::GeneratorScheduleUnderloadBucket(1, 0, 1)
            )
        }));
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::QueryUnderloadBucket(1, 0, 1)) }));
    }

    #[test]
    fn an_unsubmitted_required_field_rejects_generator_coverage_at_exactly_ninety_five_percent() {
        let mut limits = test_limits();
        limits.docops_per_second = 20;
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Index, 14, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(1));
        let fields = fields_14();
        for operation in 1..=20 {
            ledger.begin_operation(
                at(0),
                DocumentOperation::index(operation, format!("d{operation}"), fields.clone()),
            );
            if operation == 20 {
                continue;
            }
            ledger.submit_request(Request::new(
                200 + operation,
                Endpoint::Index,
                at(0),
                at(0),
                fields
                    .iter()
                    .map(|field| RequestItem::new(operation, field.clone()))
                    .collect(),
            ));
            for field in &fields {
                ledger.record_item_result(200 + operation, operation, field, Outcome::Succeeded);
            }
            ledger.finish_request(
                200 + operation,
                Duration::from_millis(1),
                Outcome::Succeeded,
            );
        }
        ledger.record_query(
            at(0),
            at(0),
            Some(Duration::from_millis(1)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger
            .validate()
            .expect_err("a never-submitted field cannot hide in the allowed five percent");
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::MissingRequiredItemSubmissions(1)) }));
        assert!(!failures
            .iter()
            .any(|failure| matches!(failure, GateFailure::CompletionRateTooLow(..))));
    }

    #[test]
    fn a_scheduled_input_request_that_starts_in_drain_is_not_input_evidence() {
        let mut ledger = ledger(Endpoint::Replace, 1);
        ledger.begin_operation(at(0), DocumentOperation::replace(30, "backlogged"));
        ledger.submit_request(Request::new(
            300,
            Endpoint::Replace,
            at(0),
            at(1),
            vec![RequestItem::new(30, REPLACE_ITEM)],
        ));
        ledger.record_item_result(300, 30, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(300, at(2), Outcome::Succeeded);
        record_valid_non_document_evidence(&mut ledger);

        let failures = ledger
            .validate()
            .expect_err("the scheduled request did not reach the server during input");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::ScheduledInputRequestMissedInput(300))
            )
        }));
        assert!(failures
            .iter()
            .any(|failure| { matches!(failure, GateFailure::MissingRequiredItemSubmissions(1)) }));
    }

    #[test]
    fn an_underfilled_batch_before_a_later_full_batch_is_not_a_final_tail() {
        let mut limits = test_limits();
        limits.docops_per_second = 3;
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Replace, 2, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(1));
        for operation in 40..=42 {
            ledger.begin_operation(
                at(0),
                DocumentOperation::replace(operation, format!("d{operation}")),
            );
        }
        ledger.submit_request(Request::new(
            400,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(40, REPLACE_ITEM)],
        ));
        ledger.record_item_result(400, 40, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(400, at(1), Outcome::Succeeded);
        ledger.submit_request(Request::new(
            401,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![
                RequestItem::new(41, REPLACE_ITEM),
                RequestItem::new(42, REPLACE_ITEM),
            ],
        ));
        ledger.record_item_result(401, 41, REPLACE_ITEM, Outcome::Succeeded);
        ledger.record_item_result(401, 42, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(401, at(1), Outcome::Succeeded);
        ledger.record_query(
            at(0),
            at(0),
            Some(Duration::from_millis(1)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let failures = ledger
            .validate()
            .expect_err("only the final request may be a short batch");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::UnderfilledBatchBeforeFinalTail(
                    Endpoint::Replace
                ))
            )
        }));
    }

    #[test]
    fn late_full_window_burst_of_started_requests_is_rejected() {
        let mut limits = test_limits();
        limits.input_duration = at(180);
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Replace, 1, Backend::FlatCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(180));

        // The generator creates exactly one operation per second and every
        // request starts before the input edge. The aggregate counts therefore
        // look valid. A formed request held until second 179, however, exceeds
        // the existing 60-second drain/backlog allowance and cannot prove a
        // workload that covered the full measurement window.
        for operation in 0..180 {
            let scheduled_at = at(operation);
            ledger.begin_operation(
                scheduled_at,
                DocumentOperation::replace(operation + 1, format!("late-{operation}")),
            );
            ledger.submit_request(Request::new(
                operation + 1,
                Endpoint::Replace,
                scheduled_at,
                at(179),
                vec![RequestItem::new(operation + 1, REPLACE_ITEM)],
            ));
            ledger.record_item_result(
                operation + 1,
                operation + 1,
                REPLACE_ITEM,
                Outcome::Succeeded,
            );
            ledger.finish_request(
                operation + 1,
                at(179) + Duration::from_millis(1),
                Outcome::Succeeded,
            );
        }
        record_query_series(&mut ledger, 180);

        let failures = ledger
            .validate()
            .expect_err("late queued request bodies cannot substitute for a full-window workload");
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::RequestQueueDelayTooLong(
                    1,
                    observed,
                    allowed,
                )) if *observed == at(179) && *allowed == at(60)
            )
        }));
        assert!(failures.iter().any(|failure| {
            matches!(
                failure,
                GateFailure::Evidence(EvidenceFault::OperationBodyStartTooLate(
                    1,
                    observed,
                    allowed,
                )) if *observed == at(179) && *allowed == at(63)
            )
        }));
    }

    #[test]
    fn an_index_primary_cell_still_accounts_for_replace_and_unindex_work() {
        let mut limits = test_limits();
        limits.docops_per_second = 3;
        let mut ledger = WorkloadLedger::with_limits(
            WorkloadCase::new(Endpoint::Index, 1, Backend::HnswCpu),
            limits,
        );
        ledger.set_input_window(at(0), at(1));
        let fields = fields_14();
        ledger.begin_operation(at(0), DocumentOperation::index(50, "added", fields.clone()));
        ledger.begin_operation(at(0), DocumentOperation::replace(51, "updated"));
        ledger.begin_operation(at(0), DocumentOperation::unindex(52, "deleted"));
        for (offset, field) in fields.iter().enumerate() {
            let request = 500 + offset as u64;
            ledger.submit_request(Request::new(
                request,
                Endpoint::Index,
                at(0),
                at(0),
                vec![RequestItem::new(50, field.clone())],
            ));
            ledger.record_item_result(request, 50, field, Outcome::Succeeded);
            ledger.finish_request(request, Duration::from_millis(1), Outcome::Succeeded);
        }
        ledger.submit_request(Request::new(
            600,
            Endpoint::Replace,
            at(0),
            at(0),
            vec![RequestItem::new(51, REPLACE_ITEM)],
        ));
        ledger.record_item_result(600, 51, REPLACE_ITEM, Outcome::Succeeded);
        ledger.finish_request(600, Duration::from_millis(1), Outcome::Succeeded);
        ledger.submit_request(Request::new(
            601,
            Endpoint::Unindex,
            at(0),
            at(0),
            vec![RequestItem::new(52, UNINDEX_ITEM)],
        ));
        ledger.record_item_result(601, 52, UNINDEX_ITEM, Outcome::Succeeded);
        ledger.finish_request(601, Duration::from_millis(1), Outcome::Succeeded);
        ledger.record_classified_query(
            QueryClass::Hot,
            at(0),
            at(0),
            Some(Duration::from_millis(1)),
            Outcome::Succeeded,
        );
        ledger.record_classified_query(
            QueryClass::Idle,
            at(0),
            at(0),
            Some(Duration::from_millis(1)),
            Outcome::Succeeded,
        );
        ledger.record_checkpoint_completion();
        ledger.record_merge_completion();
        ledger.observe_peak_rss_bytes(1);

        let report = ledger
            .validate()
            .expect("the matrix primary must not discard update or delete operations");
        assert_eq!(report.docops_offered_in_input, 3);
        assert_eq!(report.docops_completed_in_input, 3);
    }
}
