use super::rc::{MbObject, MbRwLock, ObjData};
use super::value::MbValue;
/// Async/await runtime with tokio for Mamba (#293).
///
/// Thread-safe version — all async state is global, protected by DashMap/RwLock.
/// Coroutines and tasks can be accessed from any thread.
///
/// Architecture:
/// - Async functions produce "coroutine" objects (similar to generators)
/// - `await` suspends the coroutine and schedules it on the tokio runtime
/// - The event loop drives coroutines to completion
///
/// Task management, event loop, and bridge functions live in `async_task`.
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

// Re-export task/bridge/GIL functions so `symbols.rs` can reference
// them via `async_rt::*` without changing import paths.
pub use super::async_task::{
    mb_async_iter, mb_async_next_or_stop, mb_async_wait, mb_await, mb_await_external,
    mb_cancel_task, mb_create_task, mb_gather, mb_gil_acquire, mb_gil_held, mb_gil_release,
    mb_orbit_register_waker, mb_orbit_schedule, mb_run_until_complete, mb_sleep, mb_task_cancelled,
    mb_task_done, mb_task_result,
};

/// Coroutine state — similar to generator but for async functions.
pub struct MbCoroutine {
    pub name: String,
    pub module_name: String,
    pub state: u32,
    pub locals: Vec<MbValue>,
    pub result: Option<MbValue>,
    pub exhausted: bool,
    pub running: bool,
    pub awaiting: bool,
    pub suspend_requested: bool,
    pub pending_await: Option<MbValue>,
    pub pending_await_coro_id: Option<u64>,
    pub resume_value: Option<MbValue>,
    pub close_raises_ignored_exit: bool,
    /// Body function pointer for deferred execution (#313 R1).
    /// Set by compiled wrapper via `mb_coroutine_set_body`.
    /// Called by `mb_coroutine_step` to execute the body on first step.
    pub body_fn: Option<unsafe extern "C" fn(i64) -> i64>,
}

// Safety: MbCoroutine fields are only accessed through the global
// COROUTINES map which is RwLock-protected. body_fn is a plain
// function pointer (inherently Send+Sync).
unsafe impl Send for MbCoroutine {}
unsafe impl Sync for MbCoroutine {}

#[derive(Default)]
pub(crate) struct CompletedCoroutines {
    ranges: BTreeMap<u64, u64>,
}

impl CompletedCoroutines {
    fn clear(&mut self) {
        self.ranges.clear();
    }

    fn contains(&self, id: u64) -> bool {
        self.ranges
            .range(..=id)
            .next_back()
            .is_some_and(|(&start, &end)| start <= id && id <= end)
    }

    fn insert(&mut self, id: u64) {
        let mut start = id;
        let mut end = id;

        if let Some((&prev_start, &prev_end)) = self.ranges.range(..=id).next_back() {
            if prev_end >= id {
                return;
            }
            if prev_end.saturating_add(1) == id {
                start = prev_start;
                self.ranges.remove(&prev_start);
            }
        }

        if let Some((&next_start, &next_end)) = self.ranges.range(id..).next() {
            if next_start == id.saturating_add(1) {
                end = next_end;
                self.ranges.remove(&next_start);
            }
        }

        self.ranges.insert(start, end);
    }

    fn remove(&mut self, id: u64) {
        let Some((&start, &end)) = self.ranges.range(..=id).next_back() else {
            return;
        };
        if id < start || id > end {
            return;
        }
        self.ranges.remove(&start);
        if start < id {
            self.ranges.insert(start, id - 1);
        }
        if id < end {
            self.ranges.insert(id + 1, end);
        }
    }
}

fn compact_completed_coroutine(coro: &mut MbCoroutine) {
    // Recursive await workloads materialize large numbers of short-lived
    // coroutines. Once a coroutine is closed or completed, compiled code no
    // longer needs its execution frame payload; retaining locals/body/module
    // data for every exhausted coroutine drives the #1184 perf pin's RSS far
    // above CPython.
    coro.locals = Vec::new();
    coro.module_name = String::new();
    coro.body_fn = None;
    coro.running = false;
}

/// Task state for async execution.
pub struct MbTask {
    pub name: String,
    pub coroutine_id: u64,
    pub done: bool,
    pub cancelled: bool,
    pub result: MbValue,
}

// Safety: MbTask fields are only accessed through the global
// TASKS map which is RwLock-protected.
unsafe impl Send for MbTask {}
unsafe impl Sync for MbTask {}

// ── Global async runtime state (R5, R7) ──

/// Global coroutine registry — replaces thread_local COROUTINES.
pub(crate) static COROUTINES: std::sync::LazyLock<MbRwLock<HashMap<u64, MbCoroutine>>> =
    std::sync::LazyLock::new(|| MbRwLock::new(HashMap::new()));

/// Exhausted coroutine handles that have already had their completion consumed.
///
/// Keep a small tombstone so post-completion surface checks (`inspect`,
/// `iscoroutine`, and "already awaited" errors) still behave like a coroutine
/// object without retaining the full execution record in `COROUTINES`.
pub(crate) static COMPLETED_COROUTINES: std::sync::LazyLock<MbRwLock<CompletedCoroutines>> =
    std::sync::LazyLock::new(|| MbRwLock::new(CompletedCoroutines::default()));

/// Global task registry — replaces thread_local TASKS.
pub static TASKS: std::sync::LazyLock<MbRwLock<HashMap<u64, MbTask>>> =
    std::sync::LazyLock::new(|| MbRwLock::new(HashMap::new()));

const CORO_ID_BASE: u64 = 1 << 40;

/// Atomic counter for globally unique coroutine IDs (R7).
///
/// Coroutines are represented as int-tagged handles, like generators and a
/// handful of stdlib handles. Keep them in their own high range so a generator
/// created inside `await` cannot alias the currently running coroutine handle.
static NEXT_CORO_ID: AtomicU64 = AtomicU64::new(CORO_ID_BASE);

/// Atomic counter for globally unique task IDs (R7).
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn alloc_coro_id() -> u64 {
    NEXT_CORO_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn alloc_task_id() -> u64 {
    NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
}

/// Reset all global async state — coroutines, tasks, and ID counters.
/// Must be called between test runs to prevent stale function pointers
/// from causing SIGBUS on aarch64.
pub(crate) fn cleanup_all_async() {
    COROUTINES.write().unwrap().clear();
    COMPLETED_COROUTINES.write().unwrap().clear();
    TASKS.write().unwrap().clear();
    NEXT_CORO_ID.store(CORO_ID_BASE, Ordering::Relaxed);
    NEXT_TASK_ID.store(1, Ordering::Relaxed);
}

// ── Coroutine Creation ──

/// Create a new coroutine from an async function.
pub fn mb_coroutine_new(name: MbValue, locals: MbValue) -> MbValue {
    let coro_name = extract_str(name).unwrap_or_else(|| "<coroutine>".to_string());
    let module_name = super::closure::current_active_module_name();
    let local_vars = extract_list(locals);

    let coro = MbCoroutine {
        name: coro_name,
        module_name,
        state: 0,
        locals: local_vars,
        result: None,
        exhausted: false,
        running: false,
        awaiting: false,
        suspend_requested: false,
        pending_await: None,
        pending_await_coro_id: None,
        resume_value: None,
        close_raises_ignored_exit: false,
        body_fn: None,
    };
    let id = alloc_coro_id();
    COROUTINES.write().unwrap().insert(id, coro);
    MbValue::from_int(id as i64)
}

fn decode_coroutine_body(fn_ptr: MbValue) -> Option<unsafe extern "C" fn(i64) -> i64> {
    let addr = fn_ptr
        .as_func()
        .or_else(|| fn_ptr.as_int().map(|v| v as usize))?;
    if addr == 0 {
        return None;
    }
    Some(unsafe { std::mem::transmute(addr) })
}

/// Create a new coroutine with a pre-sized frame and registered body.
///
/// Async lowering hits coroutine construction on every recursive call in perf
/// pins such as #1184. Accepting the frame size and body pointer directly lets
/// codegen avoid the empty-list allocation/extraction round-trip and the
/// follow-up registry write in `mb_coroutine_set_body`.
pub fn mb_coroutine_new_with_body(name: MbValue, local_count: i64, fn_ptr: MbValue) -> MbValue {
    let coro_name = extract_str(name).unwrap_or_else(|| "<coroutine>".to_string());
    let module_name = super::closure::current_active_module_name();
    let local_count = local_count.max(0) as usize;
    let locals = if local_count == 0 {
        Vec::new()
    } else {
        vec![MbValue::none(); local_count]
    };

    let coro = MbCoroutine {
        name: coro_name,
        module_name,
        state: 0,
        locals,
        result: None,
        exhausted: false,
        running: false,
        awaiting: false,
        suspend_requested: false,
        pending_await: None,
        pending_await_coro_id: None,
        resume_value: None,
        close_raises_ignored_exit: false,
        body_fn: decode_coroutine_body(fn_ptr),
    };
    let id = alloc_coro_id();
    COROUTINES.write().unwrap().insert(id, coro);
    MbValue::from_int(id as i64)
}

/// Set the body function pointer for deferred execution (#313 R1).
/// Called by the compiled async wrapper after creating the coroutine.
/// Accepts both TAG_FUNC (MirConst::FuncRef lowering) and raw integer addresses.
pub fn mb_coroutine_set_body(coro_handle: MbValue, fn_ptr: MbValue) {
    if let Some(id) = coro_handle.as_int() {
        if let Some(body) = decode_coroutine_body(fn_ptr) {
            if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
                coro.body_fn = Some(body);
            }
        }
    }
}

thread_local! {
    static CURRENT_COROUTINE_ID: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

enum CoroutineStepPost {
    Snapshot,
    MarkAwaiting,
}

struct CoroutineStepOutcome {
    value: MbValue,
    exhausted: bool,
    result: Option<MbValue>,
}

impl CoroutineStepOutcome {
    fn none() -> Self {
        Self {
            value: MbValue::none(),
            exhausted: true,
            result: None,
        }
    }
}

pub(crate) fn has_current_coroutine() -> bool {
    CURRENT_COROUTINE_ID.with(|cell| cell.get().is_some())
}

fn mb_coroutine_step_with_post(
    coro_handle: MbValue,
    post: CoroutineStepPost,
) -> CoroutineStepOutcome {
    // Safepoint poll at coroutine step (R4)
    super::gc::gc_safepoint();
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return CoroutineStepOutcome::none();
    };

    enum StepPlan {
        Exhausted(MbValue),
        Invoke {
            body: unsafe extern "C" fn(i64) -> i64,
            module_name: String,
        },
        Idle,
        Error,
    }

    // Prepare the body invocation while holding the registry lock only once.
    let step_plan = {
        let mut coros = COROUTINES.write().unwrap();
        if let Some(coro) = coros.get_mut(&id) {
            if coro.exhausted {
                StepPlan::Exhausted(coro.result.unwrap_or_else(MbValue::none))
            } else if coro.running {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("coroutine already executing".to_string())),
                );
                StepPlan::Error
            } else if coro.state == 0 || coro.state > 1 {
                if coro.state == 0 {
                    coro.state = 1; // Mark as started
                }
                coro.running = true;
                if let Some(body) = coro.body_fn {
                    StepPlan::Invoke {
                        body,
                        module_name: coro.module_name.clone(),
                    }
                } else {
                    // Fail fast: no body function registered (#313 R1)
                    coro.exhausted = true;
                    coro.running = false;
                    coro.result = Some(MbValue::none());
                    StepPlan::Exhausted(MbValue::none())
                }
            } else {
                StepPlan::Idle
            }
        } else {
            StepPlan::Idle
        }
    };

    let mut body_return = None;
    match step_plan {
        StepPlan::Exhausted(result) => {
            return CoroutineStepOutcome {
                value: result,
                exhausted: true,
                result: Some(result),
            };
        }
        StepPlan::Invoke { body, module_name } => {
            // Call the compiled body function with coroutine handle.
            let previous = CURRENT_COROUTINE_ID.with(|cell| {
                let previous = cell.get();
                cell.set(Some(id));
                previous
            });
            super::closure::push_active_module_name(module_name);
            struct ModuleGuard;
            impl Drop for ModuleGuard {
                fn drop(&mut self) {
                    crate::runtime::closure::pop_active_module_name();
                }
            }
            let _module_guard = ModuleGuard;
            let raw_return = unsafe { body(coro_handle.to_bits() as i64) };
            body_return = Some(MbValue::from_bits(raw_return as u64));
            CURRENT_COROUTINE_ID.with(|cell| cell.set(previous));
            if let Some(coro) = COROUTINES.write().unwrap().get_mut(&id) {
                coro.running = false;
            }
            if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
                super::exception::mb_clear_exception();
                raise_runtime_error("coroutine raised StopIteration");
            }
        }
        StepPlan::Idle => {}
        StepPlan::Error => return CoroutineStepOutcome::none(),
    }

    let (exhausted, result) = match post {
        CoroutineStepPost::Snapshot => COROUTINES
            .read()
            .unwrap()
            .get(&id)
            .map(|c| (c.exhausted, c.result))
            .unwrap_or((true, None)),
        CoroutineStepPost::MarkAwaiting => {
            let mut coros = COROUTINES.write().unwrap();
            coros
                .get_mut(&id)
                .map(|coro| {
                    if coro.exhausted {
                        (true, coro.result)
                    } else {
                        coro.awaiting = true;
                        (false, None)
                    }
                })
                .unwrap_or((true, None))
        }
    };

    CoroutineStepOutcome {
        value: if exhausted {
            result.unwrap_or_else(MbValue::none)
        } else {
            body_return.unwrap_or_else(MbValue::none)
        },
        exhausted,
        result,
    }
}

/// Advance a coroutine to its next suspension point.
/// If the coroutine has a registered body function and hasn't started yet
/// (state == 0), calls the body function to execute it (#313 R1).
pub fn mb_coroutine_step(coro_handle: MbValue) -> MbValue {
    mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::Snapshot).value
}

/// Mark a coroutine as complete with a result.
///
/// Retains the result so `c.result` owns its own reference independent of
/// the caller. Without this, an async fn returning a heap value (e.g.
/// `return "hello " + name`) shared rc=1 between c.result and the awaiting
/// caller — caller scope-end release would free the heap object and
/// subsequent reads of c.result hit a dangling pointer.
pub fn mb_coroutine_complete(coro_handle: MbValue, result: MbValue) {
    if let Some(id) = coro_handle.as_int() {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            if coro.suspend_requested {
                return;
            }
            coro.exhausted = true;
            coro.awaiting = false;
            if let Some(pending) = coro.pending_await.take() {
                unsafe {
                    super::rc::release_if_ptr(pending);
                }
            }
            coro.pending_await_coro_id = None;
            if let Some(resume_value) = coro.resume_value.take() {
                unsafe {
                    super::rc::release_if_ptr(resume_value);
                }
            }
            // Retain so c.result holds a fresh ref independent of caller's rc.
            unsafe {
                super::rc::retain_if_ptr(result);
            }
            coro.result = Some(result);
            compact_completed_coroutine(coro);
        }
    }
}

pub fn mb_coroutine_set_close_raises(coro_handle: MbValue, value: MbValue) {
    if let Some(id) = coro_handle.as_int() {
        let flag = value
            .as_bool()
            .or_else(|| value.as_int().map(|i| i != 0))
            .or_else(|| match value.to_bits() {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            })
            .unwrap_or(false);
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            coro.close_raises_ignored_exit = flag;
        }
    }
}

fn new_str(value: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(value.into()))
}

fn raise_type_error(message: impl Into<String>) -> MbValue {
    super::exception::mb_raise(new_str("TypeError"), new_str(message.into()));
    MbValue::none()
}

fn raise_runtime_error(message: impl Into<String>) -> MbValue {
    super::exception::mb_raise(new_str("RuntimeError"), new_str(message.into()));
    MbValue::none()
}

fn raise_stop_iteration_value(value: MbValue) -> MbValue {
    let args = if value.is_none() {
        Vec::new()
    } else {
        vec![value]
    };
    let instance = super::exception::mb_exception_new_with_args(
        new_str("StopIteration"),
        MbValue::from_ptr(MbObject::new_list(args)),
    );
    super::class::mb_raise_instance(instance);
    MbValue::none()
}

pub fn is_known_coroutine(coro_handle: MbValue) -> bool {
    let Some(id) = coro_handle.as_int() else {
        return false;
    };
    let id = id as u64;
    COROUTINES.read().unwrap().contains_key(&id)
        || COMPLETED_COROUTINES.read().unwrap().contains(id)
}

pub(crate) fn is_completed_coroutine(coro_handle: MbValue) -> bool {
    let Some(id) = coro_handle.as_int() else {
        return false;
    };
    COMPLETED_COROUTINES.read().unwrap().contains(id as u64)
}

pub(crate) fn is_live_coroutine(coro_handle: MbValue) -> bool {
    let Some(id) = coro_handle.as_int() else {
        return false;
    };
    COROUTINES.read().unwrap().contains_key(&(id as u64))
}

pub(crate) fn live_await_target_coroutine(coro_like: MbValue) -> Option<MbValue> {
    if is_live_coroutine(coro_like) {
        return Some(coro_like);
    }
    let target = coroutine_wrapper_target(coro_like)?;
    is_live_coroutine(target).then_some(target)
}

pub(crate) fn live_await_target_coroutine_id(coro_like: MbValue) -> Option<u64> {
    live_await_target_coroutine(coro_like)
        .and_then(|coro| coro.as_int())
        .map(|id| id as u64)
}

pub(crate) fn await_target_coroutine(coro_like: MbValue) -> Option<MbValue> {
    if is_known_coroutine(coro_like) {
        return Some(coro_like);
    }
    coroutine_wrapper_target(coro_like)
}

pub(crate) fn tombstone_completed_coroutine(coro_handle: MbValue) {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return;
    };
    if let Some(mut coro) = COROUTINES.write().unwrap().remove(&id) {
        if let Some(pending) = coro.pending_await.take() {
            unsafe {
                super::rc::release_if_ptr(pending);
            }
        }
        coro.pending_await_coro_id = None;
        if let Some(resume_value) = coro.resume_value.take() {
            unsafe {
                super::rc::release_if_ptr(resume_value);
            }
        }
        if let Some(result) = coro.result.take() {
            unsafe {
                super::rc::release_if_ptr(result);
            }
        }
    }
    COMPLETED_COROUTINES.write().unwrap().insert(id);
}

pub fn mb_coroutine_is_exhausted(coro_handle: MbValue) -> MbValue {
    let exhausted = coro_handle
        .as_int()
        .and_then(|id| {
            COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|c| c.exhausted)
        })
        .unwrap_or(true);
    MbValue::from_bool(exhausted)
}

pub fn mb_coroutine_frame(coro_handle: MbValue) -> MbValue {
    let live = coro_handle
        .as_int()
        .and_then(|id| {
            COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|c| !c.exhausted)
        })
        .unwrap_or(false);
    if live {
        coro_handle
    } else {
        MbValue::none()
    }
}

pub fn mb_coroutine_running(coro_handle: MbValue) -> MbValue {
    let running = coro_handle
        .as_int()
        .and_then(|id| {
            COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|c| c.running)
        })
        .unwrap_or(false);
    MbValue::from_bool(running)
}

pub fn mb_coroutine_awaited(coro_handle: MbValue) -> MbValue {
    let awaited = coro_handle
        .as_int()
        .and_then(|id| {
            COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .map(|c| c.awaiting)
        })
        .unwrap_or(false);
    MbValue::from_bool(awaited)
}

pub fn mb_coroutine_await_wrapper(coro_handle: MbValue) -> MbValue {
    let wrapper = MbObject::new_instance("coroutine_wrapper".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*wrapper).data {
            fields
                .write()
                .unwrap()
                .insert("__coro__".to_string(), coro_handle);
        }
    }
    MbValue::from_ptr(wrapper)
}

pub fn coroutine_wrapper_target(wrapper: MbValue) -> Option<MbValue> {
    let ptr = wrapper.as_ptr()?;
    unsafe {
        let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        else {
            return None;
        };
        if class_name != "coroutine_wrapper" {
            return None;
        }
        fields.read().unwrap().get("__coro__").copied()
    }
}

pub fn is_coroutine_wrapper(wrapper: MbValue) -> bool {
    coroutine_wrapper_target(wrapper).is_some()
}

pub(crate) fn mb_coroutine_suspend_current_known_target(
    awaitable: MbValue,
    await_coro_id: Option<u64>,
) {
    CURRENT_COROUTINE_ID.with(|cell| {
        let Some(id) = cell.get() else {
            return;
        };
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&id) {
            coro.suspend_requested = true;
            coro.awaiting = true;
            unsafe {
                super::rc::retain_if_ptr(awaitable);
            }
            if let Some(previous) = coro.pending_await.replace(awaitable) {
                unsafe {
                    super::rc::release_if_ptr(previous);
                }
            }
            coro.pending_await_coro_id = await_coro_id;
        }
    });
}

pub(crate) fn mb_coroutine_suspend_current(awaitable: MbValue) {
    mb_coroutine_suspend_current_known_target(awaitable, live_await_target_coroutine_id(awaitable));
}

pub fn mb_coroutine_should_suspend(coro_handle: MbValue) -> MbValue {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return MbValue::from_bool(false);
    };
    let suspend = COROUTINES
        .write()
        .unwrap()
        .get_mut(&id)
        .map(|c| {
            let suspend = c.suspend_requested;
            c.suspend_requested = false;
            suspend
        })
        .unwrap_or(false);
    MbValue::from_bool(suspend)
}

pub fn mb_coroutine_should_suspend_set_state_i64(coro_handle: MbValue, state: i64) -> MbValue {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return MbValue::from_bool(false);
    };
    let state = state.max(0) as u32;
    let suspend = COROUTINES
        .write()
        .unwrap()
        .get_mut(&id)
        .map(|c| {
            let suspend = c.suspend_requested;
            c.suspend_requested = false;
            if suspend {
                c.state = state;
            }
            suspend
        })
        .unwrap_or(false);
    MbValue::from_bool(suspend)
}

pub(crate) enum CoroutineAwaitPoll {
    Yielded(MbValue),
    Complete(MbValue),
    Error,
}

pub(crate) fn mb_coroutine_send_for_await(
    coro_handle: MbValue,
    value: MbValue,
) -> CoroutineAwaitPoll {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return CoroutineAwaitPoll::Error;
    };
    if COMPLETED_COROUTINES.read().unwrap().contains(id) {
        raise_runtime_error("cannot reuse already awaited coroutine");
        return CoroutineAwaitPoll::Error;
    }
    let Some((state, exhausted, running)) = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .map(|c| (c.state, c.exhausted, c.running))
    else {
        return CoroutineAwaitPoll::Error;
    };
    if exhausted {
        let result = COROUTINES
            .read()
            .unwrap()
            .get(&id)
            .and_then(|c| c.result)
            .unwrap_or_else(MbValue::none);
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        return CoroutineAwaitPoll::Complete(result);
    }
    if running {
        super::exception::mb_raise(
            new_str("ValueError"),
            new_str("coroutine already executing"),
        );
        return CoroutineAwaitPoll::Error;
    }
    if state == 0 && !value.is_none() {
        raise_type_error("can't send non-None value to a just-started coroutine");
        return CoroutineAwaitPoll::Error;
    }

    if let Some(resumed) = super::async_task::mb_coroutine_resume_pending_await(coro_handle, value)
    {
        match resumed {
            super::async_task::AwaitResume::Yield(yielded) => {
                if super::exception::current_exception_type().is_some() {
                    return CoroutineAwaitPoll::Error;
                }
                return CoroutineAwaitPoll::Yielded(yielded);
            }
            super::async_task::AwaitResume::Complete(result) => {
                mb_coroutine_store_resume_value(coro_handle, result);
                let step = mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::Snapshot);
                if super::exception::current_exception_type().is_some() {
                    return CoroutineAwaitPoll::Error;
                }
                if step.exhausted {
                    let result = step.result.unwrap_or_else(MbValue::none);
                    unsafe {
                        super::rc::retain_if_ptr(result);
                    }
                    return CoroutineAwaitPoll::Complete(result);
                }
                return CoroutineAwaitPoll::Yielded(step.value);
            }
        }
    }

    let step = mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::MarkAwaiting);
    if super::exception::current_exception_type().is_some() {
        return CoroutineAwaitPoll::Error;
    }

    if step.exhausted {
        let result = step.result.unwrap_or_else(MbValue::none);
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        return CoroutineAwaitPoll::Complete(result);
    }

    CoroutineAwaitPoll::Yielded(step.value)
}

pub fn mb_coroutine_send(coro_handle: MbValue, value: MbValue) -> MbValue {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return MbValue::none();
    };
    if COMPLETED_COROUTINES.read().unwrap().contains(id) {
        return raise_runtime_error("cannot reuse already awaited coroutine");
    }
    let Some((state, exhausted, running)) = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .map(|c| (c.state, c.exhausted, c.running))
    else {
        return MbValue::none();
    };
    if exhausted {
        return raise_runtime_error("cannot reuse already awaited coroutine");
    }
    if running {
        super::exception::mb_raise(
            new_str("ValueError"),
            new_str("coroutine already executing"),
        );
        return MbValue::none();
    }
    if state == 0 && !value.is_none() {
        return raise_type_error("can't send non-None value to a just-started coroutine");
    }

    if let Some(resumed) = super::async_task::mb_coroutine_resume_pending_await(coro_handle, value)
    {
        match resumed {
            super::async_task::AwaitResume::Yield(yielded) => {
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                return yielded;
            }
            super::async_task::AwaitResume::Complete(result) => {
                mb_coroutine_store_resume_value(coro_handle, result);
                let step = mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::Snapshot);
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                if step.exhausted {
                    return raise_stop_iteration_value(step.result.unwrap_or_else(MbValue::none));
                }
                return step.value;
            }
        }
    }

    let step = mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::MarkAwaiting);
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }

    if step.exhausted {
        return raise_stop_iteration_value(step.result.unwrap_or_else(MbValue::none));
    }

    step.value
}

pub fn mb_coroutine_throw(coro_handle: MbValue, exc_type: MbValue, exc_msg: MbValue) -> MbValue {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return MbValue::none();
    };
    if COMPLETED_COROUTINES.read().unwrap().contains(id) {
        return raise_runtime_error("cannot reuse already awaited coroutine");
    }
    let exhausted = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .map(|c| c.exhausted)
        .unwrap_or(true);
    if exhausted {
        return raise_runtime_error("cannot reuse already awaited coroutine");
    }

    if let Some(resumed) =
        super::async_task::mb_coroutine_throw_pending_await(coro_handle, exc_type, exc_msg)
    {
        match resumed {
            super::async_task::AwaitResume::Yield(yielded) => {
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                return yielded;
            }
            super::async_task::AwaitResume::Complete(result) => {
                mb_coroutine_store_resume_value(coro_handle, result);
                let step = mb_coroutine_step_with_post(coro_handle, CoroutineStepPost::Snapshot);
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                if step.exhausted {
                    return raise_stop_iteration_value(step.result.unwrap_or_else(MbValue::none));
                }
                return step.value;
            }
        }
    }

    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let message = extract_str(exc_msg).unwrap_or_default();
    super::exception::mb_raise(new_str(type_name), new_str(message));
    MbValue::none()
}

pub fn mb_coroutine_close(coro_handle: MbValue) -> MbValue {
    let Some(id) = coro_handle.as_int().map(|id| id as u64) else {
        return MbValue::none();
    };
    if COMPLETED_COROUTINES.read().unwrap().contains(id) {
        return MbValue::none();
    }
    let Some((state, exhausted, running, close_raises_ignored_exit)) = COROUTINES
        .read()
        .unwrap()
        .get(&id)
        .map(|c| (c.state, c.exhausted, c.running, c.close_raises_ignored_exit))
    else {
        return MbValue::none();
    };
    if exhausted {
        return MbValue::none();
    }
    if running {
        return raise_runtime_error("cannot close a coroutine while it is running");
    }
    if let Some(coro) = COROUTINES.write().unwrap().get_mut(&id) {
        coro.exhausted = true;
        coro.awaiting = false;
        if let Some(pending) = coro.pending_await.take() {
            unsafe {
                super::rc::release_if_ptr(pending);
            }
        }
        coro.pending_await_coro_id = None;
        if let Some(resume_value) = coro.resume_value.take() {
            unsafe {
                super::rc::release_if_ptr(resume_value);
            }
        }
        coro.result = Some(MbValue::none());
        compact_completed_coroutine(coro);
    }
    if close_raises_ignored_exit && state != 0 {
        return raise_runtime_error("coroutine ignored GeneratorExit");
    }
    MbValue::none()
}

// ── Coroutine State Helpers (for compiled code) ──

pub fn mb_coroutine_get_state(coro_handle: MbValue) -> u32 {
    if let Some(id) = coro_handle.as_int() {
        COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .map(|c| c.state)
            .unwrap_or(u32::MAX)
    } else {
        u32::MAX
    }
}

pub fn mb_coroutine_get_state_value(coro_handle: MbValue) -> MbValue {
    MbValue::from_int(mb_coroutine_get_state(coro_handle) as i64)
}

pub fn mb_coroutine_get_state_i64(coro_handle: MbValue) -> i64 {
    mb_coroutine_get_state(coro_handle) as i64
}

pub fn mb_coroutine_set_state(coro_handle: MbValue, state: u32) {
    if let Some(id) = coro_handle.as_int() {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            coro.state = state;
            if state == u32::MAX {
                coro.exhausted = true;
            }
        }
    }
}

pub fn mb_coroutine_set_state_value(coro_handle: MbValue, state: MbValue) {
    let state = state.as_int().unwrap_or(0).max(0) as u32;
    mb_coroutine_set_state(coro_handle, state);
}

pub fn mb_coroutine_set_state_i64(coro_handle: MbValue, state: i64) {
    mb_coroutine_set_state(coro_handle, state.max(0) as u32);
}

pub(crate) fn mb_coroutine_store_resume_value(coro_handle: MbValue, value: MbValue) {
    if let Some(id) = coro_handle.as_int() {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            unsafe {
                super::rc::retain_if_ptr(value);
            }
            if let Some(previous) = coro.resume_value.replace(value) {
                unsafe {
                    super::rc::release_if_ptr(previous);
                }
            }
        }
    }
}

pub fn mb_coroutine_take_resume_value(coro_handle: MbValue) -> MbValue {
    if let Some(id) = coro_handle.as_int() {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            return coro.resume_value.take().unwrap_or_else(MbValue::none);
        }
    }
    MbValue::none()
}

pub fn mb_coroutine_get_local(coro_handle: MbValue, index: MbValue) -> MbValue {
    let idx = index.as_int().unwrap_or(0) as usize;
    if let Some(id) = coro_handle.as_int() {
        let val = COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .and_then(|c| c.locals.get(idx).copied())
            .unwrap_or(MbValue::none());
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        val
    } else {
        MbValue::none()
    }
}

pub fn mb_coroutine_set_local(coro_handle: MbValue, index: MbValue, value: MbValue) {
    let idx = index.as_int().unwrap_or(0) as usize;
    if let Some(id) = coro_handle.as_int() {
        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
            if idx >= coro.locals.len() {
                coro.locals.resize(idx + 1, MbValue::none());
            }
            coro.locals[idx] = value;
        }
    }
}

pub fn mb_coroutine_release(coro_handle: MbValue) {
    if let Some(id) = coro_handle.as_int() {
        COMPLETED_COROUTINES.write().unwrap().remove(id as u64);
        if let Some(mut coro) = COROUTINES.write().unwrap().remove(&(id as u64)) {
            if let Some(pending) = coro.pending_await.take() {
                unsafe {
                    super::rc::release_if_ptr(pending);
                }
            }
            coro.pending_await_coro_id = None;
            if let Some(resume_value) = coro.resume_value.take() {
                unsafe {
                    super::rc::release_if_ptr(resume_value);
                }
            }
            if let Some(result) = coro.result.take() {
                unsafe {
                    super::rc::release_if_ptr(result);
                }
            }
        }
    }
}

// ── Helpers ──

pub(crate) fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

pub(crate) fn extract_list(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::super::rc::MbObject;
    use super::*;

    #[test]
    fn test_coroutine_lifecycle() {
        let name = MbValue::from_ptr(MbObject::new_str("test_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);

        assert_eq!(mb_coroutine_get_state(coro), 0);
        mb_coroutine_set_state(coro, 1);
        assert_eq!(mb_coroutine_get_state(coro), 1);

        mb_coroutine_complete(coro, MbValue::from_int(42));
        let result = mb_await(coro);
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("RuntimeError")
        );
        super::super::exception::mb_clear_exception();

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_coroutine_local_set_get() {
        let name = MbValue::from_ptr(MbObject::new_str("local_test".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        // Store a value at index 0
        mb_coroutine_set_local(coro, MbValue::from_int(0), MbValue::from_int(77));
        let val = mb_coroutine_get_local(coro, MbValue::from_int(0));
        assert_eq!(val.as_int(), Some(77));
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_await_completed_coroutine_returns_immediately() {
        let name = MbValue::from_ptr(MbObject::new_str("done_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_complete(coro, MbValue::from_int(123));
        // Awaiting a completed coroutine should return immediately
        let result = mb_await(coro);
        assert_eq!(result.as_int(), Some(123));
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_completed_coroutine_discards_execution_payload() {
        let name = MbValue::from_ptr(MbObject::new_str("done_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("payload".to_string()),
        )]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_complete(coro, MbValue::from_int(123));

        let stored = COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .map(|c| {
                (
                    c.exhausted,
                    c.locals.len(),
                    c.module_name.clone(),
                    c.body_fn.is_none(),
                )
            })
            .unwrap();
        assert_eq!(stored, (true, 0, String::new(), true));

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_tombstoned_coroutine_stays_known_and_rejects_reuse() {
        let name = MbValue::from_ptr(MbObject::new_str("done_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_complete(coro, MbValue::from_int(123));
        tombstone_completed_coroutine(coro);

        assert!(is_known_coroutine(coro));
        assert!(is_completed_coroutine(coro));
        assert!(COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .is_none());
        assert!(COMPLETED_COROUTINES
            .read()
            .unwrap()
            .contains(coro.as_int().unwrap() as u64));

        let reused = mb_coroutine_send(coro, MbValue::none());
        assert!(reused.is_none());
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("RuntimeError")
        );
        super::super::exception::mb_clear_exception();

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_internal_await_poll_returns_completion_without_stop_iteration() {
        unsafe extern "C" fn complete_immediately(coro_bits: i64) -> i64 {
            let coro = MbValue::from_bits(coro_bits as u64);
            mb_coroutine_complete(coro, MbValue::from_int(7));
            MbValue::none().to_bits() as i64
        }

        let name = MbValue::from_ptr(MbObject::new_str("await_child".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_set_body(coro, MbValue::from_func(complete_immediately as usize));

        match mb_coroutine_send_for_await(coro, MbValue::none()) {
            CoroutineAwaitPoll::Complete(result) => {
                assert_eq!(result.as_int(), Some(7));
            }
            CoroutineAwaitPoll::Yielded(_) => panic!("expected direct completion"),
            CoroutineAwaitPoll::Error => panic!("expected direct completion"),
        }
        assert!(
            super::super::exception::current_exception_type().is_none(),
            "internal await completion should not materialize StopIteration"
        );

        let resumed = mb_coroutine_send(coro, MbValue::none());
        assert!(resumed.is_none());
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("RuntimeError")
        );
        super::super::exception::mb_clear_exception();
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_coroutine_new_with_body_presizes_locals_and_registers_body() {
        unsafe extern "C" fn body(_: i64) -> i64 {
            MbValue::none().to_bits() as i64
        }

        let name = MbValue::from_ptr(MbObject::new_str("await_child".to_string()));
        let coro = mb_coroutine_new_with_body(name, 2, MbValue::from_func(body as usize));

        let stored = COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .map(|c| (c.locals.len(), c.body_fn.is_some()))
            .unwrap();
        assert_eq!(stored, (2, true));

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_live_await_target_coroutine_skips_tombstones() {
        let name = MbValue::from_ptr(MbObject::new_str("done_coro".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        let wrapper = mb_coroutine_await_wrapper(coro);

        assert_eq!(live_await_target_coroutine(coro), Some(coro));
        assert_eq!(live_await_target_coroutine(wrapper), Some(coro));

        mb_coroutine_complete(coro, MbValue::from_int(123));
        tombstone_completed_coroutine(coro);

        assert_eq!(await_target_coroutine(coro), Some(coro));
        assert_eq!(await_target_coroutine(wrapper), Some(coro));
        assert_eq!(live_await_target_coroutine(coro), None);
        assert_eq!(live_await_target_coroutine(wrapper), None);

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_coroutine_should_suspend_set_state_combines_hot_suspend_path() {
        let name = MbValue::from_ptr(MbObject::new_str("await_parent".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        let child_name = MbValue::from_ptr(MbObject::new_str("await_child".to_string()));
        let child = mb_coroutine_new(child_name, MbValue::from_ptr(MbObject::new_list(vec![])));

        CURRENT_COROUTINE_ID.with(|cell| cell.set(coro.as_int().map(|id| id as u64)));
        mb_coroutine_suspend_current(child);
        CURRENT_COROUTINE_ID.with(|cell| cell.set(None));

        assert_eq!(
            mb_coroutine_should_suspend_set_state_i64(coro, 17).as_bool(),
            Some(true)
        );
        assert_eq!(mb_coroutine_get_state(coro), 17);
        assert_eq!(
            mb_coroutine_should_suspend_set_state_i64(coro, 23).as_bool(),
            Some(false)
        );
        assert_eq!(mb_coroutine_get_state(coro), 17);

        mb_coroutine_release(child);
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_coroutine_send_sets_awaiting_on_non_terminal_suspend() {
        unsafe extern "C" fn suspend_once(coro_bits: i64) -> i64 {
            let coro = MbValue::from_bits(coro_bits as u64);
            mb_coroutine_suspend_current(coro);
            MbValue::none().to_bits() as i64
        }

        let name = MbValue::from_ptr(MbObject::new_str("await_parent".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_set_body(coro, MbValue::from_func(suspend_once as usize));

        let yielded = mb_coroutine_send(coro, MbValue::none());
        assert!(yielded.is_none());
        assert_eq!(mb_coroutine_awaited(coro).as_bool(), Some(true));

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_coroutine_step_restores_module_name_after_suspend() {
        unsafe extern "C" fn suspend_once(coro_bits: i64) -> i64 {
            let coro = MbValue::from_bits(coro_bits as u64);
            mb_coroutine_suspend_current(coro);
            MbValue::none().to_bits() as i64
        }

        super::super::closure::push_active_module_name("bench.coroutines".to_string());
        let name = MbValue::from_ptr(MbObject::new_str("suspend_once".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        mb_coroutine_set_body(coro, MbValue::from_func(suspend_once as usize));
        super::super::closure::pop_active_module_name();

        let _ = mb_coroutine_step(coro);

        let stored_module = COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .map(|stored| stored.module_name.clone())
            .unwrap();
        assert_eq!(stored_module, "bench.coroutines");

        mb_coroutine_release(coro);
    }

    #[test]
    fn test_completed_coroutine_ranges_merge_and_split() {
        let mut completed = CompletedCoroutines::default();

        completed.insert(3);
        completed.insert(4);
        completed.insert(2);
        completed.insert(6);
        completed.insert(7);
        completed.insert(5);

        assert_eq!(completed.ranges.len(), 1);
        assert_eq!(completed.ranges.get(&2), Some(&7));
        assert!(completed.contains(2));
        assert!(completed.contains(7));
        assert!(!completed.contains(8));

        completed.remove(4);
        assert_eq!(completed.ranges.len(), 2);
        assert_eq!(completed.ranges.get(&2), Some(&3));
        assert_eq!(completed.ranges.get(&5), Some(&7));
        assert!(!completed.contains(4));
    }

    #[test]
    fn test_missing_body_fails_fast() {
        // Coroutine with no body fn should fail fast on step, not spin
        let name = MbValue::from_ptr(MbObject::new_str("no_body".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        // Don't set body_fn — step should mark exhausted immediately
        let result = mb_coroutine_step(coro);
        assert_eq!(result.as_int(), None, "missing body should return None");
        // Coroutine should now be exhausted
        let is_exhausted = COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .map(|c| c.exhausted)
            .unwrap_or(false);
        assert!(
            is_exhausted,
            "coroutine with no body should be exhausted after step"
        );
    }

    #[test]
    fn test_deferred_body_not_executed_before_step() {
        // Creating a coroutine should NOT execute the body
        let name = MbValue::from_ptr(MbObject::new_str("deferred".to_string()));
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = mb_coroutine_new(name, locals);
        // Before stepping, coroutine should not be exhausted
        let is_exhausted = COROUTINES
            .read()
            .unwrap()
            .get(&(coro.as_int().unwrap() as u64))
            .map(|c| c.exhausted)
            .unwrap_or(true);
        assert!(
            !is_exhausted,
            "coroutine should not be exhausted before step"
        );
        // State should still be 0 (not started)
        assert_eq!(mb_coroutine_get_state(coro), 0);
        mb_coroutine_release(coro);
    }

    #[test]
    fn test_atomic_id_allocation_unique() {
        let id1 = alloc_coro_id();
        let id2 = alloc_coro_id();
        let id3 = alloc_task_id();
        let id4 = alloc_task_id();
        assert_ne!(id1, id2, "coroutine IDs must be unique");
        assert_ne!(id3, id4, "task IDs must be unique");
    }
}
