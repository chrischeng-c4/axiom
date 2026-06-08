# Tasks: improve-meteor-scheduler

## 1. Scheduler Core

- [ ] **1.1** Add dependencies
  - File: `crates/cclab-meteor/Cargo.toml` (MODIFY)
  - Do: Add `cron` crate for crontab parsing. Ensure `cclab-ion-client` is available.

- [ ] **1.2** Implement ScheduleType enum
  - File: `crates/cclab-meteor/src/scheduler/schedule.rs` (CREATE)
  - Do: Add `ScheduleType` enum with `Cron(String)` and `Interval(Duration)` variants. Implement `is_due()` method.

- [ ] **1.3** Implement PeriodicTaskConfig
  - File: `crates/cclab-meteor/src/scheduler/config.rs` (CREATE)
  - Do: Define `PeriodicTaskConfig` struct with name, task, args, kwargs, schedule, options.

## 2. Backend Abstraction

- [ ] **2.1** Define SchedulerBackend trait
  - File: `crates/cclab-meteor/src/scheduler/backend.rs` (CREATE)
  - Do: Define trait with `acquire_leader()`, `renew_leader()`, `get_task_state()`, `set_task_state()`.

- [ ] **2.2** Implement IonSchedulerBackend
  - File: `crates/cclab-meteor/src/scheduler/ion_backend.rs` (CREATE)
  - Do: Implement `SchedulerBackend` using `cclab-ion-client`. Use `lock()` for leader election.

- [ ] **2.3** Implement InMemoryBackend (for testing)
  - File: `crates/cclab-meteor/src/scheduler/memory_backend.rs` (CREATE)
  - Do: Simple in-memory implementation for unit tests.

## 3. Scheduler Loop

- [ ] **3.1** Implement PeriodicScheduler
  - File: `crates/cclab-meteor/src/scheduler/periodic.rs` (CREATE)
  - Do: Main scheduler loop with leader election, schedule evaluation, task enqueue.

- [ ] **3.2** Leader election loop
  - File: `crates/cclab-meteor/src/scheduler/periodic.rs` (MODIFY)
  - Do: Implement loop that acquires/renews leader lock, or sleeps as follower.

- [ ] **3.3** Schedule evaluation
  - File: `crates/cclab-meteor/src/scheduler/periodic.rs` (MODIFY)
  - Do: Check each task's `is_due()`, respect paused state, enqueue if due.

## 4. CLI Commands

- [ ] **4.1** Add schedule subcommand to CLI
  - File: `crates/cclab-cli/src/meteor.rs` (MODIFY or CREATE)
  - Do: Add `cc meteor schedule` subcommand group.

- [ ] **4.2** Implement list command
  - File: `crates/cclab-cli/src/meteor.rs` (MODIFY)
  - Do: `cc meteor schedule list` - show all registered tasks with status.

- [ ] **4.3** Implement pause/resume commands
  - File: `crates/cclab-cli/src/meteor.rs` (MODIFY)
  - Do: `cc meteor schedule pause/resume <name>` - toggle task enabled state.

- [ ] **4.4** Implement trigger command
  - File: `crates/cclab-cli/src/meteor.rs` (MODIFY)
  - Do: `cc meteor schedule trigger <name>` - manually enqueue task now.

## 5. Python Bindings

- [ ] **5.1** Export PeriodicTask class
  - File: `crates/cclab-nucleus/src/meteor.rs` (MODIFY)
  - Do: PyO3 bindings for `PeriodicTask`, `crontab()`, `schedule()`.

- [ ] **5.2** Support METEOR_SCHEDULE config
  - File: `crates/cclab-nucleus/src/meteor.rs` (MODIFY)
  - Do: Allow Python dict config like Celery's `CELERYBEAT_SCHEDULE`.

## 6. Tests

- [ ] **6.1** Unit tests for ScheduleType
  - File: `crates/cclab-meteor/src/scheduler/schedule.rs` (MODIFY)
  - Do: Test crontab parsing, interval parsing, `is_due()` logic.

- [ ] **6.2** Unit tests for leader election
  - File: `crates/cclab-meteor/src/scheduler/tests.rs` (CREATE)
  - Do: Test leader acquisition, renewal, failover.

- [ ] **6.3** Integration tests
  - File: `crates/cclab-meteor/tests/scheduler_integration.rs` (CREATE)
  - Do: End-to-end test with Ion backend, multiple scheduler instances.
