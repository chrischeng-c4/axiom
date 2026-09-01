//! Named task shutdown and error collection over the shared lifecycle.

use std::{
    fmt,
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::oneshot;

use crate::{
    DeadlineError, HookStage, LifecycleController, LifecycleError, ShutdownContext,
    ShutdownDeadline, ShutdownReport,
};

#[derive(Clone)]
pub struct TaskSupervisor {
    lifecycle: LifecycleController,
    total: Duration,
    reserve: Duration,
}

impl TaskSupervisor {
    pub fn new(total: Duration, reserve: Duration) -> Result<Self, DeadlineError> {
        ShutdownDeadline::from_now(total, reserve)?;
        Ok(Self {
            lifecycle: LifecycleController::serving(),
            total,
            reserve,
        })
    }

    pub fn lifecycle(&self) -> LifecycleController {
        self.lifecycle.clone()
    }

    pub fn register_hook<F, Fut>(
        &self,
        stage: HookStage,
        name: impl Into<String>,
        hook: F,
    ) -> Result<(), LifecycleError>
    where
        F: Fn(ShutdownContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        self.lifecycle.register_hook(stage, name, hook)
    }

    pub fn register_oneshot_task<T, E>(
        &self,
        stage: HookStage,
        name: impl Into<String>,
        shutdown: oneshot::Sender<()>,
        task: tokio::task::JoinHandle<Result<T, E>>,
    ) -> Result<(), LifecycleError>
    where
        T: Send + 'static,
        E: fmt::Display + Send + 'static,
    {
        let shutdown = Arc::new(Mutex::new(Some(shutdown)));
        let task = Arc::new(Mutex::new(Some(task)));
        self.register_hook(stage, name, move |_context| {
            let shutdown = shutdown.clone();
            let task = task.clone();
            async move {
                if let Some(shutdown) = shutdown
                    .lock()
                    .expect("supervised shutdown sender lock poisoned")
                    .take()
                {
                    let _ = shutdown.send(());
                }
                let task = task.lock().expect("supervised task lock poisoned").take();
                let Some(task) = task else {
                    return Ok(());
                };
                let mut task = AbortOnDrop::new(task);
                match task.join().await {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(error)) => Err(error.to_string()),
                    Err(error) if error.is_panic() => Err("supervised task panicked".to_string()),
                    Err(_) => Err("supervised task was cancelled".to_string()),
                }
            }
        })
    }

    pub async fn shutdown(
        &self,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Arc<ShutdownReport> {
        let deadline = ShutdownDeadline::from_now(self.total, self.reserve)
            .expect("TaskSupervisor validates its shutdown durations");
        self.lifecycle.shutdown(deadline, reason_code, detail).await
    }
}

struct AbortOnDrop<T> {
    task: Option<tokio::task::JoinHandle<T>>,
}

impl<T> AbortOnDrop<T> {
    fn new(task: tokio::task::JoinHandle<T>) -> Self {
        Self { task: Some(task) }
    }

    async fn join(&mut self) -> Result<T, tokio::task::JoinError> {
        let result = self.task.as_mut().expect("supervised task exists").await;
        self.task.take();
        result
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
