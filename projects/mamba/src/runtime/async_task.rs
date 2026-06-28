use super::async_rt::{
    alloc_task_id, mb_coroutine_step, mb_coroutine_suspend_current, MbTask, COROUTINES, TASKS,
};
use super::rc::{MbObject, ObjData};
use super::value::MbValue;
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

thread_local! {
    static AWAIT_DEADLINE: std::cell::Cell<Option<std::time::Instant>> =
        const { std::cell::Cell::new(None) };
}

// ── Task Creation (asyncio.create_task equivalent) ──

/// Create an async task from a coroutine.
pub fn mb_create_task(coro: MbValue) -> MbValue {
    let coro_id = coro.as_int().unwrap_or(0) as u64;
    let task = MbTask {
        name: format!("task-{coro_id}"),
        coroutine_id: coro_id,
        done: false,
        cancelled: false,
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
            TASKS
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|t| t.done)
                .unwrap_or(true),
        )
    } else {
        MbValue::from_bool(true)
    }
}

/// Get the result of a completed task.
pub fn mb_task_result(task_handle: MbValue) -> MbValue {
    let result = if let Some(id) = task_handle.as_int() {
        TASKS
            .read()
            .unwrap()
            .get(&(id as u64))
            .map(|t| t.result)
            .unwrap_or(MbValue::none())
    } else {
        MbValue::none()
    };
    unsafe {
        super::rc::retain_if_ptr(result);
    }
    result
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
                    task.cancelled = true;
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
            TASKS
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|t| t.cancelled)
                .unwrap_or(false),
        )
    } else {
        MbValue::from_bool(false)
    }
}

fn await_deadline_expired() -> bool {
    AWAIT_DEADLINE.with(|deadline| {
        deadline
            .get()
            .is_some_and(|d| std::time::Instant::now() >= d)
    })
}

fn raise_timeout_error() {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TimeoutError".to_string())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );
}

fn new_str(value: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(value.into()))
}

fn raise_type_error(message: impl Into<String>) -> MbValue {
    super::exception::mb_raise(new_str("TypeError"), new_str(message.into()));
    MbValue::none()
}

fn lookup_dunder(obj: MbValue, name: &str) -> MbValue {
    super::class::mb_lookup_dunder(obj, new_str(name))
}

fn has_dunder(obj: MbValue, name: &str) -> bool {
    !lookup_dunder(obj, name).is_none()
}

fn is_await_iterator(obj: MbValue) -> bool {
    if super::iter::is_iter_handle(obj)
        || super::generator::is_known_generator(obj)
        || super::async_rt::is_known_coroutine(obj)
    {
        return true;
    }
    has_dunder(obj, "__next__")
}

fn is_awaitable_value(obj: MbValue) -> bool {
    if obj.as_int().is_some_and(|id| {
        COROUTINES.read().unwrap().contains_key(&(id as u64))
            || super::generator::is_known_generator(obj)
    }) {
        return true;
    }
    has_dunder(obj, "__await__")
}

fn raise_non_awaitable(awaitable: MbValue) -> MbValue {
    raise_type_error(format!(
        "object {} can't be used in 'await' expression",
        super::builtins::value_type_name(awaitable)
    ))
}

fn await_iterator(iterator: MbValue) -> MbValue {
    if super::async_rt::is_known_coroutine(iterator) {
        match resume_await_iterator(iterator, MbValue::none()) {
            AwaitResume::Yield(yielded) => {
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                mb_coroutine_suspend_current(iterator);
                return yielded;
            }
            AwaitResume::Complete(result) => return result,
        }
    }

    if super::generator::is_known_generator(iterator) {
        let yielded = super::generator::mb_generator_send(iterator, MbValue::none());
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            super::exception::mb_clear_exception();
            return super::generator::mb_generator_stop_value();
        }
        if super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
        mb_coroutine_suspend_current(iterator);
        return yielded;
    }

    let yielded = super::iter::mb_next_or_stop(iterator);
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    if yielded.is_stop_iter_sentinel() {
        return MbValue::none();
    }
    mb_coroutine_suspend_current(iterator);
    yielded
}

fn await_via_dunder(awaitable: MbValue, method: MbValue) -> MbValue {
    let iterator = super::class::mb_call_method1(method, awaitable);
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    if !is_await_iterator(iterator) {
        return raise_type_error(format!(
            "__await__() returned non-iterator of type '{}'",
            super::builtins::value_type_name(iterator)
        ));
    }
    await_iterator(iterator)
}

pub(crate) enum AwaitResume {
    Yield(MbValue),
    Complete(MbValue),
}

pub(crate) fn stop_iteration_exception_value() -> MbValue {
    let exc = super::exception::mb_get_exception();
    let Some(ptr) = exc.as_ptr() else {
        return MbValue::none();
    };
    unsafe {
        let ObjData::Instance { ref fields, .. } = (*ptr).data else {
            return MbValue::none();
        };
        let guard = fields.read().unwrap();
        if let Some(value) = guard.get("value").copied() {
            super::rc::retain_if_ptr(value);
            return value;
        }
        if let Some(args) = guard.get("args").copied() {
            if let Some(args_ptr) = args.as_ptr() {
                if let ObjData::Tuple(items) = &(*args_ptr).data {
                    if let Some(value) = items.first().copied() {
                        super::rc::retain_if_ptr(value);
                        return value;
                    }
                }
            }
        }
    }
    MbValue::none()
}

fn resume_await_iterator(iterator: MbValue, value: MbValue) -> AwaitResume {
    if super::async_rt::is_known_coroutine(iterator) {
        let yielded = super::async_rt::mb_coroutine_send(iterator, value);
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            let result = stop_iteration_exception_value();
            super::exception::mb_clear_exception();
            return AwaitResume::Complete(result);
        }
        return AwaitResume::Yield(yielded);
    }
    if super::generator::is_known_generator(iterator) {
        let yielded = super::generator::mb_generator_send(iterator, value);
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            super::exception::mb_clear_exception();
            return AwaitResume::Complete(super::generator::mb_generator_stop_value());
        }
        return AwaitResume::Yield(yielded);
    }
    let yielded = super::iter::mb_next_or_stop(iterator);
    if yielded.is_stop_iter_sentinel() {
        AwaitResume::Complete(MbValue::none())
    } else {
        AwaitResume::Yield(yielded)
    }
}

fn throw_await_iterator(iterator: MbValue, exc_type: MbValue, exc_msg: MbValue) -> AwaitResume {
    if super::async_rt::is_known_coroutine(iterator) {
        let yielded = super::async_rt::mb_coroutine_throw(iterator, exc_type, exc_msg);
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            let result = stop_iteration_exception_value();
            super::exception::mb_clear_exception();
            return AwaitResume::Complete(result);
        }
        if super::exception::current_exception_type().is_some() {
            return AwaitResume::Complete(MbValue::none());
        }
        return AwaitResume::Yield(yielded);
    }
    if super::generator::is_known_generator(iterator) {
        let yielded = super::generator::mb_generator_throw(iterator, exc_type, exc_msg);
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            super::exception::mb_clear_exception();
            return AwaitResume::Complete(super::generator::mb_generator_stop_value());
        }
        if super::exception::current_exception_type().is_some() {
            return AwaitResume::Complete(MbValue::none());
        }
        return AwaitResume::Yield(yielded);
    }
    super::exception::mb_raise(exc_type, exc_msg);
    AwaitResume::Complete(MbValue::none())
}

pub(crate) fn mb_coroutine_resume_pending_await(
    coro_handle: MbValue,
    value: MbValue,
) -> Option<AwaitResume> {
    let id = coro_handle.as_int()? as u64;
    let pending = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .and_then(|coro| coro.pending_await)?;
    let resumed = resume_await_iterator(pending, value);
    if matches!(resumed, AwaitResume::Complete(_)) {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&id) {
            if let Some(pending) = coro.pending_await.take() {
                unsafe {
                    super::rc::release_if_ptr(pending);
                }
            }
            coro.awaiting = false;
        }
    }
    Some(resumed)
}

pub(crate) fn mb_coroutine_throw_pending_await(
    coro_handle: MbValue,
    exc_type: MbValue,
    exc_msg: MbValue,
) -> Option<AwaitResume> {
    let id = coro_handle.as_int()? as u64;
    let pending = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .and_then(|coro| coro.pending_await)?;
    let resumed = throw_await_iterator(pending, exc_type, exc_msg);
    if matches!(resumed, AwaitResume::Complete(_)) {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&id) {
            if let Some(pending) = coro.pending_await.take() {
                unsafe {
                    super::rc::release_if_ptr(pending);
                }
            }
            coro.awaiting = false;
        }
    }
    Some(resumed)
}

pub fn mb_await_async_context(awaitable: MbValue, method_name: &str) -> MbValue {
    if !is_awaitable_value(awaitable) {
        return raise_type_error(format!(
            "async with {} returned an object that does not implement __await__",
            method_name
        ));
    }
    mb_await(awaitable)
}

fn mb_await_async_for_anext(awaitable: MbValue) -> MbValue {
    if !is_awaitable_value(awaitable) {
        return raise_type_error(
            "async for __anext__ returned an object that does not implement __await__",
        );
    }
    mb_await(awaitable)
}

// ── Event Loop ──

/// Event loop state for scheduling tasks.
struct EventLoop {
    ready_queue: Vec<u64>, // task IDs ready to run
}

impl EventLoop {
    fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
        }
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
            timers
                .iter()
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
                tasks
                    .get(&task_id)
                    .map(|t| (t.coroutine_id, t.done))
                    .unwrap_or((0, true))
            };
            if is_done {
                continue;
            }

            // Skip timer coroutines that haven't expired yet
            let is_pending_timer = TIMERS.read().unwrap().contains_key(&coro_id);
            if is_pending_timer {
                self.ready_queue.push(task_id);
                continue;
            }

            // Step the coroutine
            mb_coroutine_step(MbValue::from_int(coro_id as i64));

            if super::exception::current_exception_type().is_some() {
                if let Some(coro) = COROUTINES.write().unwrap().get_mut(&coro_id) {
                    coro.exhausted = true;
                }
                if let Some(task) = TASKS.write().unwrap().get_mut(&task_id) {
                    task.done = true;
                    task.cancelled = false;
                    task.result = MbValue::none();
                }
                continue;
            }

            // Check if coroutine is done
            let exhausted = COROUTINES
                .read()
                .unwrap()
                .get(&coro_id)
                .map(|c| c.exhausted)
                .unwrap_or(true);

            if exhausted {
                let result = COROUTINES
                    .read()
                    .unwrap()
                    .get(&coro_id)
                    .and_then(|c| c.result)
                    .unwrap_or(MbValue::none());
                if let Some(task) = TASKS.write().unwrap().get_mut(&task_id) {
                    task.done = true;
                    task.cancelled = false;
                    task.result = result;
                }
            } else {
                // Re-schedule for next tick
                self.ready_queue.push(task_id);
            }
        }

        // If there are pending timers and no other work to do, yield CPU
        let has_timers = !TIMERS.read().unwrap().is_empty();
        if has_timers
            && self.ready_queue.iter().all(|tid| {
                let tasks = TASKS.read().unwrap();
                tasks
                    .get(tid)
                    .map(|t| TIMERS.read().unwrap().contains_key(&t.coroutine_id))
                    .unwrap_or(false)
            })
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

/// Drive not-yet-started pending tasks until `done` becomes true.
///
/// The current async lowering calls `mb_await` synchronously from inside the
/// running coroutine body, so a primitive such as `asyncio.Event.wait()` needs a
/// narrow way to let sibling tasks make progress while the current coroutine is
/// blocked in that call. We only schedule coroutines with state 0, which avoids
/// recursively stepping the coroutine currently on the stack.
pub fn mb_drive_pending_tasks_until<F>(mut done: F, max_iterations: usize) -> bool
where
    F: FnMut() -> bool,
{
    let mut event_loop = EventLoop::new();
    for _ in 0..max_iterations {
        if done() {
            return true;
        }
        let pending: Vec<u64> = {
            let tasks = TASKS.read().unwrap();
            let coros = COROUTINES.read().unwrap();
            tasks
                .iter()
                .filter(|(_, task)| !task.done)
                .filter(|(_, task)| {
                    coros
                        .get(&task.coroutine_id)
                        .is_some_and(|coro| !coro.exhausted && coro.state == 0)
                })
                .map(|(&task_id, _)| task_id)
                .collect()
        };
        if pending.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        } else {
            for task_id in pending {
                event_loop.schedule(task_id);
            }
        }
        event_loop.tick();
    }
    done()
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
            tasks
                .iter()
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
        if super::generator::is_known_generator(awaitable) {
            return await_iterator(awaitable);
        }
        // Distinguish a coroutine handle from a plain int. Async-iter
        // helpers (`g.__anext__()` on a generator-backed async-gen)
        // hand back the yielded primitive value directly, and we must
        // not treat it as an unknown coroutine — driving a non-existent
        // coroutine wastes the 100k iteration budget and surfaces as
        // `None` to the caller.
        let known = COROUTINES.read().unwrap().contains_key(&(id as u64));
        if !known {
            let method = lookup_dunder(awaitable, "__await__");
            if !method.is_none() {
                return await_via_dunder(awaitable, method);
            }
            return raise_non_awaitable(awaitable);
        }
        let already_awaited = COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .map(|c| c.awaiting && !c.exhausted)
            .unwrap_or(false);
        if already_awaited {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "coroutine is being awaited already".to_string(),
                )),
            );
            return MbValue::none();
        }
        // Fast path: coroutine already complete
        let (state, exhausted) = COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .map(|c| (c.state, c.exhausted))
            .unwrap_or((0, true));
        if exhausted {
            if state != 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "cannot reuse already awaited coroutine".to_string(),
                    )),
                );
                return MbValue::none();
            }
            let result = COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .and_then(|c| c.result)
                .unwrap_or(MbValue::none());
            unsafe {
                super::rc::retain_if_ptr(result);
            }
            return result;
        }
        if super::async_rt::has_current_coroutine() {
            return await_iterator(awaitable);
        }
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            coro.awaiting = true;
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
        let mut timed_out = false;
        for _ in 0..max_iterations {
            super::gc::gc_safepoint();
            if await_deadline_expired() {
                timed_out = true;
                mb_cancel_task(task_handle);
                raise_timeout_error();
                break;
            }
            let done = COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|c| c.exhausted)
                .unwrap_or(true);
            if done {
                completed = true;
                break;
            }
            event_loop.tick();
        }
        if !completed && !timed_out && super::exception::current_exception_type().is_none() {
            eprintln!("mamba: mb_await: iteration limit reached, coroutine may be incomplete");
        }

        WAKERS.write().unwrap().remove(&(id as u64));

        if timed_out {
            return MbValue::none();
        }

        let result = COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .and_then(|c| c.result)
            .unwrap_or(MbValue::none());
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        result
    } else if let Some(result) = await_asyncio_task(awaitable) {
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        result
    } else if let Some(result) = await_asyncio_future(awaitable) {
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        result
    } else {
        let method = lookup_dunder(awaitable, "__await__");
        if !method.is_none() {
            return await_via_dunder(awaitable, method);
        }
        raise_non_awaitable(awaitable)
    }
}

pub fn mb_async_iter(iterable: MbValue) -> MbValue {
    let method = lookup_dunder(iterable, "__aiter__");
    if method.is_none() {
        return raise_type_error(format!(
            "async for requires an object with __aiter__ method, got '{}'",
            super::builtins::value_type_name(iterable)
        ));
    }
    let async_iter = super::class::mb_call_method1(method, iterable);
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    if lookup_dunder(async_iter, "__anext__").is_none() {
        return raise_type_error(format!(
            "async for object from __aiter__ does not implement __anext__ (got '{}')",
            super::builtins::value_type_name(async_iter)
        ));
    }
    async_iter
}

pub fn mb_async_next_or_stop(async_iter: MbValue) -> MbValue {
    let method = lookup_dunder(async_iter, "__anext__");
    if method.is_none() {
        return raise_type_error(format!(
            "async for object '{}' does not implement __anext__",
            super::builtins::value_type_name(async_iter)
        ));
    }
    let awaitable = super::class::mb_call_method1(method, async_iter);
    if super::exception::current_exception_type().as_deref() == Some("StopAsyncIteration") {
        super::exception::mb_clear_exception();
        return MbValue::stop_iter_sentinel();
    }
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    let value = mb_await_async_for_anext(awaitable);
    if super::exception::current_exception_type().as_deref() == Some("StopAsyncIteration") {
        super::exception::mb_clear_exception();
        return MbValue::stop_iter_sentinel();
    }
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    value
}

pub fn mb_await_with_timeout(awaitable: MbValue, duration: std::time::Duration) -> MbValue {
    let deadline = std::time::Instant::now() + duration;
    let previous = AWAIT_DEADLINE.with(|cell| {
        let previous = cell.get();
        let next = match previous {
            Some(existing) if existing < deadline => Some(existing),
            _ => Some(deadline),
        };
        cell.set(next);
        previous
    });
    let result = mb_await(awaitable);
    AWAIT_DEADLINE.with(|cell| cell.set(previous));
    result
}

fn raise_cancelled_error() {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("CancelledError".to_string())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );
}

fn asyncio_task_ids(awaitable: MbValue) -> Option<(u64, u64)> {
    awaitable.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name != "asyncio.Task" {
                return None;
            }
            let fields = fields.read().unwrap();
            let task_id = fields.get("_task_id").and_then(|v| v.as_int())? as u64;
            let coro_id = fields.get("_coro_id").and_then(|v| v.as_int())? as u64;
            Some((task_id, coro_id))
        } else {
            None
        }
    })
}

fn await_asyncio_task(awaitable: MbValue) -> Option<MbValue> {
    let (task_id, coro_id) = asyncio_task_ids(awaitable)?;
    if TASKS
        .read()
        .unwrap()
        .get(&task_id)
        .map(|task| task.cancelled)
        .unwrap_or(false)
    {
        raise_cancelled_error();
        return Some(MbValue::none());
    }

    let mut event_loop = EventLoop::new();
    event_loop.schedule(task_id);
    let max_iterations = 100_000;
    for _ in 0..max_iterations {
        super::gc::gc_safepoint();
        let done = TASKS
            .read()
            .unwrap()
            .get(&task_id)
            .map(|task| task.done)
            .unwrap_or(true);
        if done {
            break;
        }
        event_loop.tick();
    }

    let (cancelled, result) = TASKS
        .read()
        .unwrap()
        .get(&task_id)
        .map(|task| (task.cancelled, task.result))
        .unwrap_or((false, MbValue::none()));
    if cancelled {
        raise_cancelled_error();
        return Some(MbValue::none());
    }
    if COROUTINES
        .read()
        .unwrap()
        .get(&coro_id)
        .is_some_and(|coro| coro.exhausted)
    {
        return Some(result);
    }
    Some(awaitable)
}

fn await_asyncio_future(awaitable: MbValue) -> Option<MbValue> {
    let (class_name, state, result) = awaitable.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            let fields = fields.read().unwrap();
            let state = fields
                .get("_state")
                .and_then(|v| v.as_ptr())
                .map(|p| {
                    if let ObjData::Str(ref s) = (*p).data {
                        s.clone()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();
            let result = fields.get("_result").copied().unwrap_or_else(MbValue::none);
            Some((class_name.clone(), state, result))
        } else {
            None
        }
    })?;
    if class_name != "asyncio.Future" {
        return None;
    }
    match state.as_str() {
        "CANCELLED" => {
            raise_cancelled_error();
            Some(MbValue::none())
        }
        "FINISHED" => Some(result),
        _ => Some(awaitable),
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
                let task_ids: Vec<(u64, u64)> = coro_list
                    .iter()
                    .map(|c| {
                        let coro_id = c.as_int().unwrap_or(0) as u64;
                        let task = mb_create_task(*c);
                        let tid = task.as_int().unwrap_or(0) as u64;
                        event_loop.schedule(tid);
                        (coro_id, tid)
                    })
                    .collect();

                let max_iterations = 100_000;
                let mut completed = false;
                for _ in 0..max_iterations {
                    let all_done = task_ids.iter().all(|(cid, _)| {
                        COROUTINES
                            .read()
                            .unwrap()
                            .get(cid)
                            .map(|c| c.exhausted)
                            .unwrap_or(true)
                    });
                    if all_done {
                        completed = true;
                        break;
                    }
                    event_loop.tick();
                }
                if !completed {
                    eprintln!(
                        "mamba: mb_gather: iteration limit reached, some tasks may be incomplete"
                    );
                }

                let results: Vec<MbValue> = task_ids
                    .iter()
                    .map(|(cid, _)| {
                        COROUTINES
                            .read()
                            .unwrap()
                            .get(cid)
                            .and_then(|c| c.result)
                            .unwrap_or(MbValue::none())
                    })
                    .collect();
                return MbValue::from_ptr(MbObject::new_list_borrowed(results));
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
            cancelled: false,
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
        let main_done = TASKS
            .read()
            .unwrap()
            .get(&main_task_id)
            .map(|t| t.done)
            .unwrap_or(true);
        if main_done {
            break;
        }
        event_loop.tick();
    }

    let result = TASKS
        .read()
        .unwrap()
        .get(&main_task_id)
        .map(|t| t.result)
        .unwrap_or(MbValue::none());
    unsafe {
        super::rc::retain_if_ptr(result);
    }
    result
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
        let is_coro = COROUTINES.read().unwrap().contains_key(&(id as u64));
        if is_coro {
            return mb_await(future);
        }
    }
    future
}

#[cfg(test)]
mod tests {
    use super::super::async_rt::{mb_coroutine_complete, mb_coroutine_new, mb_coroutine_release};
    use super::*;

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
        let has_waker = WAKERS
            .read()
            .unwrap()
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
        let is_registered = COROUTINES
            .read()
            .unwrap()
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

        let exhausted = COROUTINES
            .read()
            .unwrap()
            .get(&(timer_coro.as_int().unwrap() as u64))
            .map(|c| c.exhausted)
            .unwrap_or(false);
        assert!(exhausted, "zero-duration timer should expire after tick");
    }
}
