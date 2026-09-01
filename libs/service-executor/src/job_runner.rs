//! Blocking job execution with shared durable state transitions.

use std::{fmt, future::Future, panic::AssertUnwindSafe, sync::Arc};

use futures::FutureExt;

pub trait JobState<O>: Send + Sync + 'static {
    type Error: fmt::Display + Send + Sync + 'static;
    type Id: Clone + Send + Sync + 'static;

    fn mark_running(&self, id: &Self::Id) -> Result<(), Self::Error>;
    fn succeed(&self, id: &Self::Id, output: O) -> Result<(), Self::Error>;
    fn fail(&self, id: &Self::Id, message: String) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobRunState {
    Succeeded,
    Failed,
    PersistenceFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobRunReport {
    pub state: JobRunState,
    pub persistence_error: Option<String>,
}

pub struct JobRunner<S> {
    state: Arc<S>,
}

impl<S> Clone for JobRunner<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<S> JobRunner<S> {
    pub fn new(state: Arc<S>) -> Self {
        Self { state }
    }

    pub fn spawn_blocking<I, O, E, F>(
        &self,
        id: S::Id,
        input: I,
        execute: F,
    ) -> tokio::task::JoinHandle<JobRunReport>
    where
        S: JobState<O>,
        I: Send + 'static,
        O: Send + 'static,
        E: fmt::Display + Send + 'static,
        F: FnOnce(I) -> Result<O, E> + Send + 'static,
    {
        let state = self.state.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(error) = state.mark_running(&id) {
                return JobRunReport {
                    state: JobRunState::PersistenceFailed,
                    persistence_error: Some(error.to_string()),
                };
            }

            let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| execute(input)));
            match outcome {
                Ok(Ok(output)) => match state.succeed(&id, output) {
                    Ok(()) => JobRunReport {
                        state: JobRunState::Succeeded,
                        persistence_error: None,
                    },
                    Err(error) => JobRunReport {
                        state: JobRunState::PersistenceFailed,
                        persistence_error: Some(error.to_string()),
                    },
                },
                Ok(Err(error)) => persist_failure(&*state, &id, error.to_string()),
                Err(_) => persist_failure(&*state, &id, "job execution panicked".to_string()),
            }
        })
    }

    pub fn spawn_async<I, O, E, F, Fut>(
        &self,
        id: S::Id,
        input: I,
        execute: F,
    ) -> tokio::task::JoinHandle<JobRunReport>
    where
        S: JobState<O>,
        I: Send + 'static,
        O: Send + 'static,
        E: fmt::Display + Send + 'static,
        F: FnOnce(I) -> Fut + Send + 'static,
        Fut: Future<Output = Result<O, E>> + Send + 'static,
    {
        let state = self.state.clone();
        tokio::spawn(async move {
            if let Err(error) = state.mark_running(&id) {
                return JobRunReport {
                    state: JobRunState::PersistenceFailed,
                    persistence_error: Some(error.to_string()),
                };
            }

            let outcome = AssertUnwindSafe(execute(input)).catch_unwind().await;
            match outcome {
                Ok(Ok(output)) => match state.succeed(&id, output) {
                    Ok(()) => JobRunReport {
                        state: JobRunState::Succeeded,
                        persistence_error: None,
                    },
                    Err(error) => JobRunReport {
                        state: JobRunState::PersistenceFailed,
                        persistence_error: Some(error.to_string()),
                    },
                },
                Ok(Err(error)) => persist_failure(&*state, &id, error.to_string()),
                Err(_) => persist_failure(&*state, &id, "job execution panicked".to_string()),
            }
        })
    }
}

fn persist_failure<S, O>(state: &S, id: &S::Id, message: String) -> JobRunReport
where
    S: JobState<O>,
{
    match state.fail(id, message) {
        Ok(()) => JobRunReport {
            state: JobRunState::Failed,
            persistence_error: None,
        },
        Err(error) => JobRunReport {
            state: JobRunState::PersistenceFailed,
            persistence_error: Some(error.to_string()),
        },
    }
}
