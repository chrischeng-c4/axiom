/// Task management, event loop, await, orbit bridge, and GIL for Mamba async (#313).
///
/// Thread-safe version — uses global RwLock-protected state from async_rt.
///
/// ## Architecture: v2 Multi-Threaded Executor
///
/// All async state (COROUTINES, TASKS, WAKERS, TIMERS) is global and
/// thread-safe. Tasks can be scheduled on Tokio's multi-threaded executor
/// for true parallel async I/O.

use std::collections::HashMap;
use std::sync::RwLock;
use super::value::MbValue;
use super::rc::{MbObject, ObjData};
use super::async_rt::{
    MbTask, COROUTINES, TASKS, alloc_task_id,
    mb_coroutine_step,
};

// ── Task Creation (asyncio.create_task equivalent) ──

/// Create an async task from a coroutine.
pub fn mb_create_task(coro: MbValue) -> MbValue {
    let coro_id = coro.as_int().unwrap_or(0) as u64;
    let task = MbTask {
        name: format!("task-{coro_id}"),
        coroutine_id: coro_id,
        done: false,
        result: MbValue::none(),
    };
    let id = alloc_task_id();
    TASKS.write().unwrap().insert(id, task);
    MbValue::from_int(id as i64)
}

/// Check if a task is done.
pub fn mb_task_done(task_handle: MbValue) -> MbValue {
    if let Some(id) = task_handle.as_int() {
        MbValue::from_bool(
            TASKS.read().unwrap()
                .get(&(id as u64)).map(|t| t.done).unwrap_or(true)
        )
    } else {
        MbValue::from_bool(true)
    }
}

/// Get the result of a completed task.
pub fn mb_task_result(task_handle: MbValue) -> MbValue {
    if let Some(id) = task_handle.as_int() {
        TASKS.read().unwrap()
            .get(&(id as u64))
            .map(|t| t.result)
            .unwrap_or(MbValue::none())
    } else {
        MbValue::none()
    }
}

// ── Task Cancellation ──

/// Cancel a task.
pub fn mb_cancel_task(task_handle: MbValue) -> MbValue {
    if let Some(id) = task_handle.as_int() {
        let coro_id = {
            let mut tasks = TASKS.write().unwrap();
            if let Some(task) = tasks.get_mut(&(id as u64)) {
                if !task.done {
                    task.done = true;
                    task.result = MbValue::none();
                    Some(task.coroutine_id)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(cid) = coro_id {
            if let Some(coro) = COROUTINES.write().unwrap().get_mut(&cid) {
                coro.exhausted = true;
            }
            return MbValue::from_bool(true);
        }
        MbValue::from_bool(false)
    } else {
        MbValue::from_bool(false)
    }
}

/// Check if a task was cancelled.
pub fn mb_task_cancelled(task_handle: MbValue) -> MbValue {
    if let Some(id) = task_handle.as_int() {
        MbValue::from_bool(
            TASKS.read().unwrap()
                .get(&(id as u64))
                .map(|t| t.done && t.result.is_none())
                .unwrap_or(false)
        )
    } else {
        MbValue::from_bool(false)
    }
}

// ── Event Loop ──

/// Event loop state for scheduling tasks.
struct EventLoop {
    ready_queue: Vec<u64>,  // task IDs ready to run
}

impl EventLoop {
    fn new() -> Self {
        Self { ready_queue: Vec::new() }
    }

    fn schedule(&mut self, task_id: u64) {
        if !self.ready_queue.contains(&task_id) {
            self.ready_queue.push(task_id);
        }
    }

    /// Run one iteration: check timers, then step all ready tasks.
    fn tick(&mut self) {
        // Safepoint poll at event loop tick (R4)
        super::gc::gc_safepoint();

        // Phase 1: Check timers — complete any expired timer coroutines
        let now = std::time::Instant::now();
        let expired: Vec<u64> = {
            let timers = TIMERS.read().unwrap();
            timers.iter()
                .filter(|(_, deadline)| now >= **deadline)
                .map(|(&cid, _)| cid)
                .collect()
        };
        for cid in &expired {
            use super::async_rt::mb_coroutine_complete;
            mb_coroutine_complete(MbValue::from_int(*cid as i64), MbValue::none());
            TIMERS.write().unwrap().remove(cid);
        }

        // Phase 2: Step all ready tasks
        let tasks_to_run: Vec<u64> = self.ready_queue.drain(..).collect();
        for task_id in tasks_to_run {
            let (coro_id, is_done) = {
                let tasks = TASKS.read().unwrap();
                tasks.get(&task_id)
                    .map(|t| (t.coroutine_id, t.done))
                    .unwrap_or((0, true))
            };
            if is_done { continue; }

            // Skip timer coroutines that haven't expired yet
            let is_pending_timer = TIMERS.read().unwrap().contains_key(&coro_id);
            if is_pending_timer {
                self.ready_queue.push(task_id);
                continue;
            }

            // Step the coroutine
            mb_coroutine_step(MbValue::from_int(coro_id as i64));

            // Check if coroutine is done
            let exhausted = COROUTINES.read().unwrap()
                .get(&coro_id).map(|c| c.exhausted).unwrap_or(true);

            if exhausted {
                let result = COROUTINES.read().unwrap()
                    .get(&coro_id)
                    .and_then(|c| c.result)
                    .unwrap_or(MbValue::none());
                if let Some(task) = TASKS.write().unwrap().get_mut(&task_id) {
                    task.done = true;
                    task.result = result;
                }
            } else {
                // Re-schedule for next tick
                self.ready_queue.push(task_id);
            }
        }

        // If there are pending timers and no other work to do, yield CPU
        let has_timers = !TIMERS.read().unwrap().is_empty();
        if has_timers && self.ready_queue.iter().all(|tid| {
            let tasks = TASKS.read().unwrap();
            tasks.get(tid)
                .map(|t| TIMERS.read().unwrap().contains_key(&t.coroutine_id))
                .unwrap_or(false)
        }) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

// ── Orbit Bridge (#313 R2) ──

/// Schedule a coroutine on the Orbit event loop.
pub fn mb_orbit_schedule(coro: MbValue) -> MbValue {
    let task = mb_create_task(coro);
    task
}

/// Global waker registry — maps coroutine IDs to their pending task IDs.
pub(crate) static WAKERS: std::sync::LazyLock<RwLock<HashMap<u64, u64>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a waker for a coroutine, linking it to a task.
pub fn mb_orbit_register_waker(coro: MbValue) -> MbValue {
    if let Some(coro_id) = coro.as_int() {
        let task_id = {
            let tasks = TASKS.read().unwrap();
            tasks.iter()
                .find(|(_, t)| t.coroutine_id == coro_id as u64 && !t.done)
                .map(|(&id, _)| id)
        };
        if let Some(tid) = task_id {
            WAKERS.write().unwrap().insert(coro_id as u64, tid);
        }
    }
    MbValue::none()
}

// ── Await Support ──

/// Await a coroutine or future.
///
/// NEW per ownership audit (rc.rs:13-67): caller receives a fresh ref.
/// `c.result` keeps its own ref via mb_coroutine_complete's retain;
/// mb_await retains again on the way out so caller and storage are
/// independent. Without this retain, async fn returning heap values
/// (Str concat, list ctor, etc.) SIGSEGVs after caller's scope drops.
pub fn mb_await(awaitable: MbValue) -> MbValue {
    if let Some(id) = awaitable.as_int() {
        // Distinguish a coroutine handle from a plain int. Async-iter
        // helpers (`g.__anext__()` on a generator-backed async-gen)
        // hand back the yielded primitive value directly, and we must
        // not treat it as an unknown coroutine — driving a non-existent
        // coroutine wastes the 100k iteration budget and surfaces as
        // `None` to the caller.
        let known = COROUTINES.read().unwrap().contains_key(&(id as u64));
        if !known {
            return awaitable;
        }
        // Fast path: coroutine already complete
        let exhausted = COROUTINES.read().unwrap()
            .get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true);
        if exhausted {
            let result = COROUTINES.read().unwrap()
                .get(&(id as u64))
                .and_then(|c| c.result)
                .unwrap_or(MbValue::none());
            unsafe { super::rc::retain_if_ptr(result); }
            return result;
        }

        // Schedule via event loop and drive to completion
        let task_handle = mb_orbit_schedule(awaitable);
        let task_id = task_handle.as_int().unwrap_or(0) as u64;

        mb_orbit_register_waker(awaitable);

        let mut event_loop = EventLoop::new();
        event_loop.schedule(task_id);

        // Also schedule any waker-associated tasks
        {
            let wakers = WAKERS.read().unwrap();
            for (_, &tid) in wakers.iter() {
                event_loop.schedule(tid);
            }
        }

        let max_iterations = 100_000;
        let mut completed = false;
        for _ in 0..max_iterations {
            super::gc::gc_safepoint();
            let done = COROUTINES.read().unwrap()
                .get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true);
            if done { completed = true; break; }
            event_loop.tick();
        }
        if !completed {
            eprintln!("mamba: mb_await: iteration limit reached, coroutine may be incomplete");
        }

        WAKERS.write().unwrap().remove(&(id as u64));

        let result = COROUTINES.read().unwrap()
            .get(&(id as u64))
            .and_then(|c| c.result)
            .unwrap_or(MbValue::none());
        unsafe { super::rc::retain_if_ptr(result); }
        result
    } else {
        // await on a non-coroutine (e.g. list from asyncio.gather, constant value):
        // pass through unchanged so users can `await asyncio.gather(...)`.
        awaitable
    }
}

// ── asyncio-compatible Functions ──

/// asyncio.gather(*coros) — run multiple coroutines concurrently.
pub fn mb_gather(coros: MbValue) -> MbValue {
    if let Some(ptr) = coros.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let coro_list = lock.read().unwrap();
                let mut event_loop = EventLoop::new();
                let task_ids: Vec<(u64, u64)> = coro_list.iter().map(|c| {
                    let coro_id = c.as_int().unwrap_or(0) as u64;
                    let task = mb_create_task(*c);
                    let tid = task.as_int().unwrap_or(0) as u64;
                    event_loop.schedule(tid);
                    (coro_id, tid)
                }).collect();

                let max_iterations = 100_000;
                let mut completed = false;
                for _ in 0..max_iterations {
                    let all_done = task_ids.iter().all(|(cid, _)| {
                        COROUTINES.read().unwrap()
                            .get(cid).map(|c| c.exhausted).unwrap_or(true)
                    });
                    if all_done { completed = true; break; }
                    event_loop.tick();
                }
                if !completed {
                    eprintln!("mamba: mb_gather: iteration limit reached, some tasks may be incomplete");
                }

                let results: Vec<MbValue> = task_ids.iter().map(|(cid, _)| {
                    COROUTINES.read().unwrap()
                        .get(cid)
                        .and_then(|c| c.result)
                        .unwrap_or(MbValue::none())
                }).collect();
                return MbValue::from_ptr(MbObject::new_list(results));
            }
        }
    }
    MbValue::none()
}

/// asyncio.sleep(seconds) — cooperative sleep.
pub fn mb_sleep(seconds: MbValue) -> MbValue {
    let duration = if let Some(secs) = seconds.as_float() {
        if secs <= 0.0 || secs.is_nan() || secs.is_infinite() {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_secs_f64(secs)
        }
    } else if let Some(secs) = seconds.as_int() {
        if secs <= 0 {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_secs(secs as u64)
        }
    } else {
        return MbValue::none();
    };

    let name = MbValue::from_ptr(MbObject::new_str("sleep_timer".to_string()));
    let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
    let coro = super::async_rt::mb_coroutine_new(name, locals);

    let deadline = std::time::Instant::now() + duration;
    if let Some(coro_id) = coro.as_int() {
        TIMERS.write().unwrap().insert(coro_id as u64, deadline);
    }

    coro
}

/// Global timer registry — maps coroutine IDs to their wake-up deadlines.
static TIMERS: std::sync::LazyLock<RwLock<HashMap<u64, std::time::Instant>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// asyncio.wait(tasks, timeout=None).
pub fn mb_async_wait(tasks: MbValue, _timeout: MbValue) -> MbValue {
    if let Some(ptr) = tasks.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let task_list = lock.read().unwrap();
                let mut done = Vec::new();
                let mut pending = Vec::new();
                for t in task_list.iter() {
                    let is_done = mb_task_done(*t).as_bool().unwrap_or(false);
                    if is_done {
                        done.push(*t);
                    } else {
                        pending.push(*t);
                    }
                }
                return MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_ptr(MbObject::new_list(done)),
                    MbValue::from_ptr(MbObject::new_list(pending)),
                ]));
            }
        }
    }
    MbValue::none()
}

/// Run the event loop until the main coroutine completes.
pub fn mb_run_until_complete(main_coro: MbValue) -> MbValue {
    let main_task_id = {
        let coro_id = main_coro.as_int().unwrap_or(0) as u64;
        let task = MbTask {
            name: "main".to_string(),
            coroutine_id: coro_id,
            done: false,
            result: MbValue::none(),
        };
        let id = alloc_task_id();
        TASKS.write().unwrap().insert(id, task);
        id
    };

    let mut event_loop = EventLoop::new();
    event_loop.schedule(main_task_id);

    // Also schedule all pending tasks
    {
        let tasks = TASKS.read().unwrap();
        for (&id, task) in tasks.iter() {
            if !task.done {
                event_loop.schedule(id);
            }
        }
    }

    let max_iterations = 10_000;
    for _ in 0..max_iterations {
        super::gc::gc_safepoint();
        let main_done = TASKS.read().unwrap()
            .get(&main_task_id).map(|t| t.done).unwrap_or(true);
        if main_done { break; }
        event_loop.tick();
    }

    TASKS.read().unwrap()
        .get(&main_task_id)
        .map(|t| t.result)
        .unwrap_or(MbValue::none())
}

// ── GIL Management (#313 R3) ──
// In no-GIL mode, these are no-ops but kept for API compatibility.

thread_local! {
    static GIL_HELD: std::cell::Cell<bool> = std::cell::Cell::new(true);
}

/// Release the GIL (no-op in no-GIL mode, kept for API compat).
pub fn mb_gil_release() {
    GIL_HELD.with(|g| g.set(false));
}

/// Re-acquire the GIL (no-op in no-GIL mode, kept for API compat).
pub fn mb_gil_acquire() {
    GIL_HELD.with(|g| g.set(true));
}

/// Check if the current thread holds the GIL.
pub fn mb_gil_held() -> MbValue {
    MbValue::from_bool(GIL_HELD.with(|g| g.get()))
}

// ── Future Interoperability (#313 R4) ──

/// Await an external future.
pub fn mb_await_external(future: MbValue) -> MbValue {
    if let Some(id) = future.as_int() {
        let is_coro = COROUTINES.read().unwrap()
            .contains_key(&(id as u64));
        if is_coro {
            return mb_await(future);
        }
    }
    future
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::async_rt::{
        mb_coroutine_new, mb_coroutine_complete, mb_coroutine_release,
    };

    #[test]
    fn test_task() {
        let name = MbValue::from_ptr(MbObject::new_str("async_fn".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_complete(coro, MbValue::from_int(99));

        let task = mb_create_task(coro);
        assert!(task.is_int());
    }

    #[test]
    fn test_orbit_schedule() {
        let name = MbValue::from_ptr(MbObject::new_str("orbit_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        let task = mb_orbit_schedule(coro);
        assert!(task.is_int(), "orbit_schedule should return a task handle");
    }

    #[test]
    fn test_gil_release_acquire() {
        mb_gil_acquire();
        assert_eq!(mb_gil_held().as_bool(), Some(true));
        mb_gil_release();
        assert_eq!(mb_gil_held().as_bool(), Some(false));
        mb_gil_acquire();
        assert_eq!(mb_gil_held().as_bool(), Some(true));
    }

    #[test]
    fn test_await_external_passthrough() {
        let val = MbValue::from_int(42);
        let result = mb_await_external(val);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_waker_registration() {
        let name = MbValue::from_ptr(MbObject::new_str("waker_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        let task = mb_orbit_schedule(coro);
        assert!(task.is_int());
        mb_orbit_register_waker(coro);
        let has_waker = WAKERS.read().unwrap()
            .contains_key(&(coro.as_int().unwrap() as u64));
        assert!(has_waker, "waker should be registered for the coroutine");
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_gather_completed_coroutines() {
        let n1 = MbValue::from_ptr(MbObject::new_str("c1".to_string()));
        let l1 = MbValue::from_ptr(MbObject::new_list(vec![]));
        let c1 = mb_coroutine_new(n1, l1);
        mb_coroutine_complete(c1, MbValue::from_int(10));

        let n2 = MbValue::from_ptr(MbObject::new_str("c2".to_string()));
        let l2 = MbValue::from_ptr(MbObject::new_list(vec![]));
        let c2 = mb_coroutine_new(n2, l2);
        mb_coroutine_complete(c2, MbValue::from_int(20));

        let coros = MbValue::from_ptr(MbObject::new_list(vec![c1, c2]));
        let results = mb_gather(coros);
        assert!(results.as_ptr().is_some(), "gather should return a list");
    }

    #[test]
    fn test_sleep_creates_timer_coroutine() {
        let coro = mb_sleep(MbValue::from_int(0));
        assert!(coro.is_int(), "sleep should return a coroutine handle");
        let is_registered = COROUTINES.read().unwrap()
            .contains_key(&(coro.as_int().unwrap() as u64));
        assert!(is_registered, "sleep timer should be in COROUTINES");
    }

    #[test]
    fn test_sleep_timer_expires_in_event_loop() {
        let timer_coro = mb_sleep(MbValue::from_int(0));
        let task = mb_create_task(timer_coro);
        let tid = task.as_int().unwrap() as u64;

        let mut event_loop = EventLoop::new();
        event_loop.schedule(tid);
        std::thread::sleep(std::time::Duration::from_millis(1));
        event_loop.tick();

        let exhausted = COROUTINES.read().unwrap()
            .get(&(timer_coro.as_int().unwrap() as u64))
            .map(|c| c.exhausted).unwrap_or(false);
        assert!(exhausted, "zero-duration timer should expire after tick");
    }
}
