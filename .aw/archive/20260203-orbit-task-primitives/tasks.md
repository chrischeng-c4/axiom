---
id: orbit-task-primitives
type: tasks
version: 1
created_at: 2026-01-31T15:35:00+00:00
updated_at: 2026-01-31T15:35:00+00:00
spec_type: utility
total_tasks: 6
---

# Implementation Tasks: orbit-task-primitives

## Overview

Implement Rust core primitives for task cancellation depth, task registry, and completion tracking.

## Tasks

### Task 1: Cancellation Depth Counter

**File**: `crates/cclab-orbit/src/task.rs`

**Changes**:
- Replace `cancelled: Arc<AtomicBool>` with `cancel_depth: Arc<AtomicUsize>`
- Update `cancel()` to increment depth
- Add `uncancel()` to decrement depth (min 0)
- Update `cancelled()` to return `depth > 0`
- Update `is_cancelled()` internal API

**Tests**:
- `test_cancel_increments_depth`
- `test_uncancel_decrements_depth`
- `test_nested_cancellation`

---

### Task 2: TaskId Type

**File**: `crates/cclab-orbit/src/task.rs`

**Changes**:
- Add `TaskId` newtype wrapper over `u64`
- Add atomic counter for generating unique IDs
- Add `id()` method to Task
- Implement `Hash`, `Eq`, `Copy` for TaskId

---

### Task 3: Task Registry

**File**: `crates/cclab-orbit/src/registry.rs` (new)

**Changes**:
- Create `TaskRegistry` struct with `RwLock<HashMap<TaskId, Weak<TaskInner>>>`
- Add `register(task: &Task) -> TaskId`
- Add `get(id: TaskId) -> Option<Task>`
- Add `remove(id: TaskId)`
- Add `all_tasks() -> Vec<Task>` (filters out dropped weak refs)
- Add `cleanup()` to prune dead weak refs

**Tests**:
- `test_register_and_get`
- `test_weak_ref_cleanup`
- `test_all_tasks`

---

### Task 4: Completion Channel

**File**: `crates/cclab-orbit/src/completion.rs` (new)

**Changes**:
- Define `CompletionEvent { task_id: TaskId, state: TaskState }`
- Define `TaskState` enum: `Done`, `Cancelled`, `Failed`
- Create `CompletionNotifier` with mpsc sender
- Create `CompletionReceiver` wrapper
- Integrate with Task::mark_done() to send completion event

**Tests**:
- `test_completion_notification`
- `test_multiple_completions`

---

### Task 5: WaitSet Abstraction

**File**: `crates/cclab-orbit/src/completion.rs`

**Changes**:
- Create `WaitSet` struct holding `Vec<TaskId>` (static)
- Add `WaitCondition` enum: `FirstCompleted`, `FirstException`, `AllCompleted`
- Add `async fn wait(condition: WaitCondition, timeout: Option<Duration>) -> (Vec<TaskId>, Vec<TaskId>)` (done, pending)
- Integrate with CompletionReceiver

**Tests**:
- `test_wait_first_completed`
- `test_wait_all_completed`
- `test_wait_timeout`

---

### Task 6: PyLoop Integration

**File**: `crates/cclab-orbit/src/loop_impl.rs`

**Changes**:
- Add `registry: Arc<TaskRegistry>` to PyLoop
- Add `completion_tx: mpsc::Sender<CompletionEvent>` to PyLoop
- Update `create_task()` to register task and wire completion notifier
- Add `all_tasks()` method
- Update lib.rs exports

**Tests**:
- `test_pyloop_task_registry`
- `test_pyloop_completion_events`

---

## Dependencies

```
Task 1 (cancel depth) - standalone
Task 2 (TaskId) - standalone
Task 3 (registry) - depends on Task 2
Task 4 (completion) - depends on Task 2
Task 5 (WaitSet) - depends on Task 4
Task 6 (integration) - depends on Task 3, 4, 5
```

## Acceptance Criteria

- [ ] All tasks pass `cargo test -p cclab-orbit --lib`
- [ ] `cargo check -p cclab-orbit` passes
- [ ] No unsafe code added
- [ ] Static assertions for Send+Sync on new types
