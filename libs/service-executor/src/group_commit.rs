//! Bounded group commit with one timer, one batch execution, and result fan-out.

use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    sync::Arc,
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};

/// Limits owned by the shared group-commit runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupCommitConfig {
    pub max_delay: Duration,
    pub max_items: usize,
    pub max_bytes: usize,
    pub queue_capacity: usize,
}

impl GroupCommitConfig {
    pub fn new(
        max_delay: Duration,
        max_items: usize,
        max_bytes: usize,
    ) -> Result<Self, GroupCommitConfigError> {
        let config = Self {
            max_delay,
            max_items,
            max_bytes,
            queue_capacity: 1_024,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn with_queue_capacity(
        mut self,
        queue_capacity: usize,
    ) -> Result<Self, GroupCommitConfigError> {
        self.queue_capacity = queue_capacity;
        self.validate()?;
        Ok(self)
    }

    fn validate(self) -> Result<(), GroupCommitConfigError> {
        if self.max_delay.is_zero() {
            return Err(GroupCommitConfigError::ZeroMaxDelay);
        }
        if self.max_items == 0 {
            return Err(GroupCommitConfigError::ZeroMaxItems);
        }
        if self.max_bytes == 0 {
            return Err(GroupCommitConfigError::ZeroMaxBytes);
        }
        if self.queue_capacity == 0 {
            return Err(GroupCommitConfigError::ZeroQueueCapacity);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum GroupCommitConfigError {
    #[error("group-commit max delay must be positive")]
    ZeroMaxDelay,
    #[error("group-commit max item count must be positive")]
    ZeroMaxItems,
    #[error("group-commit max byte count must be positive")]
    ZeroMaxBytes,
    #[error("group-commit queue capacity must be positive")]
    ZeroQueueCapacity,
}

/// A domain request that can be flattened into one shared batch.
///
/// The domain owns its key and byte accounting. The runtime owns grouping,
/// timing, limits, execution, and response fan-out.
pub trait GroupCommitRequest: Send + 'static {
    type Item: Send + 'static;
    type Key: Clone + Eq + Send + 'static;

    fn key(&self) -> Self::Key;
    fn item_count(&self) -> usize;
    fn encoded_bytes(&self) -> usize;
    fn into_items(self) -> Vec<Self::Item>;
}

/// A submit-side failure. Sink errors use `Arc` so every request in the same
/// batch sees the same original failure without requiring it to be cloneable.
#[derive(Debug)]
pub enum GroupCommitError<E> {
    Closed,
    EmptyRequest,
    RequestTooLarge {
        items: usize,
        bytes: usize,
        max_items: usize,
        max_bytes: usize,
    },
    OutputCount {
        expected: usize,
        actual: usize,
    },
    Sink(Arc<E>),
}

impl<E: fmt::Display> fmt::Display for GroupCommitError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => formatter.write_str("group-commit worker is closed"),
            Self::EmptyRequest => formatter.write_str("group-commit request must not be empty"),
            Self::RequestTooLarge {
                items,
                bytes,
                max_items,
                max_bytes,
            } => write!(
                formatter,
                "group-commit request exceeds its limit: {items}/{max_items} items and {bytes}/{max_bytes} bytes"
            ),
            Self::OutputCount { expected, actual } => write!(
                formatter,
                "group-commit sink returned {actual} outputs for {expected} inputs"
            ),
            Self::Sink(error) => write!(formatter, "group-commit sink failed: {error}"),
        }
    }
}

impl<E> std::error::Error for GroupCommitError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync + 'static,
{
}

struct Pending<R, O, E> {
    request: R,
    reply: oneshot::Sender<Result<Vec<O>, GroupCommitError<E>>>,
}

/// Cloneable submit handle. Dropping every handle closes the worker after it
/// drains already accepted requests.
pub struct GroupCommitQueue<R, O, E> {
    sender: mpsc::Sender<Pending<R, O, E>>,
    config: GroupCommitConfig,
}

impl<R, O, E> Clone for GroupCommitQueue<R, O, E> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            config: self.config,
        }
    }
}

impl<R, O, E> GroupCommitQueue<R, O, E>
where
    R: GroupCommitRequest,
    O: Send + 'static,
    E: Send + Sync + 'static,
{
    pub async fn submit(&self, request: R) -> Result<Vec<O>, GroupCommitError<E>> {
        let items = request.item_count();
        let bytes = request.encoded_bytes();
        if items == 0 {
            return Err(GroupCommitError::EmptyRequest);
        }
        if items > self.config.max_items || bytes > self.config.max_bytes {
            return Err(GroupCommitError::RequestTooLarge {
                items,
                bytes,
                max_items: self.config.max_items,
                max_bytes: self.config.max_bytes,
            });
        }
        let (reply, response) = oneshot::channel();
        self.sender
            .send(Pending { request, reply })
            .await
            .map_err(|_| GroupCommitError::Closed)?;
        response.await.unwrap_or(Err(GroupCommitError::Closed))
    }
}

/// Join handle kept by the service that owns shutdown ordering.
pub struct GroupCommitWorker {
    task: tokio::task::JoinHandle<()>,
}

impl GroupCommitWorker {
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.task.await
    }
}

/// Start one group-commit runtime.
pub fn spawn_group_commit<R, O, E, F, Fut>(
    config: GroupCommitConfig,
    execute: F,
) -> (GroupCommitQueue<R, O, E>, GroupCommitWorker)
where
    R: GroupCommitRequest,
    O: Send + 'static,
    E: fmt::Debug + fmt::Display + Send + Sync + 'static,
    F: Fn(Vec<R::Item>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<O>, E>> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(config.queue_capacity);
    let task = tokio::spawn(run_group_commit(config, receiver, execute));
    (
        GroupCommitQueue { sender, config },
        GroupCommitWorker { task },
    )
}

async fn run_group_commit<R, O, E, F, Fut>(
    config: GroupCommitConfig,
    mut receiver: mpsc::Receiver<Pending<R, O, E>>,
    execute: F,
) where
    R: GroupCommitRequest,
    O: Send + 'static,
    E: fmt::Debug + fmt::Display + Send + Sync + 'static,
    F: Fn(Vec<R::Item>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<O>, E>> + Send + 'static,
{
    let mut carried = None;
    loop {
        let first = match carried.take() {
            Some(pending) => pending,
            None => match receiver.recv().await {
                Some(pending) => pending,
                None => return,
            },
        };
        let key = first.request.key();
        let mut item_count = first.request.item_count();
        let mut encoded_bytes = first.request.encoded_bytes();
        let mut requests = vec![first];
        let deadline = tokio::time::Instant::now() + config.max_delay;

        loop {
            let next = match tokio::time::timeout_at(deadline, receiver.recv()).await {
                Ok(Some(pending)) => pending,
                Ok(None) | Err(_) => break,
            };
            let next_items = next.request.item_count();
            let next_bytes = next.request.encoded_bytes();
            if next.request.key() != key
                || item_count.saturating_add(next_items) > config.max_items
                || encoded_bytes.saturating_add(next_bytes) > config.max_bytes
            {
                carried = Some(next);
                break;
            }
            item_count += next_items;
            encoded_bytes += next_bytes;
            requests.push(next);
        }

        let counts = requests
            .iter()
            .map(|pending| pending.request.item_count())
            .collect::<Vec<_>>();
        let mut replies = Vec::with_capacity(requests.len());
        let mut items = Vec::with_capacity(item_count);
        for pending in requests {
            items.extend(pending.request.into_items());
            replies.push(pending.reply);
        }

        match execute(items).await {
            Ok(outputs) if outputs.len() == item_count => {
                let mut outputs = VecDeque::from(outputs);
                for (count, reply) in counts.into_iter().zip(replies) {
                    let response = outputs.drain(..count).collect();
                    let _ = reply.send(Ok(response));
                }
            }
            Ok(outputs) => {
                let actual = outputs.len();
                for reply in replies {
                    let _ = reply.send(Err(GroupCommitError::OutputCount {
                        expected: item_count,
                        actual,
                    }));
                }
            }
            Err(error) => {
                let error = Arc::new(error);
                for reply in replies {
                    let _ = reply.send(Err(GroupCommitError::Sink(error.clone())));
                }
            }
        }
    }
}
