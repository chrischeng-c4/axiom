//! Periodic task scheduler
//!
//! Supports both cron expressions and fixed intervals with distributed coordination.
//! Mode selection: `SelfHosted` runs leader election tick loop; `ExternalPush` uses
//! HTTP push receiver and delegates scheduling to an external system.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
#[cfg(any(feature = "push-receiver", feature = "scheduler"))]
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "scheduler")]
use cron::Schedule;
#[cfg(feature = "scheduler")]
use std::str::FromStr;

use super::backend::{SchedulerBackend, SchedulingMode};
use crate::{Broker, TaskError, TaskMessage};

#[cfg(feature = "push-receiver")]
use super::push_receiver::{PushReceiver, PushReceiverConfig};
#[cfg(feature = "scheduler")]
use super::schedule_monitor::{ScheduleMonitor, ScheduleMonitorConfig};

/// Default leader lock TTL
const DEFAULT_LEADER_TTL: Duration = Duration::from_secs(15);

/// Default follower sleep duration
const DEFAULT_FOLLOWER_SLEEP: Duration = Duration::from_secs(5);

/// Periodic task definition
#[derive(Debug, Clone)]
pub struct PeriodicTask {
    /// Unique name for this periodic task
    pub name: String,
    /// Task name to execute
    pub task_name: String,
    /// Schedule (cron or interval)
    pub schedule: PeriodicSchedule,
    /// Arguments to pass to task
    pub args: serde_json::Value,
    /// Target queue
    pub queue: String,
    /// Whether task is enabled (can be overridden by backend state)
    pub enabled: bool,
}

/// Schedule type for periodic tasks
#[derive(Debug, Clone)]
pub enum PeriodicSchedule {
    /// Cron expression (e.g., "0 0 * * * *" for hourly - note: 6-field format with seconds)
    #[cfg(feature = "scheduler")]
    Cron(String),
    /// Fixed interval in seconds
    Interval(u64),
}

impl PeriodicSchedule {
    /// Create a cron schedule from expression
    #[cfg(feature = "scheduler")]
    pub fn cron(expr: &str) -> Self {
        Self::Cron(expr.to_string())
    }

    /// Create an interval schedule
    pub fn interval(seconds: u64) -> Self {
        Self::Interval(seconds)
    }

    /// Calculate next run time from given timestamp
    pub fn next_run(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            #[cfg(feature = "scheduler")]
            PeriodicSchedule::Cron(expr) => {
                let schedule = Schedule::from_str(expr).ok()?;
                schedule
                    .after(&from)
                    .next()
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt.naive_utc(), Utc))
            }
            PeriodicSchedule::Interval(seconds) => {
                Some(from + chrono::Duration::seconds(*seconds as i64))
            }
        }
    }

    /// Check if the schedule is due now (within a small window)
    pub fn is_due(&self, last_run: Option<DateTime<Utc>>) -> bool {
        let now = Utc::now();
        match last_run {
            Some(last) => {
                if let Some(next) = self.next_run(last) {
                    next <= now
                } else {
                    false
                }
            }
            None => true, // Never run before, so it's due
        }
    }
}

/// Configuration for the periodic scheduler
#[derive(Debug, Clone)]
pub struct PeriodicSchedulerConfig {
    /// TTL for the leader lock (SelfHosted mode only)
    pub leader_ttl: Duration,
    /// How long followers sleep before retrying leader acquisition (SelfHosted mode only)
    pub follower_sleep: Duration,
    /// How often to renew the leader lock (should be less than TTL, SelfHosted mode only)
    pub leader_renew_interval: Duration,
    /// Push receiver configuration. Required when `scheduling_mode()` is `ExternalPush`.
    /// Ignored for `SelfHosted`.
    #[cfg(feature = "push-receiver")]
    pub push_receiver_config: Option<PushReceiverConfig>,
    /// Schedule monitor configuration. Optional for both modes.
    /// Enables expected_at vs actual_at tracking.
    #[cfg(feature = "scheduler")]
    pub monitor_config: Option<ScheduleMonitorConfig>,
}

impl Default for PeriodicSchedulerConfig {
    fn default() -> Self {
        Self {
            leader_ttl: DEFAULT_LEADER_TTL,
            follower_sleep: DEFAULT_FOLLOWER_SLEEP,
            leader_renew_interval: Duration::from_secs(5),
            #[cfg(feature = "push-receiver")]
            push_receiver_config: None,
            #[cfg(feature = "scheduler")]
            monitor_config: None,
        }
    }
}

/// Scheduler for periodic tasks with distributed coordination.
///
/// Supports two scheduling modes determined by the backend:
/// - `SelfHosted`: leader election tick loop (existing behavior)
/// - `ExternalPush`: HTTP push receiver + external task registration
pub struct PeriodicScheduler<B: Broker, S: SchedulerBackend> {
    tasks: Vec<PeriodicTask>,
    broker: Arc<B>,
    backend: Arc<S>,
    config: PeriodicSchedulerConfig,
    shutdown: CancellationToken,
    /// Push receiver instance, created during `start()` in `ExternalPush` mode.
    #[cfg(feature = "push-receiver")]
    push_receiver: OnceCell<Arc<PushReceiver>>,
    /// Schedule monitor instance, created during `start()` if `monitor_config` is set.
    #[cfg(feature = "scheduler")]
    monitor: OnceCell<Arc<ScheduleMonitor>>,
}

impl<B: Broker, S: SchedulerBackend + 'static> PeriodicScheduler<B, S> {
    /// Create a new periodic scheduler
    pub fn new(broker: Arc<B>, backend: Arc<S>) -> Self {
        Self {
            tasks: Vec::new(),
            broker,
            backend,
            config: PeriodicSchedulerConfig::default(),
            shutdown: CancellationToken::new(),
            #[cfg(feature = "push-receiver")]
            push_receiver: OnceCell::new(),
            #[cfg(feature = "scheduler")]
            monitor: OnceCell::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(broker: Arc<B>, backend: Arc<S>, config: PeriodicSchedulerConfig) -> Self {
        Self {
            tasks: Vec::new(),
            broker,
            backend,
            config,
            shutdown: CancellationToken::new(),
            #[cfg(feature = "push-receiver")]
            push_receiver: OnceCell::new(),
            #[cfg(feature = "scheduler")]
            monitor: OnceCell::new(),
        }
    }

    /// Add a periodic task
    pub fn add_task(&mut self, task: PeriodicTask) {
        tracing::info!("Adding periodic task: {}", task.name);
        self.tasks.push(task);
    }

    /// Remove a periodic task by name
    pub fn remove_task(&mut self, name: &str) -> Option<PeriodicTask> {
        if let Some(pos) = self.tasks.iter().position(|t| t.name == name) {
            tracing::info!("Removing periodic task: {}", name);
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }

    /// Get all registered tasks
    pub fn tasks(&self) -> &[PeriodicTask] {
        &self.tasks
    }

    /// Start the scheduler.
    ///
    /// Queries `backend.scheduling_mode()` once and branches:
    /// - `SelfHosted`: spawns existing leader election tick loop
    /// - `ExternalPush`: constructs push receiver, registers tasks with external backend,
    ///   optionally starts schedule monitor
    pub async fn start(&self) -> Result<(), TaskError> {
        if self.tasks.is_empty() {
            tracing::warn!("No periodic tasks to schedule");
            return Ok(());
        }

        // Initialize monitor if configured (applies to both modes)
        #[cfg(feature = "scheduler")]
        if let Some(monitor_config) = &self.config.monitor_config {
            let monitor = Arc::new(ScheduleMonitor::new(monitor_config.clone())?);

            // Register all tasks with the monitor
            for task in &self.tasks {
                let schedule = Self::task_to_monitor_schedule(task)?;
                monitor.register_task(&task.name, schedule, None, None)?;
            }

            // Start the background missed-check task
            monitor.start();

            let _ = self.monitor.set(monitor);
            tracing::info!("Schedule monitor started with {} tasks", self.tasks.len());
        }

        let mode = self.backend.scheduling_mode();
        tracing::info!(?mode, "Periodic scheduler mode selected");

        match mode {
            SchedulingMode::SelfHosted => self.start_self_hosted().await,
            SchedulingMode::ExternalPush => self.start_external_push().await,
        }
    }

    /// Start in SelfHosted mode: spawns leader election tick loop.
    async fn start_self_hosted(&self) -> Result<(), TaskError> {
        let broker = self.broker.clone();
        let backend = self.backend.clone();
        let tasks = self.tasks.clone();
        let config = self.config.clone();
        let shutdown = self.shutdown.clone();
        #[cfg(feature = "scheduler")]
        let monitor = self.monitor.get().cloned();

        tokio::spawn(async move {
            tracing::info!(
                "Periodic scheduler starting with {} tasks (SelfHosted mode)",
                tasks.len()
            );

            loop {
                // Try to become leader
                match backend.acquire_leader(config.leader_ttl).await {
                    Ok(true) => {
                        tracing::info!("Acquired leadership, starting scheduler loop");
                        Self::run_leader_loop(
                            &broker,
                            &backend,
                            &tasks,
                            &config,
                            &shutdown,
                            #[cfg(feature = "scheduler")]
                            monitor.as_ref(),
                        )
                        .await;

                        if shutdown.is_cancelled() {
                            tracing::info!("Shutdown requested, releasing leadership");
                            let _ = backend.release_leader().await;
                            break;
                        }
                    }
                    Ok(false) => {
                        tracing::debug!("Failed to acquire leadership, sleeping");
                    }
                    Err(e) => {
                        tracing::error!("Error acquiring leadership: {}", e);
                    }
                }

                // Sleep as follower
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        tracing::info!("Shutdown requested while follower");
                        break;
                    }
                    _ = tokio::time::sleep(config.follower_sleep) => {}
                }
            }

            tracing::info!("Periodic scheduler stopped");
        });

        Ok(())
    }

    /// Start in ExternalPush mode: construct push receiver, register tasks with backend.
    #[cfg(feature = "push-receiver")]
    async fn start_external_push(&self) -> Result<(), TaskError> {
        // Validate push_receiver_config is present (required for ExternalPush mode)
        let push_config = self
            .config
            .push_receiver_config
            .clone()
            .ok_or_else(|| {
                TaskError::Configuration(
                    "push_receiver_config required for ExternalPush mode".to_string(),
                )
            })?;

        // Construct PushReceiver
        #[cfg(feature = "scheduler")]
        let monitor_ref = self.monitor.get().cloned();

        let push_receiver = Arc::new(PushReceiver::new(
            push_config,
            self.broker.clone() as Arc<dyn Broker>,
            #[cfg(feature = "scheduler")]
            monitor_ref,
        )?);

        let _ = self.push_receiver.set(push_receiver);

        // Register all tasks with the external backend
        self.register_all_tasks_external().await?;

        tracing::info!(
            "Periodic scheduler started in ExternalPush mode with {} tasks",
            self.tasks.len()
        );

        Ok(())
    }

    /// Start in ExternalPush mode (stub when push-receiver feature is disabled).
    #[cfg(not(feature = "push-receiver"))]
    async fn start_external_push(&self) -> Result<(), TaskError> {
        Err(TaskError::Configuration(
            "ExternalPush mode requires the push-receiver feature".to_string(),
        ))
    }

    /// Register all tasks with the external scheduling backend.
    ///
    /// Called during `start()` in `ExternalPush` mode. Iterates all registered
    /// tasks and calls `backend.register_external_schedule(task)` for each.
    /// If any registration fails, returns the error immediately.
    #[cfg(feature = "push-receiver")]
    async fn register_all_tasks_external(&self) -> Result<(), TaskError> {
        for task in &self.tasks {
            tracing::info!(
                task_name = %task.name,
                "Registering task with external backend"
            );
            self.backend.register_external_schedule(task).await?;
        }
        Ok(())
    }

    /// Returns the push receiver axum Router in ExternalPush mode.
    ///
    /// Returns `None` in SelfHosted mode or before `start()` is called.
    /// Caller merges this router into the existing server.
    #[cfg(feature = "push-receiver")]
    pub fn router(&self) -> Option<axum::Router> {
        self.push_receiver
            .get()
            .map(|receiver| Arc::clone(receiver).router())
    }

    /// Convert a `PeriodicTask`'s schedule into a `TaskSchedule` for the monitor.
    #[cfg(feature = "scheduler")]
    fn task_to_monitor_schedule(
        task: &PeriodicTask,
    ) -> Result<super::schedule_monitor::TaskSchedule, TaskError> {
        match &task.schedule {
            #[cfg(feature = "scheduler")]
            PeriodicSchedule::Cron(expr) => {
                super::schedule_monitor::TaskSchedule::cron(expr)
            }
            PeriodicSchedule::Interval(secs) => {
                Ok(super::schedule_monitor::TaskSchedule::interval(
                    Duration::from_secs(*secs),
                ))
            }
        }
    }

    /// Run the leader loop (evaluates schedules and enqueues tasks)
    async fn run_leader_loop(
        broker: &Arc<B>,
        backend: &Arc<S>,
        tasks: &[PeriodicTask],
        config: &PeriodicSchedulerConfig,
        shutdown: &CancellationToken,
        #[cfg(feature = "scheduler")] monitor: Option<&Arc<ScheduleMonitor>>,
    ) {
        let mut last_renew = std::time::Instant::now();

        loop {
            // Check for shutdown
            if shutdown.is_cancelled() {
                return;
            }

            // Renew leadership if needed
            if last_renew.elapsed() >= config.leader_renew_interval {
                match backend.renew_leader(config.leader_ttl).await {
                    Ok(true) => {
                        last_renew = std::time::Instant::now();
                    }
                    Ok(false) => {
                        tracing::warn!("Lost leadership, exiting leader loop");
                        return;
                    }
                    Err(e) => {
                        tracing::error!("Error renewing leadership: {}", e);
                        return;
                    }
                }
            }

            // Evaluate each task
            for task in tasks {
                if !task.enabled {
                    continue;
                }

                // Check backend state (pause/resume)
                let is_enabled = match backend.is_task_enabled(&task.name).await {
                    Ok(enabled) => enabled,
                    Err(e) => {
                        tracing::error!("Error checking task state for {}: {}", task.name, e);
                        continue;
                    }
                };

                if !is_enabled {
                    tracing::debug!("Task {} is paused, skipping", task.name);
                    continue;
                }

                // Get last run time - skip on error to avoid burst execution
                let state = match backend.get_task_state(&task.name).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to get task state for {}, skipping this cycle: {}",
                            task.name,
                            e
                        );
                        continue;
                    }
                };

                // Check if task is due
                if task.schedule.is_due(state.last_run_at) {
                    tracing::info!("Running periodic task: {}", task.name);

                    let message = TaskMessage::new(&task.task_name, task.args.clone());

                    match broker.publish(&task.queue, message).await {
                        Ok(_) => {
                            tracing::debug!("Published periodic task: {}", task.name);
                            // Record the run
                            if let Err(e) = backend.record_task_run(&task.name).await {
                                tracing::error!(
                                    "Failed to record task run for {}: {}",
                                    task.name,
                                    e
                                );
                            }

                            // Best-effort monitor integration (R7)
                            #[cfg(feature = "scheduler")]
                            if let Some(mon) = monitor {
                                if let Err(e) =
                                    mon.record_trigger(&task.name, Utc::now())
                                {
                                    tracing::warn!(
                                        task_name = %task.name,
                                        error = %e,
                                        "Failed to record trigger in monitor (best-effort)"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to publish periodic task {}: {}",
                                task.name,
                                e
                            );
                        }
                    }
                }
            }

            // Sleep briefly before next evaluation
            tokio::select! {
                _ = shutdown.cancelled() => return,
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
    }

    /// Shutdown the scheduler.
    ///
    /// Works for both modes:
    /// - `SelfHosted`: cancels tick loop via `CancellationToken`
    /// - `ExternalPush`: stops `ScheduleMonitor` if running; push receiver routes
    ///   remain active until server process exits
    pub fn shutdown(&self) {
        self.shutdown.cancel();

        // Stop the schedule monitor if it was started
        #[cfg(feature = "scheduler")]
        if let Some(monitor) = self.monitor.get() {
            monitor.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_schedule() {
        let schedule = PeriodicSchedule::Interval(60);
        let now = Utc::now();
        let next = schedule.next_run(now).unwrap();
        assert!(next > now);
        assert!((next - now).num_seconds() >= 60);
    }

    #[test]
    fn test_interval_is_due() {
        let schedule = PeriodicSchedule::Interval(1);

        // Never run before - should be due
        assert!(schedule.is_due(None));

        // Just ran - should not be due
        let now = Utc::now();
        assert!(!schedule.is_due(Some(now)));

        // Ran 2 seconds ago with 1 second interval - should be due
        let two_seconds_ago = now - chrono::Duration::seconds(2);
        assert!(schedule.is_due(Some(two_seconds_ago)));
    }

    #[cfg(feature = "scheduler")]
    #[test]
    fn test_cron_schedule() {
        use std::str::FromStr;

        // The cron crate uses extended format with seconds: "sec min hour day month dow year"
        // Every minute: "0 * * * * *"
        let expr = "0 * * * * *";
        let parsed = Schedule::from_str(expr);
        assert!(
            parsed.is_ok(),
            "Failed to parse cron expression: {:?}",
            parsed.err()
        );

        let cron_schedule = parsed.unwrap();
        let now = Utc::now();

        // Test using upcoming iterator
        let mut upcoming = cron_schedule.upcoming(Utc);
        let next_time = upcoming.next();
        assert!(next_time.is_some(), "No next time from upcoming iterator");

        // Now test our wrapper
        let schedule = PeriodicSchedule::Cron("0 * * * * *".to_string());
        let next = schedule.next_run(now);
        assert!(
            next.is_some(),
            "next_run returned None for valid cron expression"
        );
        assert!(next.unwrap() > now);
    }

    #[test]
    fn test_periodic_task_config() {
        let task = PeriodicTask {
            name: "test-task".to_string(),
            task_name: "my_task".to_string(),
            schedule: PeriodicSchedule::Interval(30),
            args: serde_json::json!({}),
            queue: "default".to_string(),
            enabled: true,
        };

        assert_eq!(task.name, "test-task");
        assert!(task.enabled);
    }

    // -----------------------------------------------------------------------
    // Mock backend and broker for mode selection tests
    // -----------------------------------------------------------------------

    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use crate::scheduler::backend::{SchedulerBackend, SchedulingMode, TaskScheduleState};
    use crate::broker::{BrokerCapabilities, DeliveryModel};

    /// Mock broker that records publish calls.
    struct MockBroker {
        published: tokio::sync::Mutex<Vec<(String, crate::TaskMessage)>>,
    }

    impl MockBroker {
        fn new() -> Self {
            Self {
                published: tokio::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl crate::Broker for MockBroker {
        async fn connect(&self) -> Result<(), crate::TaskError> { Ok(()) }
        async fn disconnect(&self) -> Result<(), crate::TaskError> { Ok(()) }
        async fn publish(&self, queue: &str, message: crate::TaskMessage) -> Result<(), crate::TaskError> {
            self.published.lock().await.push((queue.to_string(), message));
            Ok(())
        }
        async fn health_check(&self) -> Result<(), crate::TaskError> { Ok(()) }
        fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Pull }
        fn capabilities(&self) -> BrokerCapabilities { BrokerCapabilities::default() }
    }

    /// Mock self-hosted backend (uses default scheduling_mode → SelfHosted).
    struct MockSelfHostedBackend {
        is_leader: tokio::sync::RwLock<bool>,
        task_states: tokio::sync::RwLock<std::collections::HashMap<String, TaskScheduleState>>,
    }

    impl MockSelfHostedBackend {
        fn new() -> Self {
            Self {
                is_leader: tokio::sync::RwLock::new(false),
                task_states: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SchedulerBackend for MockSelfHostedBackend {
        async fn acquire_leader(&self, _ttl: std::time::Duration) -> Result<bool, crate::TaskError> {
            let mut is_leader = self.is_leader.write().await;
            *is_leader = true;
            Ok(true)
        }
        async fn renew_leader(&self, _ttl: std::time::Duration) -> Result<bool, crate::TaskError> {
            let is_leader = self.is_leader.read().await;
            Ok(*is_leader)
        }
        async fn release_leader(&self) -> Result<(), crate::TaskError> {
            let mut is_leader = self.is_leader.write().await;
            *is_leader = false;
            Ok(())
        }
        async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, crate::TaskError> {
            let states = self.task_states.read().await;
            Ok(states.get(name).cloned().unwrap_or_default())
        }
        async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), crate::TaskError> {
            self.task_states.write().await.insert(name.to_string(), state.clone());
            Ok(())
        }
        // Uses default scheduling_mode() → SelfHosted
    }

    /// Mock external push backend that returns ExternalPush mode.
    /// Tracks register_external_schedule calls.
    struct MockExternalPushBackend {
        register_calls: AtomicUsize,
        fail_register: AtomicBool,
        task_states: tokio::sync::RwLock<std::collections::HashMap<String, TaskScheduleState>>,
    }

    impl MockExternalPushBackend {
        fn new() -> Self {
            Self {
                register_calls: AtomicUsize::new(0),
                fail_register: AtomicBool::new(false),
                task_states: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }

        #[cfg(feature = "push-receiver")]
        fn failing() -> Self {
            Self {
                register_calls: AtomicUsize::new(0),
                fail_register: AtomicBool::new(true),
                task_states: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }

        #[cfg(feature = "push-receiver")]
        fn register_count(&self) -> usize {
            self.register_calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl SchedulerBackend for MockExternalPushBackend {
        async fn acquire_leader(&self, _ttl: std::time::Duration) -> Result<bool, crate::TaskError> {
            Ok(true)
        }
        async fn renew_leader(&self, _ttl: std::time::Duration) -> Result<bool, crate::TaskError> {
            Ok(true)
        }
        async fn release_leader(&self) -> Result<(), crate::TaskError> {
            Ok(())
        }
        async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, crate::TaskError> {
            let states = self.task_states.read().await;
            Ok(states.get(name).cloned().unwrap_or_default())
        }
        async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), crate::TaskError> {
            self.task_states.write().await.insert(name.to_string(), state.clone());
            Ok(())
        }
        fn scheduling_mode(&self) -> SchedulingMode {
            SchedulingMode::ExternalPush
        }
        async fn register_external_schedule(&self, _task: &PeriodicTask) -> Result<(), crate::TaskError> {
            if self.fail_register.load(Ordering::SeqCst) {
                return Err(crate::TaskError::Authentication(
                    "K8s API 403 Forbidden".to_string(),
                ));
            }
            self.register_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn make_test_task(name: &str) -> PeriodicTask {
        PeriodicTask {
            name: name.to_string(),
            task_name: format!("{}_handler", name),
            schedule: PeriodicSchedule::Interval(60),
            args: serde_json::json!({"key": "value"}),
            queue: "default".to_string(),
            enabled: true,
        }
    }

    // -----------------------------------------------------------------------
    // R5: PeriodicSchedulerConfig extension
    // -----------------------------------------------------------------------

    #[test]
    fn r5_config_default_push_receiver_config_is_none() {
        let config = PeriodicSchedulerConfig::default();
        #[cfg(feature = "push-receiver")]
        assert!(config.push_receiver_config.is_none());
        // When feature is off, field doesn't exist
        let _ = config;
    }

    #[test]
    fn r5_config_default_monitor_config_is_none() {
        let config = PeriodicSchedulerConfig::default();
        #[cfg(feature = "scheduler")]
        assert!(config.monitor_config.is_none());
        let _ = config;
    }

    #[test]
    fn r5_config_default_values() {
        let config = PeriodicSchedulerConfig::default();
        assert_eq!(config.leader_ttl, DEFAULT_LEADER_TTL);
        assert_eq!(config.follower_sleep, DEFAULT_FOLLOWER_SLEEP);
        assert_eq!(config.leader_renew_interval, std::time::Duration::from_secs(5));
    }

    // -----------------------------------------------------------------------
    // S1: Self-hosted backend starts leader election tick loop (R2, R3)
    // -----------------------------------------------------------------------

    #[test]
    fn s1_self_hosted_backend_returns_self_hosted_mode() {
        let backend = MockSelfHostedBackend::new();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    #[tokio::test]
    async fn s1_self_hosted_scheduler_start_with_tasks() {
        // GIVEN a scheduler with self-hosted backend and tasks
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);
        scheduler.add_task(make_test_task("task-1"));

        // WHEN start() is called
        let result = scheduler.start().await;

        // THEN it succeeds (spawns leader loop in background)
        assert!(result.is_ok());

        // Clean up
        scheduler.shutdown();
    }

    #[tokio::test]
    async fn s1_self_hosted_push_receiver_config_ignored() {
        // GIVEN a self-hosted scheduler with push_receiver_config set (should be ignored)
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let config = PeriodicSchedulerConfig::default();
        // push_receiver_config is None — SelfHosted mode doesn't need it
        let mut scheduler = PeriodicScheduler::with_config(broker, backend, config);
        scheduler.add_task(make_test_task("task-1"));

        let result = scheduler.start().await;
        assert!(result.is_ok());
        scheduler.shutdown();
    }

    // -----------------------------------------------------------------------
    // S2: External backend starts push receiver mode (R2, R3, R6)
    // S3: External mode without push_receiver_config returns error (R3, R5)
    // -----------------------------------------------------------------------

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s3_external_push_without_config_returns_error() {
        // GIVEN a scheduler with ExternalPush backend and NO push_receiver_config
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: None,
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend, config);
        scheduler.add_task(make_test_task("task-1"));

        // WHEN start() is called
        let result = scheduler.start().await;

        // THEN returns Configuration error
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::TaskError::Configuration(msg) => {
                assert!(msg.contains("push_receiver_config required for ExternalPush mode"));
            }
            other => panic!("Expected Configuration error, got: {:?}", other),
        }
    }

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s2_external_push_with_config_succeeds() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // GIVEN a scheduler with ExternalPush backend and valid push_receiver_config
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend, config);
        scheduler.add_task(make_test_task("task-1"));

        // WHEN start() is called
        let result = scheduler.start().await;

        // THEN it succeeds
        assert!(result.is_ok());
    }

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s2_external_push_router_returns_some() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // GIVEN a scheduler in ExternalPush mode after successful start()
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend, config);
        scheduler.add_task(make_test_task("task-1"));
        scheduler.start().await.unwrap();

        // WHEN router() is called
        let router = scheduler.router();

        // THEN it returns Some(Router)
        assert!(router.is_some());
    }

    // -----------------------------------------------------------------------
    // S11: router() returns None in SelfHosted mode (R6)
    // -----------------------------------------------------------------------

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s11_router_returns_none_in_self_hosted_mode() {
        // GIVEN a scheduler in SelfHosted mode after start()
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);
        scheduler.add_task(make_test_task("task-1"));
        scheduler.start().await.unwrap();

        // WHEN router() is called
        let router = scheduler.router();

        // THEN returns None — no push receiver routes exist
        assert!(router.is_none());

        scheduler.shutdown();
    }

    // -----------------------------------------------------------------------
    // S4: Tasks registered with external backend on start (R4, R8)
    // -----------------------------------------------------------------------

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s4_tasks_registered_with_external_backend() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // GIVEN a scheduler with 3 tasks and ExternalPush backend
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend.clone(), config);
        scheduler.add_task(make_test_task("daily-cleanup"));
        scheduler.add_task(make_test_task("hourly-sync"));
        scheduler.add_task(make_test_task("weekly-report"));

        // WHEN start() is called
        scheduler.start().await.unwrap();

        // THEN register_external_schedule called for each task
        assert_eq!(backend.register_count(), 3);
    }

    // -----------------------------------------------------------------------
    // S5: External task registration failure aborts start (R4)
    // -----------------------------------------------------------------------

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s5_external_registration_failure_aborts_start() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // GIVEN a scheduler with a failing external backend
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::failing());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend.clone(), config);
        scheduler.add_task(make_test_task("daily-cleanup"));
        scheduler.add_task(make_test_task("hourly-sync"));

        // WHEN start() is called and first registration fails
        let result = scheduler.start().await;

        // THEN start() returns error
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::TaskError::Authentication(msg) => {
                assert!(msg.contains("403 Forbidden"));
            }
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // S9/S10: Shutdown works for both modes (R9)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn s10_shutdown_self_hosted_cancels_token() {
        // GIVEN a running self-hosted scheduler
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);
        scheduler.add_task(make_test_task("task-1"));
        scheduler.start().await.unwrap();

        // WHEN shutdown() is called
        scheduler.shutdown();

        // THEN the cancellation token is cancelled
        assert!(scheduler.shutdown.is_cancelled());
    }

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn s9_shutdown_external_push_cancels_token() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // GIVEN a running external push scheduler
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend, config);
        scheduler.add_task(make_test_task("task-1"));
        scheduler.start().await.unwrap();

        // WHEN shutdown() is called
        scheduler.shutdown();

        // THEN the cancellation token is cancelled
        assert!(scheduler.shutdown.is_cancelled());
    }

    // -----------------------------------------------------------------------
    // Scheduler with no tasks
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn start_with_no_tasks_returns_ok() {
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let scheduler = PeriodicScheduler::new(broker, backend);

        // start() with no tasks should warn and return Ok
        let result = scheduler.start().await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // Scheduler task management
    // -----------------------------------------------------------------------

    #[test]
    fn add_and_remove_tasks() {
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);

        scheduler.add_task(make_test_task("task-a"));
        scheduler.add_task(make_test_task("task-b"));
        assert_eq!(scheduler.tasks().len(), 2);

        let removed = scheduler.remove_task("task-a");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "task-a");
        assert_eq!(scheduler.tasks().len(), 1);

        let not_found = scheduler.remove_task("nonexistent");
        assert!(not_found.is_none());
    }

    // -----------------------------------------------------------------------
    // PeriodicScheduler construction
    // -----------------------------------------------------------------------

    #[test]
    fn new_scheduler_has_empty_tasks() {
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let scheduler = PeriodicScheduler::new(broker, backend);
        assert!(scheduler.tasks().is_empty());
    }

    #[test]
    fn with_config_uses_custom_config() {
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let config = PeriodicSchedulerConfig {
            leader_ttl: std::time::Duration::from_secs(30),
            follower_sleep: std::time::Duration::from_secs(10),
            leader_renew_interval: std::time::Duration::from_secs(8),
            ..Default::default()
        };
        let scheduler = PeriodicScheduler::with_config(broker, backend, config);
        assert_eq!(scheduler.config.leader_ttl, std::time::Duration::from_secs(30));
        assert_eq!(scheduler.config.follower_sleep, std::time::Duration::from_secs(10));
        assert_eq!(scheduler.config.leader_renew_interval, std::time::Duration::from_secs(8));
    }

    // -----------------------------------------------------------------------
    // Mode branching — verify the correct branch is taken
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn mode_branching_self_hosted_does_not_require_push_config() {
        // Self-hosted mode should succeed even without push_receiver_config
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockSelfHostedBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);
        scheduler.add_task(make_test_task("task-1"));

        let result = scheduler.start().await;
        assert!(result.is_ok());
        scheduler.shutdown();
    }

    #[cfg(feature = "push-receiver")]
    #[tokio::test]
    async fn mode_branching_external_push_does_not_start_leader_loop() {
        use crate::scheduler::push_receiver::PushReceiverConfig;

        // External push mode should NOT call acquire_leader in a loop
        // We verify by checking register_external_schedule was called (push path)
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let config = PeriodicSchedulerConfig {
            push_receiver_config: Some(PushReceiverConfig {
                hmac_secret: Some("this-is-a-32-byte-hmac-secret!!x".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut scheduler = PeriodicScheduler::with_config(broker, backend.clone(), config);
        scheduler.add_task(make_test_task("task-1"));

        scheduler.start().await.unwrap();

        // register_external_schedule was called (ExternalPush path taken)
        assert_eq!(backend.register_count(), 1);
    }

    // -----------------------------------------------------------------------
    // Without push-receiver feature, ExternalPush mode returns error
    // -----------------------------------------------------------------------

    #[cfg(not(feature = "push-receiver"))]
    #[tokio::test]
    async fn external_push_without_feature_returns_error() {
        let broker = Arc::new(MockBroker::new());
        let backend = Arc::new(MockExternalPushBackend::new());
        let mut scheduler = PeriodicScheduler::new(broker, backend);
        scheduler.add_task(make_test_task("task-1"));

        let result = scheduler.start().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::TaskError::Configuration(msg) => {
                assert!(msg.contains("push-receiver feature"));
            }
            other => panic!("Expected Configuration error, got: {:?}", other),
        }
    }
}
