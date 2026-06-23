use super::rc::ObjData;
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
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

// Re-export task/bridge/GIL functions so `symbols.rs` can reference
// them via `async_rt::*` without changing import paths.
pub use super::async_task::{
    mb_async_wait, mb_await, mb_await_external, mb_cancel_task, mb_create_task, mb_gather,
    mb_gil_acquire, mb_gil_held, mb_gil_release, mb_orbit_register_waker, mb_orbit_schedule,
    mb_run_until_complete, mb_sleep, mb_task_cancelled, mb_task_done, mb_task_result,
};

/// Coroutine state — similar to generator but for async functions.
pub struct MbCoroutine {
    pub name: String,
    pub state: u32,
    pub locals: Vec<MbValue>,
    pub result: Option<MbValue>,
    pub exhausted: bool,
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

/// Task state for async execution.
pub struct MbTask {
    pub name: String,
    pub coroutine_id: u64,
    pub done: bool,
    pub result: MbValue,
}

// Safety: MbTask fields are only accessed through the global
// TASKS map which is RwLock-protected.
unsafe impl Send for MbTask {}
unsafe impl Sync for MbTask {}

// ── Global async runtime state (R5, R7) ──

/// Global coroutine registry — replaces thread_local COROUTINES.
pub(crate) static COROUTINES: std::sync::LazyLock<RwLock<HashMap<u64, MbCoroutine>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Global task registry — replaces thread_local TASKS.
pub static TASKS: std::sync::LazyLock<RwLock<HashMap<u64, MbTask>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Atomic counter for globally unique coroutine IDs (R7).
static NEXT_CORO_ID: AtomicU64 = AtomicU64::new(1);

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
    TASKS.write().unwrap().clear();
    NEXT_CORO_ID.store(1, Ordering::Relaxed);
    NEXT_TASK_ID.store(1, Ordering::Relaxed);
}

// ── Coroutine Creation ──

/// Create a new coroutine from an async function.
pub fn mb_coroutine_new(name: MbValue, locals: MbValue) -> MbValue {
    let coro_name = extract_str(name).unwrap_or_else(|| "<coroutine>".to_string());
    let local_vars = extract_list(locals);

    let coro = MbCoroutine {
        name: coro_name,
        state: 0,
        locals: local_vars,
        result: None,
        exhausted: false,
        body_fn: None,
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
        let addr = fn_ptr
            .as_func()
            .or_else(|| fn_ptr.as_int().map(|v| v as usize));
        if let Some(ptr_val) = addr {
            if ptr_val != 0 {
                let body: unsafe extern "C" fn(i64) -> i64 =
                    unsafe { std::mem::transmute(ptr_val) };
                if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
                    coro.body_fn = Some(body);
                }
            }
        }
    }
}

/// Advance a coroutine to its next suspension point.
/// If the coroutine has a registered body function and hasn't started yet
/// (state == 0), calls the body function to execute it (#313 R1).
pub fn mb_coroutine_step(coro_handle: MbValue) -> MbValue {
    // Safepoint poll at coroutine step (R4)
    super::gc::gc_safepoint();
    if let Some(id) = coro_handle.as_int() {
        // Check if already exhausted
        let exhausted = COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .map(|c| c.exhausted)
            .unwrap_or(true);
        if exhausted {
            return COROUTINES
                .read()
                .unwrap()
                .get(&(id as u64))
                .and_then(|c| c.result)
                .unwrap_or(MbValue::none());
        }

        // Try to call the body function if registered and not yet started
        let step_result = {
            let mut coros = COROUTINES.write().unwrap();
            if let Some(coro) = coros.get_mut(&(id as u64)) {
                if coro.state == 0 {
                    coro.state = 1; // Mark as started
                    if let Some(body) = coro.body_fn {
                        Ok(Some(body))
                    } else {
                        // Fail fast: no body function registered (#313 R1)
                        coro.exhausted = true;
                        coro.result = Some(MbValue::none());
                        Err(())
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        };

        match step_result {
            Ok(Some(body)) => {
                // Call the compiled body function with coroutine handle
                unsafe {
                    body(coro_handle.to_bits() as i64);
                }
            }
            Err(()) => { /* fail-fast: coroutine marked exhausted above */ }
            Ok(None) => { /* already started, nothing to do */ }
        }

        // Return result if now exhausted
        COROUTINES
            .read()
            .unwrap()
            .get(&(id as u64))
            .and_then(|c| c.result)
            .unwrap_or(MbValue::none())
    } else {
        MbValue::none()
    }
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
            coro.exhausted = true;
            // Retain so c.result holds a fresh ref independent of caller's rc.
            unsafe {
                super::rc::retain_if_ptr(result);
            }
            coro.result = Some(result);
        }
    }
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
        COROUTINES.write().unwrap().remove(&(id as u64));
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
        assert_eq!(result.as_int(), Some(42));

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
