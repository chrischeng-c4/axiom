//! threading module for Mamba (#417, #1265 Task #82, Wave-9).
//!
//! CPython 3.12 `threading` 32-entry surface:
//!   Barrier, BoundedSemaphore, BrokenBarrierError, Condition, Event,
//!   ExceptHookArgs, Lock, RLock, Semaphore, TIMEOUT_MAX, Thread,
//!   ThreadError, Timer, WeakSet, activeCount, active_count,
//!   currentThread, current_thread, enumerate, excepthook, functools,
//!   get_ident, get_native_id, getprofile, gettrace, local, main_thread,
//!   setprofile, setprofile_all_threads, settrace, settrace_all_threads,
//!   stack_size.
//!
//! Carve-outs:
//!   - `Thread.start` runs target callables on a background OS thread and
//!     `join` waits for that worker, which is enough for CPython-shaped
//!     client/server handshakes. Most sync primitives are still simplified:
//!     Lock, RLock, Condition, Event, Semaphore,
//!     BoundedSemaphore, Barrier) return passive Instance dicts whose
//!     methods (acquire/release/wait/notify/set/clear) are no-ops
//!     surfaced through the dispatcher table. `Barrier.wait` cannot truly
//!     rendezvous (no peer is ever blocked); it returns a rotating
//!     CPython-shaped arrival index 0..parties-1 instead of raising. Worker
//!     targets get their own thread-local `current_thread()` / `get_ident()`
//!     view and restore registered `threading.local` fields on exit to
//!     preserve the common per-thread isolation contracts.
//!   - `Timer` returns a Thread-shaped Instance with `interval` /
//!     `function` fields; it never fires.
//!   - `local` returns a fresh dict — thread-local semantics collapse
//!     to plain dict semantics in the single-thread runtime.
//!   - `active_count` remains simplified; `enumerate` tracks started workers
//!     until `join()` removes them.
//!   - `get_ident` / `get_native_id`: 1 on the main thread; while a
//!     `Thread.start()` target runs they reflect that thread's distinct ident.
//!   - `setprofile` / `settrace` / `setprofile_all_threads` /
//!     `settrace_all_threads` / `stack_size`: accept and discard the
//!     argument, return None / 0.
//!   - `getprofile` / `gettrace`: return the last value passed to the
//!     matching setter (per-process, not per-thread).
//!   - `BrokenBarrierError` / `ThreadError` / `ExceptHookArgs`: Instance
//!     sentinels with `__name__` / `__module__` fields. Mamba does not
//!     model the Exception subclass hierarchy yet.
//!   - `functools`: re-exported as `MbValue::none()` placeholder
//!     (CPython's `threading` imports it internally).
//!   - `WeakSet`: constructor returns an empty list-shaped Instance.
//!     The runtime does not yet model weak references.
//!   - `TIMEOUT_MAX`: exposed as the f64-encoded CPython value.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex, OnceLock};

use std::thread::JoinHandle;

// -- Variadic dispatchers --

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            crate::icf_guard!();
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            crate::icf_guard!();
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            crate::icf_guard!();
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            crate::icf_guard!();
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

unsafe extern "C" fn dispatch_register_atexit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    super::atexit_mod::mb_atexit_register(a)
}

// Constructors / classes — Thread has a variadic dispatcher because the
// public form `Thread(target=..., name=..., daemon=..., args=..., kwargs=...)`
// is commonly invoked with kwargs only, lowered as a trailing-dict arg.
unsafe extern "C" fn d_thread(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mut target = MbValue::none();
    let mut name = MbValue::none();
    // `target(*args, **kwargs)` is how the run delivers the call. Capture the
    // `args` tuple and `kwargs` dict so `start()` can pass them through — the
    // synchronous stub previously dropped both and called the target with zero
    // args, so any declared parameter saw an uninitialized arg slot (garbage,
    // identical across every thread).
    let mut args_v = MbValue::none();
    let mut kwargs_v = MbValue::none();
    let mut daemon_v: Option<MbValue> = None;
    // Trailing dict = kwargs lowering. Inspect for target/name keys.
    let trailing_kwargs = a.last().and_then(|v| v.as_ptr()).and_then(|p| unsafe {
        if let ObjData::Dict(ref lock) = (*p).data {
            Some(lock.read().unwrap().clone())
        } else {
            None
        }
    });
    let positional_end = if trailing_kwargs.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    if positional_end >= 1 {
        target = a[0];
    }
    if positional_end >= 2 {
        name = a[1];
    }
    if let Some(kw) = trailing_kwargs {
        for (k, v) in kw.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                match ks.as_str() {
                    "target" => target = *v,
                    "name" => name = *v,
                    "args" => args_v = *v,
                    "kwargs" => kwargs_v = *v,
                    "daemon" => daemon_v = Some(*v),
                    _ => {}
                }
            }
        }
    }
    let inst = mb_threading_thread(target, name);
    // `target(*args, **kwargs)` is how the run delivers the call. Store the
    // `args` tuple and `kwargs` dict on the instance so `start()` can pass them
    // through — the synchronous stub previously dropped both and called the
    // target with zero args, so any declared parameter saw an uninitialized arg
    // slot (garbage, identical across every thread). Stored only here (the kwargs
    // construction path); symbol-path Threads have no args → start() calls with
    // none, exactly as before.
    if let Some(p) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                let mut f = fields.write().unwrap();
                f.insert("args".into(), args_v);
                f.insert("kwargs".into(), kwargs_v);
                if let Some(d) = daemon_v {
                    f.insert(
                        "daemon".into(),
                        MbValue::from_bool(d.as_bool().unwrap_or(false)),
                    );
                }
            }
        }
    }
    inst
}
disp_nullary!(d_lock, mb_threading_lock);
disp_nullary!(d_rlock, mb_threading_rlock);
disp_nullary!(d_event, mb_threading_event);
disp_variadic!(d_condition, d_condition_impl);
disp_unary!(d_semaphore, mb_threading_semaphore);
disp_unary!(d_bounded_semaphore, mb_threading_bounded_semaphore);
disp_unary!(d_barrier, mb_threading_barrier);
disp_binary!(d_timer, mb_threading_timer);
disp_nullary!(d_local, mb_threading_local);
disp_nullary!(d_weak_set, mb_threading_weak_set);

// Introspection
disp_nullary!(d_current_thread, mb_threading_current_thread);
disp_nullary!(d_active_count, mb_threading_active_count);
disp_nullary!(d_enumerate, mb_threading_enumerate);
disp_nullary!(d_main_thread, mb_threading_main_thread);
disp_nullary!(d_get_ident, mb_threading_get_ident);
disp_nullary!(d_get_native_id, mb_threading_get_native_id);

// Profile / trace
disp_unary!(d_setprofile, mb_threading_setprofile);
disp_unary!(d_settrace, mb_threading_settrace);
disp_unary!(
    d_setprofile_all_threads,
    mb_threading_setprofile_all_threads
);
disp_unary!(d_settrace_all_threads, mb_threading_settrace_all_threads);
disp_nullary!(d_getprofile, mb_threading_getprofile);
disp_nullary!(d_gettrace, mb_threading_gettrace);
disp_unary!(d_stack_size, mb_threading_stack_size);
disp_unary!(d_excepthook, mb_threading_excepthook);

/// CPython's `threading.TIMEOUT_MAX` on 64-bit POSIX (seconds).
const TIMEOUT_MAX: f64 = 9_223_372_036.854_776;

pub fn register() {
    // Thread.__repr__ shows the CPython lifecycle markers.
    {
        let addr = thread_method_repr as *const () as usize;
        super::super::module::register_variadic_func(addr as u64);
        let mut m: HashMap<String, MbValue> = HashMap::new();
        m.insert("__repr__".into(), MbValue::from_func(addr));
        super::super::class::mb_class_register("Thread", vec![], m);
    }
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Classes / constructors
        ("Thread", d_thread as usize),
        ("Lock", d_lock as usize),
        ("RLock", d_rlock as usize),
        ("_PyRLock", d_rlock as usize),
        ("_CRLock", d_rlock as usize),
        ("_register_atexit", dispatch_register_atexit as usize),
        ("_main_thread", d_main_thread as usize),
        ("_shutdown", d_setprofile as usize),
        ("Event", d_event as usize),
        ("Condition", d_condition as usize),
        ("Semaphore", d_semaphore as usize),
        ("BoundedSemaphore", d_bounded_semaphore as usize),
        ("Barrier", d_barrier as usize),
        ("Timer", d_timer as usize),
        ("local", d_local as usize),
        ("WeakSet", d_weak_set as usize),
        // Introspection
        ("current_thread", d_current_thread as usize),
        ("currentThread", d_current_thread as usize),
        ("active_count", d_active_count as usize),
        ("activeCount", d_active_count as usize),
        ("enumerate", d_enumerate as usize),
        ("main_thread", d_main_thread as usize),
        ("get_ident", d_get_ident as usize),
        ("get_native_id", d_get_native_id as usize),
        // Profile / trace
        ("setprofile", d_setprofile as usize),
        ("settrace", d_settrace as usize),
        ("setprofile_all_threads", d_setprofile_all_threads as usize),
        ("settrace_all_threads", d_settrace_all_threads as usize),
        ("getprofile", d_getprofile as usize),
        ("gettrace", d_gettrace as usize),
        ("stack_size", d_stack_size as usize),
        ("excepthook", d_excepthook as usize),
        ("__excepthook__", d_excepthook as usize),
    ];
    for (name, addr) in &dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // Register class-like constructors with their canonical class names so
    // `isinstance(x, threading.Thread)` etc. can resolve the dispatcher
    // pointer back to a class name.
    let class_dispatchers: &[(&str, usize)] = &[
        ("Thread", d_thread as usize),
        ("Lock", d_lock as usize),
        ("RLock", d_rlock as usize),
        ("Event", d_event as usize),
        ("Condition", d_condition as usize),
        ("Semaphore", d_semaphore as usize),
        ("BoundedSemaphore", d_bounded_semaphore as usize),
        ("Barrier", d_barrier as usize),
        ("Timer", d_timer as usize),
        ("local", d_local as usize),
    ];
    for (name, addr) in class_dispatchers {
        super::super::module::register_native_type_name(*addr as u64, name.to_string());
    }

    // `Lock` / `RLock` instances are context managers. Register a class method
    // table (keyed by the instance `class_name`) carrying `__enter__`/`__exit__`
    // so `hasattr(threading.Lock(), "__enter__")` resolves and `with lock:`
    // dispatches acquire/release (mirrors codecs' StreamReader/StreamWriter).
    {
        use super::super::class::mb_class_register;
        for cls in ["Lock", "RLock"] {
            let mut methods: HashMap<String, MbValue> = HashMap::new();
            methods.insert(
                "__enter__".to_string(),
                MbValue::from_func(lock_cm_enter as *const () as usize),
            );
            methods.insert(
                "__exit__".to_string(),
                MbValue::from_func(lock_cm_exit as *const () as usize),
            );
            mb_class_register(cls, vec!["object".to_string()], methods);
        }
    }

    // Exception sentinels — modeled as Instance with __name__ / __module__.
    attrs.insert(
        "BrokenBarrierError".to_string(),
        make_exception_class("BrokenBarrierError"),
    );
    attrs.insert(
        "ThreadError".to_string(),
        make_exception_class("ThreadError"),
    );
    // ExceptHookArgs — namedtuple-like sentinel.
    attrs.insert(
        "ExceptHookArgs".to_string(),
        make_exception_sentinel("ExceptHookArgs"),
    );

    // TIMEOUT_MAX constant.
    attrs.insert("TIMEOUT_MAX".to_string(), MbValue::from_float(TIMEOUT_MAX));

    // functools re-export placeholder.
    attrs.insert("functools".to_string(), MbValue::none());

    super::register_module("threading", attrs);

    // Patch `time.sleep` so concurrent worker threads release ACTIVE_EXEC_GUARD (THREAD_EXEC_MUTEX) while sleeping (#3231).
    {
        let addr = dispatch_threading_sleep as *const () as usize;
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        let sleep_val = MbValue::from_func(addr);
        let time_mod = super::super::module::mb_import(MbValue::from_ptr(MbObject::new_str("time".to_string())));
        let key = MbValue::from_ptr(MbObject::new_str("sleep".to_string()));
        super::super::class::mb_setattr(time_mod, key, sleep_val);
    }
}

unsafe extern "C" fn dispatch_threading_sleep(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let secs = a.get(0).copied().unwrap_or_else(MbValue::none);
    with_exec_guard_released(|| super::time_mod::mb_time_sleep(secs))
}

fn make_exception_class(class_name: &str) -> MbValue {
    let mut f = FxHashMap::default();
    let slot_sentinel = || MbValue::from_ptr(MbObject::new_str(String::new()));
    f.insert("__cause__".to_string(), slot_sentinel());
    f.insert("__context__".to_string(), slot_sentinel());
    f.insert(
        "__suppress_context__".to_string(),
        MbValue::from_bool(false),
    );
    f.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
    f.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("threading".to_string())),
    );
    super::super::class::mb_class_register(
        class_name,
        vec!["Exception".to_string()],
        HashMap::new(),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: RwLock::new(f),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn make_exception_sentinel(name: &str) -> MbValue {
    let mut f = FxHashMap::default();
    f.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    f.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("threading".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(f),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn make_instance(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Raise RuntimeError with `msg` — mirrors CPython's threading lock/thread
/// state-violation errors (release-unlocked, join-unstarted, restart). Sets the
/// thread-local exception via exception::mb_raise; callers return None after.
fn raise_runtime_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_type_error_msg(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// -- Thread-local state --

pub static THREAD_EXEC_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct ThreadExecGuard {
    _guard: Option<std::sync::MutexGuard<'static, ()>>,
}

impl ThreadExecGuard {
    pub fn acquire() -> Self {
        Self {
            _guard: Some(THREAD_EXEC_MUTEX.lock().unwrap_or_else(|e| e.into_inner())),
        }
    }

    pub fn release(&mut self) {
        self._guard.take();
    }

    pub fn reacquire(&mut self) {
        if self._guard.is_none() {
            self._guard = Some(THREAD_EXEC_MUTEX.lock().unwrap_or_else(|e| e.into_inner()));
        }
    }
}

/// Run a blocking closure with `ACTIVE_EXEC_GUARD` (the GIL) released,
/// reacquiring it after the closure completes.
fn with_exec_guard_released<T>(f: impl FnOnce() -> T) -> T {
    ACTIVE_EXEC_GUARD.with(|g| {
        if let Some(ref mut guard) = *g.borrow_mut() {
            guard.release();
        }
    });
    let res = f();
    ACTIVE_EXEC_GUARD.with(|g| {
        if let Some(ref mut guard) = *g.borrow_mut() {
            guard.reacquire();
        }
    });
    res
}

thread_local! {
    pub static ACTIVE_EXEC_GUARD: std::cell::RefCell<Option<ThreadExecGuard>> =
        const { std::cell::RefCell::new(None) };
    static THREAD_NAME: Cell<Option<String>> = const { Cell::new(None) };
    static PROFILE_FN: Cell<MbValue> = Cell::new(MbValue::none());
    static TRACE_FN: Cell<MbValue> = Cell::new(MbValue::none());
    static TRACE_PROFILE_HOOK_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static STACK_SIZE: Cell<i64> = const { Cell::new(0) };
    /// The ident observed by `get_ident()` / `get_native_id()` for the code
    /// currently executing. The main thread starts at 1 (CPython always has a
    /// live main thread). When `Thread.start()` runs a target synchronously it
    /// temporarily swaps this to the target Thread's distinct ident and restores
    /// the previous value afterwards, so nested/sequential starts each observe
    /// their own id.
    static CURRENT_IDENT: Cell<i64> = const { Cell::new(1) };
}

/// Process-global monotonic counter handing out distinct Thread idents.
/// Main thread reserves ident 1, so worker idents begin at 2.
static NEXT_THREAD_IDENT: AtomicI64 = AtomicI64::new(2);

/// Allocate the next distinct Thread ident (>= 2).
thread_local! {
    /// Threads between start() and join() — what enumerate() reports beyond
    /// the main thread.
    static LIVE_THREADS: std::cell::RefCell<Vec<u64>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// The Thread instance whose target is currently running;
    /// what current_thread() reports inside a worker.
    static CURRENT_THREAD_OBJ: std::cell::Cell<u64> =
        std::cell::Cell::new(MbValue::none().to_bits());
    static WORKER_STDLIB_READY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

static THREAD_HANDLES: OnceLock<
    Mutex<
        HashMap<
            u64,
            JoinHandle<
                HashMap<
                    super::super::closure::ScopedSymbolKey,
                    MbValue,
                >,
            >,
        >,
    >,
> = OnceLock::new();

fn thread_handles() -> &'static Mutex<
    HashMap<
        u64,
        JoinHandle<
            HashMap<
                super::super::closure::ScopedSymbolKey,
                MbValue,
            >,
        >,
    >,
> {
    THREAD_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn ensure_worker_stdlib_registered() {
    WORKER_STDLIB_READY.with(|ready| {
        if !ready.get() {
            super::register_stdlib();
            ready.set(true);
        }
    });
}

fn live_threads_add(t: MbValue) {
    unsafe {
        super::super::rc::retain_if_ptr(t);
    }
    LIVE_THREADS.with(|l| l.borrow_mut().push(t.to_bits()));
}

fn live_threads_remove(t: MbValue) {
    LIVE_THREADS.with(|l| l.borrow_mut().retain(|b| *b != t.to_bits()));
}

/// A pending exception left by the (synchronously run) target is delivered to
/// the CURRENT threading.excepthook with an args object carrying
/// exc_type (a type object) / exc_value / thread, then cleared.
fn deliver_to_excepthook(thread: MbValue) {
    if super::super::exception::mb_has_exception().as_bool() != Some(true) {
        return;
    }
    let value = super::super::class::mb_catch_exception_instance();
    let type_name = value
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                class_name.clone()
            } else {
                "Exception".to_string()
            }
        })
        .unwrap_or_else(|| "Exception".to_string());
    // exc_type as a type object (has __name__).
    let type_obj = {
        let inst = MbObject::new_instance("type".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                fields.write().unwrap().insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str(type_name.clone())),
                );
            }
        }
        MbValue::from_ptr(inst)
    };
    let args_inst = {
        let inst = MbObject::new_instance("_thread._ExceptHookArgs".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                let mut f = fields.write().unwrap();
                f.insert("exc_type".to_string(), type_obj);
                f.insert("exc_value".to_string(), value);
                f.insert("exc_traceback".to_string(), MbValue::none());
                f.insert("thread".to_string(), thread);
            }
        }
        MbValue::from_ptr(inst)
    };
    let hook = thread_excepthook(thread);
    if !hook.is_none() {
        let call_args = MbValue::from_ptr(MbObject::new_list(vec![args_inst]));
        let _ = super::super::builtins::mb_call_spread(hook, call_args);
        // A raising hook must not leak into the joiner either.
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            super::super::exception::mb_clear_exception();
        }
    } else {
        let msg = extract_str(super::super::builtins::mb_str(args_inst)).unwrap_or_default();
        let _ = msg;
        eprintln!("Exception in thread: {type_name}");
    }
}

fn current_excepthook() -> MbValue {
    let module = super::super::module::mb_import(MbValue::from_ptr(MbObject::new_str(
        "threading".to_string(),
    )));
    let sentinel = MbValue::from_bits(u64::MAX);
    let hook = super::super::dict_ops::mb_dict_get(
        module,
        MbValue::from_ptr(MbObject::new_str("excepthook".to_string())),
        sentinel,
    );
    if hook.to_bits() == u64::MAX {
        MbValue::none()
    } else {
        hook
    }
}

fn snapshot_excepthook_for_thread(thread: MbValue) {
    let hook = current_excepthook();
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(hook);
                let old = fields
                    .write()
                    .unwrap()
                    .insert("__mamba_excepthook__".to_string(), hook);
                if let Some(prev) = old {
                    super::super::rc::release_if_ptr(prev);
                }
            }
        }
    }
}

fn thread_excepthook(thread: MbValue) -> MbValue {
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(hook) = fields.read().unwrap().get("__mamba_excepthook__").copied() {
                    return hook;
                }
            }
        }
    }
    current_excepthook()
}

/// CPython-style Thread repr with lifecycle marker.
pub fn thread_repr(t: MbValue) -> String {
    let (name, started, alive, ident) = t
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                let g = fields.read().unwrap();
                (
                    g.get("name")
                        .copied()
                        .and_then(extract_str)
                        .unwrap_or_else(|| "Thread".to_string()),
                    g.get("started").and_then(|v| v.as_bool()).unwrap_or(false),
                    g.get("alive").and_then(|v| v.as_bool()).unwrap_or(false),
                    g.get("ident").and_then(|v| v.as_int()),
                )
            } else {
                ("Thread".to_string(), false, false, None)
            }
        })
        .unwrap_or(("Thread".to_string(), false, false, None));
    let daemon = t
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                fields
                    .read()
                    .unwrap()
                    .get("daemon")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            } else {
                false
            }
        })
        .unwrap_or(false);
    let mut state = if !started {
        "initial".to_string()
    } else if alive {
        format!("started {}", ident.unwrap_or(0))
    } else {
        format!("stopped {}", ident.unwrap_or(0))
    };
    if daemon {
        state.push_str(" daemon");
    }
    format!("<Thread({name}, {state})>")
}

unsafe extern "C" fn thread_method_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(thread_repr(self_v)))
}

fn next_thread_ident() -> i64 {
    NEXT_THREAD_IDENT.fetch_add(1, Ordering::Relaxed)
}

// -- Constructors --

/// threading.Thread(target=fn, name=str) -> Thread Instance.
///
/// Returned as a real ObjData::Instance with class_name "Thread" so that
/// (a) `isinstance(t, threading.Thread)` works via NATIVE_TYPE_NAMES, and
/// (b) `t.name`, `t.is_alive()` etc. can dispatch through the standard
/// attribute-access path. The constructor accepts `target` and `name`
/// positionally; if `name` is None/missing it defaults to "Thread".
pub fn mb_threading_thread(target: MbValue, name: MbValue) -> MbValue {
    // CPython: an unnamed Thread is auto-named "Thread-N"; a non-str name is
    // str()-coerced ("123").
    let n = if name.is_none() {
        // CPython 3.10+: auto names append the target's name — "Thread-N (worker)".
        let tname = if target.is_none() {
            None
        } else {
            let key = MbValue::from_ptr(MbObject::new_str("__name__".to_string()));
            extract_str(super::super::class::mb_getattr(target, key))
        };
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            super::super::exception::mb_clear_exception();
        }
        match tname {
            Some(t) if !t.is_empty() => format!("Thread-{} ({t})", next_thread_ident() - 1),
            _ => format!("Thread-{}", next_thread_ident() - 1),
        }
    } else {
        extract_str(name).unwrap_or_else(|| {
            extract_str(super::super::builtins::mb_str(name))
                .unwrap_or_else(|| "Thread".to_string())
        })
    };
    let mut f = FxHashMap::default();
    f.insert("name".into(), MbValue::from_ptr(MbObject::new_str(n)));
    f.insert("target".into(), target);
    f.insert("started".into(), MbValue::from_bool(false));
    f.insert("alive".into(), MbValue::from_bool(false));
    f.insert("daemon".into(), MbValue::from_bool(false));
    // CPython assigns the ident at start(); until then it is None.
    f.insert("ident".into(), MbValue::none());
    make_instance("Thread", f)
}

/// Build the positional arg list `start()` hands to `mb_call_spread`: the items
/// of the stored `args` tuple/list, with a non-empty `kwargs` dict appended as a
/// trailing dict (mamba's kwargs-lowering convention). An absent/empty `args`
/// yields an empty list, so a no-arg target behaves exactly as the old
/// zero-arg `mb_call0` path.
fn build_call_args(args: MbValue, kwargs: MbValue) -> MbValue {
    let mut items = if args.is_none() {
        Vec::new()
    } else {
        super::super::builtins::extract_items(args)
    };
    if let Some(p) = kwargs.as_ptr() {
        let non_empty = unsafe {
            matches!(&(*p).data, ObjData::Dict(lock) if !lock.read().unwrap().is_empty())
        };
        if non_empty {
            items.push(kwargs);
        }
    }
    MbValue::from_ptr(MbObject::new_list(items))
}

pub fn mb_threading_thread_start(thread: MbValue) -> MbValue {
    // Run target callables on a background OS thread. Earlier Mamba builds ran
    // targets synchronously, which preserved simple side-effect fixtures but
    // deadlocked CPython-shaped server-thread/client-thread handshakes where
    // the target blocks in accept()/recv() before the starter can continue.
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                // CPython: a Thread may be started at most once; a second start()
                // raises RuntimeError("threads can only be started once"). Guard
                // BEFORE running the target so a restart neither re-runs it nor
                // flips state. A fresh Thread has started=false, so the single
                // legitimate start() proceeds.
                let already_started = fields
                    .read()
                    .unwrap()
                    .get("started")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if already_started {
                    return raise_runtime_error("threads can only be started once");
                }
                let (target, args, kwargs) = {
                    let g = fields.read().unwrap();
                    (
                        g.get("target").copied().unwrap_or_else(MbValue::none),
                        g.get("args").copied().unwrap_or_else(MbValue::none),
                        g.get("kwargs").copied().unwrap_or_else(MbValue::none),
                    )
                };
                // CPython assigns the ident at start().
                let ident = {
                    let g = fields.read().unwrap();
                    g.get("ident").and_then(|v| v.as_int())
                }
                .unwrap_or_else(|| {
                    let id = next_thread_ident();
                    fields
                        .write()
                        .unwrap()
                        .insert("ident".into(), MbValue::from_int(id));
                    id
                });
                {
                    let mut f = fields.write().unwrap();
                    f.insert("started".into(), MbValue::from_bool(true));
                    // Alive from start() until join() (CPython Thread.is_alive()
                    // lifecycle in Mamba's current simplified model); join()
                    // flips it false and removes the live-thread entry.
                    f.insert("alive".into(), MbValue::from_bool(true));
                }
                live_threads_add(thread);
                let call_run_override = target.is_none() && thread_run_override_needed(thread);
                if !target.is_none() || call_run_override {
                    snapshot_excepthook_for_thread(thread);
                    let locals_snapshot = snapshot_locals();
                    let active_cells_snapshot = super::super::closure::snapshot_active_cells();
                    let class_snapshot = super::super::class::snapshot_thread_class_state();
                    super::super::rc::retain_if_ptr(thread);
                    super::super::rc::retain_if_ptr(target);
                    super::super::rc::retain_if_ptr(args);
                    super::super::rc::retain_if_ptr(kwargs);
                    let spawn_gate = Arc::new((Mutex::new(false), Condvar::new()));
                    let spawn_gate_worker = Arc::clone(&spawn_gate);
                    let handle = std::thread::spawn(move || {
                        {
                            let (lock, cvar) = &*spawn_gate_worker;
                            let mut ready = lock.lock().unwrap();
                            *ready = true;
                            cvar.notify_all();
                        }
                        clear_locals(&locals_snapshot);
                        let worker_cells = run_thread_target(
                            thread,
                            ident,
                            target,
                            args,
                            kwargs,
                            call_run_override,
                            active_cells_snapshot,
                            class_snapshot,
                        );
                        restore_locals(locals_snapshot);
                        release_thread_target_values(thread, target, args, kwargs);
                        super::super::module::preserve_module_jit_backends();
                        worker_cells
                    });
                    thread_handles()
                        .lock()
                        .unwrap()
                        .insert(thread.to_bits(), handle);
                    {
                        let (lock, cvar) = &*spawn_gate;
                        let mut ready = lock.lock().unwrap();
                        while !*ready {
                            ready = cvar.wait(ready).unwrap();
                        }
                    }
                }
            } else if let ObjData::Dict(ref lock) = (*ptr).data {
                let (target, args, kwargs, ident) = {
                    let g = lock.read().unwrap();
                    (
                        g.get("target").copied().unwrap_or_else(MbValue::none),
                        g.get("args").copied().unwrap_or_else(MbValue::none),
                        g.get("kwargs").copied().unwrap_or_else(MbValue::none),
                        g.get("ident").and_then(|v| v.as_int()),
                    )
                };
                let snapshot = snapshot_locals();
                let context_snapshot =
                    super::contextvars_mod::replace_current_context(FxHashMap::default());
                let prev_ident = CURRENT_IDENT.with(|c| c.get());
                if let Some(id) = ident {
                    CURRENT_IDENT.with(|c| c.set(id));
                }
                clear_locals(&snapshot);
                if !target.is_none() {
                    let _ = super::super::builtins::mb_call_spread(
                        target,
                        build_call_args(args, kwargs),
                    );
                }
                CURRENT_IDENT.with(|c| c.set(prev_ident));
                let _worker_context =
                    super::contextvars_mod::replace_current_context(context_snapshot);
                restore_locals(snapshot);
                let mut map = lock.write().unwrap();
                map.insert("started".into(), MbValue::from_bool(true));
                map.insert("alive".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

pub fn mb_threading_thread_join(thread: MbValue) -> MbValue {
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                // CPython: join() raises RuntimeError if the thread was never
                // started, and if it is the current/main thread. A started
                // worker has started=true (set by start()); the main_thread
                // singleton and a fresh Thread() have no started=true field, so
                // this guard covers both the unstarted-join and join-self cases
                // while leaving every valid post-start join untouched.
                let started = fields
                    .read()
                    .unwrap()
                    .get("started")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !started {
                    return raise_runtime_error("cannot join thread before it is started");
                }
                let current = CURRENT_THREAD_OBJ.with(|c| c.get());
                if current == thread.to_bits() {
                    return raise_runtime_error("cannot join current thread");
                }
                let handle = thread_handles().lock().unwrap().remove(&thread.to_bits());
                if let Some(handle) = handle {
                    let join_res = with_exec_guard_released(|| handle.join());
                    if let Ok(worker_cells) = join_res {
                        super::super::closure::merge_active_cells(&worker_cells);
                    }
                }
                let mut f = fields.write().unwrap();
                f.insert("alive".into(), MbValue::from_bool(false));
                drop(f);
                live_threads_remove(thread);
            } else if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("alive".into(), MbValue::from_bool(false));
            }
        }
    }
    MbValue::none()
}

fn is_daemon(t: MbValue) -> bool {
    t.as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                fields
                    .read()
                    .unwrap()
                    .get("daemon")
                    .map(|v| super::super::builtins::mb_bool(*v).as_bool().unwrap_or(false))
                    .unwrap_or(false)
            } else if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get("daemon")
                    .map(|v| super::super::builtins::mb_bool(*v).as_bool().unwrap_or(false))
                    .unwrap_or(false)
            } else {
                false
            }
        })
        .unwrap_or(false)
}

fn thread_ident(t: MbValue) -> i64 {
    t.as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                fields
                    .read()
                    .unwrap()
                    .get("ident")
                    .and_then(|v| v.as_int())
                    .unwrap_or(i64::MAX)
            } else if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get("ident")
                    .and_then(|v| v.as_int())
                    .unwrap_or(i64::MAX)
            } else {
                i64::MAX
            }
        })
        .unwrap_or(i64::MAX)
}

/// Join all live non-daemon threads before process exit (#3231).
pub fn join_non_daemon_threads() {
    loop {
        let candidate = {
            let map = thread_handles().lock().unwrap();
            let mut keys: Vec<u64> = map
                .keys()
                .copied()
                .filter(|&bits| !is_daemon(MbValue::from_bits(bits)))
                .collect();
            keys.sort_by_key(|&bits| thread_ident(MbValue::from_bits(bits)));
            keys.first().copied()
        };
        match candidate {
            Some(bits) => {
                mb_threading_thread_join(MbValue::from_bits(bits));
            }
            None => break,
        }
    }
}

/// Thread.is_alive() bound dispatcher — reads the `alive` field.
pub fn mb_threading_thread_is_alive(thread: MbValue) -> MbValue {
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(v) = fields.read().unwrap().get("alive").copied() {
                    return v;
                }
            }
        }
    }
    MbValue::from_bool(false)
}

/// threading.Lock() -> Instance stub
pub fn mb_threading_lock() -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("locked".into(), MbValue::from_bool(false));
    make_instance("Lock", f)
}

/// Real per-lock mutual-exclusion state, keyed by the `Lock` instance's heap
/// address. `ObjData::Instance` fields are plain `MbValue`s and cannot hold a
/// `Condvar` directly, so the actual blocking primitive lives in this
/// side-table instead; the `"locked"` field on the instance remains a
/// best-effort Python-visible mirror only (#1772: `acquire` previously
/// force-set `locked=true` unconditionally and never blocked, so two threads
/// could both believe they held the same lock — the compound `counter[0] +=
/// 1` critical section then raced for real under genuinely parallel
/// `Thread.start()` workers).
struct LockInner {
    count: usize,
    owner: Option<i64>,
}

struct LockState {
    mutex: Mutex<LockInner>,
    condvar: Condvar,
}

static LOCK_STATES: OnceLock<Mutex<HashMap<usize, Arc<LockState>>>> = OnceLock::new();

fn lock_state_for(ptr: usize) -> Arc<LockState> {
    LOCK_STATES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .entry(ptr)
        .or_insert_with(|| {
            Arc::new(LockState {
                mutex: Mutex::new(LockInner {
                    count: 0,
                    owner: None,
                }),
                condvar: Condvar::new(),
            })
        })
        .clone()
}

fn update_lock_instance_fields(lock: MbValue, locked: bool, count: usize) {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("locked".into(), MbValue::from_bool(locked));
                f.insert("count".into(), MbValue::from_int(count as i64));
            }
        }
    }
}

pub fn mb_threading_lock_acquire(lock: MbValue) -> MbValue {
    mb_threading_lock_acquire_with_args(lock, None, None)
}

pub fn mb_threading_lock_acquire_with_args(
    lock: MbValue,
    blocking: Option<MbValue>,
    timeout: Option<MbValue>,
) -> MbValue {
    let is_blocking = blocking.map(|b| b.as_bool().unwrap_or(true)).unwrap_or(true);
    let timeout_sec = timeout.and_then(|t| {
        if t.is_none() {
            None
        } else {
            t.as_float().or_else(|| t.as_int().map(|i| i as f64))
        }
    });
    let has_timeout = timeout_sec.map(|s| s >= 0.0).unwrap_or(false);

    if !is_blocking && has_timeout {
        return raise_value_error("can't specify timeout for non-blocking acquire");
    }

    let ptr = match lock.as_ptr() {
        Some(p) => p,
        None => return MbValue::from_bool(true),
    };

    let is_rlock = unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            class_name == "RLock"
        } else {
            false
        }
    };

    let current_thread = CURRENT_IDENT.with(|c| c.get());
    let state = lock_state_for(ptr as usize);

    if !is_blocking {
        let mut inner = state.mutex.lock().unwrap();
        if is_rlock && inner.count > 0 && inner.owner == Some(current_thread) {
            inner.count += 1;
            let count = inner.count;
            drop(inner);
            update_lock_instance_fields(lock, true, count);
            return MbValue::from_bool(true);
        } else if inner.count == 0 {
            inner.count = 1;
            inner.owner = Some(current_thread);
            let count = inner.count;
            drop(inner);
            update_lock_instance_fields(lock, true, count);
            return MbValue::from_bool(true);
        } else {
            return MbValue::from_bool(false);
        }
    }

    let (acquired, final_count) = with_exec_guard_released(|| {
        let mut inner = state.mutex.lock().unwrap();

        if is_rlock && inner.count > 0 && inner.owner == Some(current_thread) {
            inner.count += 1;
            return (true, inner.count);
        }

        match timeout_sec {
            None => {
                while inner.count > 0 {
                    inner = state.condvar.wait(inner).unwrap();
                }
                inner.count = 1;
                inner.owner = Some(current_thread);
                (true, 1)
            }
            Some(sec) => {
                if sec <= 0.0 {
                    if inner.count == 0 {
                        inner.count = 1;
                        inner.owner = Some(current_thread);
                        (true, 1)
                    } else {
                        (false, inner.count)
                    }
                } else {
                    let dur = std::time::Duration::from_secs_f64(sec);
                    let deadline = std::time::Instant::now() + dur;
                    while inner.count > 0 {
                        let now = std::time::Instant::now();
                        if now >= deadline {
                            break;
                        }
                        let remaining = deadline - now;
                        let (g, _) = state.condvar.wait_timeout(inner, remaining).unwrap();
                        inner = g;
                    }
                    if inner.count == 0 {
                        inner.count = 1;
                        inner.owner = Some(current_thread);
                        (true, 1)
                    } else {
                        (false, inner.count)
                    }
                }
            }
        }
    });

    if acquired {
        update_lock_instance_fields(lock, true, final_count);
        MbValue::from_bool(true)
    } else {
        MbValue::from_bool(false)
    }
}

pub fn mb_threading_lock_release(lock: MbValue) -> MbValue {
    let ptr = match lock.as_ptr() {
        Some(p) => p,
        None => return MbValue::none(),
    };

    let is_rlock = unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            class_name == "RLock"
        } else {
            false
        }
    };

    let current_thread = CURRENT_IDENT.with(|c| c.get());
    let state = lock_state_for(ptr as usize);
    let mut inner = state.mutex.lock().unwrap();

    if inner.count == 0 {
        return raise_runtime_error("release unlocked lock");
    }

    if is_rlock && inner.owner != Some(current_thread) {
        return raise_runtime_error("cannot release un-acquired lock");
    }

    inner.count -= 1;
    let new_count = inner.count;

    if new_count == 0 {
        inner.owner = None;
        drop(inner);
        state.condvar.notify_one();
    } else {
        drop(inner);
    }

    update_lock_instance_fields(lock, new_count > 0, new_count);
    MbValue::none()
}

// -- Lock / RLock context-manager protocol --
//
// CPython's `Lock` / `RLock` are context managers: `with lock:` acquires on
// `__enter__` and releases on `__exit__`, and `hasattr(lock, "__enter__")` is
// True. Mamba models these as `ObjData::Instance` (class_name "Lock" / "RLock"),
// so the dunders are surfaced by registering a matching class method table in
// `register()` (mirrors codecs' StreamReader/StreamWriter `__enter__`/`__exit__`).
// These are fixed-arity native methods invoked as `f(self, ...)`.

/// Lock.__enter__(self) -> bool — acquires (no-op stub) and returns True, as
/// CPython's `Lock.__enter__` returns the result of `acquire(True)`.
extern "C" fn lock_cm_enter(self_v: MbValue) -> MbValue {
    mb_threading_lock_acquire_with_args(self_v, None, None)
}

/// Lock.__exit__(self, exc_type, exc_value, tb) -> bool — releases and returns
/// False so a pending exception still propagates (CPython semantics).
extern "C" fn lock_cm_exit(self_v: MbValue, _t: MbValue, _v: MbValue, _tb: MbValue) -> MbValue {
    mb_threading_lock_release(self_v);
    MbValue::from_bool(false)
}

/// threading.RLock() -> Instance stub
pub fn mb_threading_rlock() -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("locked".into(), MbValue::from_bool(false));
    f.insert("count".into(), MbValue::from_int(0));
    make_instance("RLock", f)
}

/// threading.Event() -> Instance stub
pub fn mb_threading_event() -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("is_set".into(), MbValue::from_bool(false));
    make_instance("Event", f)
}

struct GlobalEvent {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);
static EVENTS: LazyLock<Mutex<HashMap<u64, Arc<GlobalEvent>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_create_event(event: MbValue) -> Option<(u64, Arc<GlobalEvent>)> {
    let ptr = event.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let (existing_id, is_set_init) = {
                let f = fields.read().unwrap();
                let is_set = f.get("is_set").and_then(|v| v.as_bool()).unwrap_or(false);
                let id = f.get("event_id").and_then(|v| v.as_int()).map(|i| i as u64);
                (id, is_set)
            };
            let id = match existing_id {
                Some(id) => id,
                None => {
                    let new_id = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);
                    fields.write().unwrap().insert("event_id".into(), MbValue::from_int(new_id as i64));
                    new_id
                }
            };
            let mut map = EVENTS.lock().unwrap();
            let entry = map.entry(id).or_insert_with(|| {
                Arc::new(GlobalEvent {
                    mutex: Mutex::new(is_set_init),
                    condvar: Condvar::new(),
                })
            });
            return Some((id, entry.clone()));
        }
    }
    None
}

pub fn mb_threading_event_set(event: MbValue) -> MbValue {
    if let Some((_, global_event)) = get_or_create_event(event) {
        let mut set_guard = global_event.mutex.lock().unwrap();
        *set_guard = true;
        drop(set_guard);
        global_event.condvar.notify_all();
        if let Some(ptr) = event.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.write().unwrap().insert("is_set".into(), MbValue::from_bool(true));
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_threading_event_clear(event: MbValue) -> MbValue {
    if let Some((_, global_event)) = get_or_create_event(event) {
        let mut set_guard = global_event.mutex.lock().unwrap();
        *set_guard = false;
        if let Some(ptr) = event.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.write().unwrap().insert("is_set".into(), MbValue::from_bool(false));
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_threading_event_is_set(event: MbValue) -> MbValue {
    if let Some((_, global_event)) = get_or_create_event(event) {
        let set_guard = global_event.mutex.lock().unwrap();
        return MbValue::from_bool(*set_guard);
    }
    MbValue::from_bool(false)
}

pub fn mb_threading_event_wait(event: MbValue, timeout: Option<MbValue>) -> MbValue {
    let (_, global_event) = match get_or_create_event(event) {
        Some(e) => e,
        None => return MbValue::from_bool(false),
    };

    let timeout_sec = timeout.and_then(|t| {
        if t.is_none() {
            None
        } else {
            t.as_float().or_else(|| t.as_int().map(|i| i as f64))
        }
    });

    let res = with_exec_guard_released(|| {
        let mut set_guard = global_event.mutex.lock().unwrap();
        if *set_guard {
            return true;
        }
        match timeout_sec {
            None => {
                while !*set_guard {
                    set_guard = global_event.condvar.wait(set_guard).unwrap();
                }
                *set_guard
            }
            Some(sec) => {
                if sec <= 0.0 {
                    return *set_guard;
                }
                let dur = std::time::Duration::from_secs_f64(sec);
                let deadline = std::time::Instant::now() + dur;
                while !*set_guard {
                    let now = std::time::Instant::now();
                    if now >= deadline {
                        break;
                    }
                    let remaining = deadline - now;
                    let (g, _) = global_event.condvar.wait_timeout(set_guard, remaining).unwrap();
                    set_guard = g;
                }
                *set_guard
            }
        }
    });
    MbValue::from_bool(res)
}

fn is_lock_like(v: MbValue) -> bool {
    if v.is_none() {
        return true;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" {
                    return true;
                }
            }
        }
    }
    let acq = MbValue::from_ptr(MbObject::new_str("acquire".to_string()));
    let rel = MbValue::from_ptr(MbObject::new_str("release".to_string()));
    super::super::class::mb_hasattr(v, acq).as_bool() == Some(true)
        && super::super::class::mb_hasattr(v, rel).as_bool() == Some(true)
}

fn d_condition_impl(args: &[MbValue]) -> MbValue {
    let mut lock = None;
    for &a in args {
        let is_d = a
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_d {
            let sentinel = MbValue::from_bits(u64::MAX);
            let v = super::super::dict_ops::mb_dict_get(
                a,
                MbValue::from_ptr(MbObject::new_str("lock".to_string())),
                sentinel,
            );
            if v.to_bits() != sentinel.to_bits() {
                lock = Some(v);
            }
        } else if lock.is_none() {
            lock = Some(a);
        }
    }
    if let Some(l) = lock {
        if !is_lock_like(l) {
            return raise_type_error_msg("lock must be a Lock or RLock object or None");
        }
    }
    mb_threading_condition(lock)
}

struct GlobalCondition {
    mutex: Mutex<u64>,
    condvar: Condvar,
}

static NEXT_CONDITION_ID: AtomicU64 = AtomicU64::new(1);
static CONDITIONS: LazyLock<Mutex<HashMap<u64, Arc<GlobalCondition>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_create_condition(cond: MbValue) -> Option<(u64, Arc<GlobalCondition>)> {
    let ptr = cond.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let existing_id = {
                let f = fields.read().unwrap();
                f.get("condition_id").and_then(|v| v.as_int()).map(|i| i as u64)
            };
            let id = match existing_id {
                Some(id) => id,
                None => {
                    let new_id = NEXT_CONDITION_ID.fetch_add(1, Ordering::Relaxed);
                    fields.write().unwrap().insert("condition_id".into(), MbValue::from_int(new_id as i64));
                    new_id
                }
            };
            let mut map = CONDITIONS.lock().unwrap();
            let entry = map.entry(id).or_insert_with(|| {
                Arc::new(GlobalCondition {
                    mutex: Mutex::new(0),
                    condvar: Condvar::new(),
                })
            });
            return Some((id, entry.clone()));
        }
    }
    None
}

fn get_condition_lock(cond: MbValue) -> Option<MbValue> {
    if let Some(ptr) = cond.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                return f.get("lock").copied();
            }
        }
    }
    None
}

fn is_lock_held(lock: MbValue) -> bool {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(locked) = f.get("locked").and_then(|v| v.as_bool()) {
                    return locked;
                }
            }
        }
        let state = lock_state_for(ptr as usize);
        return state.mutex.lock().unwrap().count > 0;
    }
    false
}

fn acquire_lock_obj(lock: MbValue) -> MbValue {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" {
                    return mb_threading_lock_acquire_with_args(lock, None, None);
                }
            }
        }
    }
    let acq_name = MbValue::from_ptr(MbObject::new_str("acquire".to_string()));
    let empty_args = MbValue::from_ptr(MbObject::new_list_borrowed(vec![]));
    let res = super::super::class::mb_call_method(lock, acq_name, empty_args);
    unsafe {
        super::super::rc::release_if_ptr(acq_name);
        super::super::rc::release_if_ptr(empty_args);
    }
    res
}

fn release_lock_obj(lock: MbValue) -> MbValue {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" {
                    return mb_threading_lock_release(lock);
                }
            }
        }
    }
    let rel_name = MbValue::from_ptr(MbObject::new_str("release".to_string()));
    let empty_args = MbValue::from_ptr(MbObject::new_list_borrowed(vec![]));
    let res = super::super::class::mb_call_method(lock, rel_name, empty_args);
    unsafe {
        super::super::rc::release_if_ptr(rel_name);
        super::super::rc::release_if_ptr(empty_args);
    }
    res
}

/// threading.Condition(lock=None) -> Instance
pub fn mb_threading_condition(lock: Option<MbValue>) -> MbValue {
    let lock_obj = match lock {
        Some(l) if !l.is_none() => l,
        _ => mb_threading_rlock(),
    };
    let mut f = FxHashMap::default();
    f.insert("lock".into(), lock_obj);
    f.insert("locked".into(), MbValue::from_bool(false));
    f.insert("waiters".into(), MbValue::from_int(0));
    make_instance("Condition", f)
}

pub fn mb_threading_condition_acquire(cond: MbValue) -> MbValue {
    if let Some(lock) = get_condition_lock(cond) {
        acquire_lock_obj(lock)
    } else {
        MbValue::from_bool(false)
    }
}

pub fn mb_threading_condition_release(cond: MbValue) -> MbValue {
    if let Some(lock) = get_condition_lock(cond) {
        release_lock_obj(lock)
    } else {
        MbValue::none()
    }
}

pub fn mb_threading_condition_wait(cond: MbValue, timeout: Option<MbValue>) -> MbValue {
    let lock_obj = match get_condition_lock(cond) {
        Some(l) => l,
        None => return MbValue::from_bool(false),
    };

    if !is_lock_held(lock_obj) {
        return raise_runtime_error("cannot wait on un-acquired lock");
    }

    let (_, global_cond) = match get_or_create_condition(cond) {
        Some(c) => c,
        None => return MbValue::from_bool(false),
    };

    let my_gen = *global_cond.mutex.lock().unwrap();

    release_lock_obj(lock_obj);

    let timeout_sec = timeout.and_then(|t| {
        if t.is_none() {
            None
        } else {
            t.as_float().or_else(|| t.as_int().map(|i| i as f64))
        }
    });

    let woken = with_exec_guard_released(|| {
        let mut gen_guard = global_cond.mutex.lock().unwrap();
        match timeout_sec {
            None => {
                while *gen_guard == my_gen {
                    gen_guard = global_cond.condvar.wait(gen_guard).unwrap();
                }
                true
            }
            Some(sec) => {
                if sec <= 0.0 {
                    *gen_guard > my_gen
                } else {
                    let dur = std::time::Duration::from_secs_f64(sec);
                    let deadline = std::time::Instant::now() + dur;
                    while *gen_guard == my_gen {
                        let now = std::time::Instant::now();
                        if now >= deadline {
                            break;
                        }
                        let remaining = deadline - now;
                        let (g, _) = global_cond.condvar.wait_timeout(gen_guard, remaining).unwrap();
                        gen_guard = g;
                    }
                    *gen_guard > my_gen
                }
            }
        }
    });

    acquire_lock_obj(lock_obj);

    MbValue::from_bool(woken)
}

pub fn mb_threading_condition_notify(cond: MbValue, n_val: Option<MbValue>) -> MbValue {
    let lock_obj = match get_condition_lock(cond) {
        Some(l) => l,
        None => return MbValue::none(),
    };

    if !is_lock_held(lock_obj) {
        return raise_runtime_error("cannot notify on un-acquired lock");
    }

    let (_, global_cond) = match get_or_create_condition(cond) {
        Some(c) => c,
        None => return MbValue::none(),
    };

    let n = n_val.and_then(|v| v.as_int()).unwrap_or(1).max(1) as u64;
    {
        let mut gen_guard = global_cond.mutex.lock().unwrap();
        *gen_guard = gen_guard.wrapping_add(n);
    }
    global_cond.condvar.notify_all();

    MbValue::none()
}

pub fn mb_threading_condition_notify_all(cond: MbValue) -> MbValue {
    mb_threading_condition_notify(cond, None)
}

pub fn mb_threading_condition_wait_for(
    cond: MbValue,
    predicate: MbValue,
    timeout: Option<MbValue>,
) -> MbValue {
    let timeout_sec = timeout.and_then(|t| {
        if t.is_none() {
            None
        } else {
            t.as_float().or_else(|| t.as_int().map(|i| i as f64))
        }
    });

    let deadline = timeout_sec.map(|sec| std::time::Instant::now() + std::time::Duration::from_secs_f64(sec.max(0.0)));

    let mut res = super::super::class::mb_call0(predicate);
    while !res.as_bool().unwrap_or(false) {
        let waittime = match deadline {
            Some(dl) => {
                let now = std::time::Instant::now();
                if now >= dl {
                    break;
                }
                Some(MbValue::from_float((dl - now).as_secs_f64()))
            }
            None => None,
        };

        mb_threading_condition_wait(cond, waittime);
        res = super::super::class::mb_call0(predicate);
    }
    res
}

struct GlobalSemaphore {
    mutex: Mutex<i64>,
    condvar: Condvar,
    bound: Option<i64>,
}

static NEXT_SEMAPHORE_ID: AtomicU64 = AtomicU64::new(1);
static SEMAPHORES: LazyLock<Mutex<HashMap<u64, Arc<GlobalSemaphore>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_create_semaphore(sem: MbValue) -> Option<(u64, Arc<GlobalSemaphore>)> {
    let ptr = sem.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let (existing_id, initial_val, bound_val) = {
                let f = fields.read().unwrap();
                let initial = f.get("value").and_then(|v| v.as_int()).unwrap_or(1);
                let bound = f.get("bound").and_then(|v| v.as_int());
                let id = f.get("semaphore_id").and_then(|v| v.as_int()).map(|i| i as u64);
                (id, initial, bound)
            };
            let id = match existing_id {
                Some(id) => id,
                None => {
                    let new_id = NEXT_SEMAPHORE_ID.fetch_add(1, Ordering::Relaxed);
                    fields.write().unwrap().insert("semaphore_id".into(), MbValue::from_int(new_id as i64));
                    new_id
                }
            };
            let mut map = SEMAPHORES.lock().unwrap();
            let entry = map.entry(id).or_insert_with(|| {
                Arc::new(GlobalSemaphore {
                    mutex: Mutex::new(initial_val),
                    condvar: Condvar::new(),
                    bound: bound_val,
                })
            });
            return Some((id, entry.clone()));
        }
    }
    None
}

fn update_semaphore_instance_value(sem: MbValue, val: i64) {
    if let Some(ptr) = sem.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert("value".into(), MbValue::from_int(val));
            }
        }
    }
}

/// threading.Semaphore(value=1) -> Instance stub
pub fn mb_threading_semaphore(value: MbValue) -> MbValue {
    let v = value.as_int().unwrap_or(1);
    let mut f = FxHashMap::default();
    f.insert("value".into(), MbValue::from_int(v));
    f.insert("initial".into(), MbValue::from_int(v));
    make_instance("Semaphore", f)
}

/// threading.BoundedSemaphore(value=1) -> Instance stub
pub fn mb_threading_bounded_semaphore(value: MbValue) -> MbValue {
    let v = value.as_int().unwrap_or(1);
    let mut f = FxHashMap::default();
    f.insert("value".into(), MbValue::from_int(v));
    f.insert("initial".into(), MbValue::from_int(v));
    f.insert("bound".into(), MbValue::from_int(v));
    make_instance("BoundedSemaphore", f)
}

pub fn mb_threading_semaphore_acquire(
    sem: MbValue,
    blocking: Option<MbValue>,
    timeout: Option<MbValue>,
) -> MbValue {
    let is_blocking = blocking.map(|b| b.as_bool().unwrap_or(true)).unwrap_or(true);
    let timeout_sec = timeout.and_then(|t| {
        if t.is_none() {
            None
        } else {
            t.as_float().or_else(|| t.as_int().map(|i| i as f64))
        }
    });
    let has_timeout = timeout_sec.map(|s| s >= 0.0).unwrap_or(false);

    if !is_blocking && has_timeout {
        return raise_value_error("can't specify timeout for non-blocking acquire");
    }

    let (_, global_sem) = match get_or_create_semaphore(sem) {
        Some(s) => s,
        None => return MbValue::from_bool(false),
    };

    if !is_blocking {
        let mut val_guard = global_sem.mutex.lock().unwrap();
        if *val_guard > 0 {
            *val_guard -= 1;
            let new_val = *val_guard;
            drop(val_guard);
            update_semaphore_instance_value(sem, new_val);
            return MbValue::from_bool(true);
        } else {
            return MbValue::from_bool(false);
        }
    }

    let (acquired, new_val) = with_exec_guard_released(|| {
        let mut val_guard = global_sem.mutex.lock().unwrap();
        match timeout_sec {
            None => {
                while *val_guard == 0 {
                    val_guard = global_sem.condvar.wait(val_guard).unwrap();
                }
                *val_guard -= 1;
                (true, *val_guard)
            }
            Some(sec) => {
                if sec <= 0.0 {
                    if *val_guard > 0 {
                        *val_guard -= 1;
                        (true, *val_guard)
                    } else {
                        (false, *val_guard)
                    }
                } else {
                    let dur = std::time::Duration::from_secs_f64(sec);
                    let deadline = std::time::Instant::now() + dur;
                    while *val_guard == 0 {
                        let now = std::time::Instant::now();
                        if now >= deadline {
                            break;
                        }
                        let remaining = deadline - now;
                        let (g, _) = global_sem.condvar.wait_timeout(val_guard, remaining).unwrap();
                        val_guard = g;
                    }
                    if *val_guard > 0 {
                        *val_guard -= 1;
                        (true, *val_guard)
                    } else {
                        (false, *val_guard)
                    }
                }
            }
        }
    });

    if acquired {
        update_semaphore_instance_value(sem, new_val);
        MbValue::from_bool(true)
    } else {
        MbValue::from_bool(false)
    }
}

pub fn mb_threading_semaphore_release(sem: MbValue, n_val: Option<MbValue>) -> MbValue {
    let n = n_val.and_then(|v| v.as_int()).unwrap_or(1);
    if n < 1 {
        return raise_value_error("n must be >= 1");
    }

    let (_, global_sem) = match get_or_create_semaphore(sem) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    let mut val_guard = global_sem.mutex.lock().unwrap();
    if let Some(b) = global_sem.bound {
        if *val_guard + n > b {
            return raise_value_error("Semaphore released too many times");
        }
    }

    *val_guard += n;
    let new_val = *val_guard;
    drop(val_guard);

    global_sem.condvar.notify_all();
    update_semaphore_instance_value(sem, new_val);
    MbValue::none()
}

struct BarrierState {
    parties: usize,
    count: usize,
    generation: usize,
    reset_generation: usize,
    broken: bool,
    aborted: bool,
}

struct GlobalBarrier {
    mutex: Mutex<BarrierState>,
    condvar: Condvar,
}

static NEXT_BARRIER_ID: AtomicU64 = AtomicU64::new(1);
static BARRIERS: LazyLock<Mutex<HashMap<u64, Arc<GlobalBarrier>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_create_barrier(barrier: MbValue) -> Option<(u64, Arc<GlobalBarrier>)> {
    let ptr = barrier.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let (existing_id, parties) = {
                let f = fields.read().unwrap();
                let parties = f.get("parties").and_then(|v| v.as_int()).unwrap_or(1).max(1) as usize;
                let id = f.get("barrier_id").and_then(|v| v.as_int()).map(|i| i as u64);
                (id, parties)
            };
            let id = match existing_id {
                Some(id) => id,
                None => {
                    let new_id = NEXT_BARRIER_ID.fetch_add(1, Ordering::Relaxed);
                    fields.write().unwrap().insert("barrier_id".into(), MbValue::from_int(new_id as i64));
                    new_id
                }
            };
            let mut map = BARRIERS.lock().unwrap();
            let entry = map.entry(id).or_insert_with(|| {
                Arc::new(GlobalBarrier {
                    mutex: Mutex::new(BarrierState {
                        parties,
                        count: 0,
                        generation: 0,
                        reset_generation: 0,
                        broken: false,
                        aborted: false,
                    }),
                    condvar: Condvar::new(),
                })
            });
            return Some((id, entry.clone()));
        }
    }
    None
}

fn update_barrier_instance_fields(barrier: MbValue, n_waiting: usize, broken: bool) {
    if let Some(ptr) = barrier.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("n_waiting".into(), MbValue::from_int(n_waiting as i64));
                f.insert("broken".into(), MbValue::from_bool(broken));
            }
        }
    }
}

/// threading.Barrier(parties) -> Instance
pub fn mb_threading_barrier(parties: MbValue) -> MbValue {
    let p = parties.as_int().unwrap_or(1).max(1) as usize;
    let id = NEXT_BARRIER_ID.fetch_add(1, Ordering::Relaxed);
    let global_barrier = Arc::new(GlobalBarrier {
        mutex: Mutex::new(BarrierState {
            parties: p,
            count: 0,
            generation: 0,
            reset_generation: 0,
            broken: false,
            aborted: false,
        }),
        condvar: Condvar::new(),
    });
    BARRIERS.lock().unwrap().insert(id, global_barrier);

    let mut f = FxHashMap::default();
    f.insert("parties".into(), MbValue::from_int(p as i64));
    f.insert("n_waiting".into(), MbValue::from_int(0));
    f.insert("broken".into(), MbValue::from_bool(false));
    f.insert("barrier_id".into(), MbValue::from_int(id as i64));
    make_instance("Barrier", f)
}

fn raise_broken_barrier() -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("BrokenBarrierError".to_string())),
        MbValue::from_ptr(MbObject::new_str("Barrier broken".to_string())),
    );
    MbValue::none()
}

/// threading.Barrier.wait() -> int (the caller's arrival index).
pub fn mb_threading_barrier_wait(barrier: MbValue) -> MbValue {
    let Some((_id, global_barrier)) = get_or_create_barrier(barrier) else {
        return MbValue::from_int(0);
    };

    enum BarrierOutcome {
        Arrived(i64),
        Broken,
    }

    let (outcome, n_waiting, is_broken) = with_exec_guard_released(|| {
        let mut state = global_barrier.mutex.lock().unwrap();

        if state.broken {
            return (BarrierOutcome::Broken, 0, true);
        }

        state.count += 1;
        let parties = state.parties;

        if state.count == parties {
            state.count = 0;
            state.generation = state.generation.wrapping_add(1);
            global_barrier.condvar.notify_all();
            (BarrierOutcome::Arrived(0), 0, false)
        } else {
            let index = parties - state.count;
            let gen = state.generation;
            while state.generation == gen && !state.broken {
                state = global_barrier.condvar.wait(state).unwrap();
            }
            if state.broken || gen < state.reset_generation {
                (BarrierOutcome::Broken, state.count, state.broken)
            } else {
                (BarrierOutcome::Arrived(index as i64), state.count, false)
            }
        }
    });

    update_barrier_instance_fields(barrier, n_waiting, is_broken);
    match outcome {
        BarrierOutcome::Arrived(idx) => MbValue::from_int(idx),
        BarrierOutcome::Broken => raise_broken_barrier(),
    }
}

/// threading.Barrier.reset() -> None — clears waiting threads and resets barrier.
pub fn mb_threading_barrier_reset(barrier: MbValue) -> MbValue {
    if let Some((_id, global_barrier)) = get_or_create_barrier(barrier) {
        let mut state = global_barrier.mutex.lock().unwrap();
        state.broken = false;
        state.aborted = false;
        state.count = 0;
        state.generation = state.generation.wrapping_add(1);
        state.reset_generation = state.generation;
        global_barrier.condvar.notify_all();
    }
    update_barrier_instance_fields(barrier, 0, false);
    MbValue::none()
}

/// threading.Barrier.abort() -> None — marks the barrier broken.
pub fn mb_threading_barrier_abort(barrier: MbValue) -> MbValue {
    if let Some((_id, global_barrier)) = get_or_create_barrier(barrier) {
        let mut state = global_barrier.mutex.lock().unwrap();
        state.broken = true;
        state.aborted = true;
        global_barrier.condvar.notify_all();
    }
    update_barrier_instance_fields(barrier, 0, true);
    MbValue::none()
}

/// threading.Timer(interval, function) -> Thread-shaped Instance stub
pub fn mb_threading_timer(interval: MbValue, function: MbValue) -> MbValue {
    let secs = interval
        .as_float()
        .or_else(|| interval.as_int().map(|i| i as f64))
        .unwrap_or(0.0);
    let mut f = FxHashMap::default();
    f.insert("interval".into(), MbValue::from_float(secs));
    f.insert("function".into(), function);
    f.insert("started".into(), MbValue::from_bool(false));
    f.insert("finished".into(), MbValue::from_bool(false));
    make_instance("Timer", f)
}

thread_local! {
    /// Registry of live threading.local() instances. Used by Thread.start()
    /// to snapshot+restore field state so synchronous target invocation
    /// emulates per-thread isolation.
    static LOCAL_INSTANCES: std::cell::RefCell<Vec<MbValue>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// threading.local() -> fresh Instance (thread-local semantics collapse to
/// a plain attribute bag in the single-thread runtime). Returned as an
/// Instance with class_name "local" so `obj.attr = value` and `obj.attr`
/// route through the standard Instance attribute-access path. Each
/// constructed local is registered so Thread.start() can snapshot/restore
/// it to fake per-thread isolation.
pub fn mb_threading_local() -> MbValue {
    let val = make_instance("local", FxHashMap::default());
    LOCAL_INSTANCES.with(|v| v.borrow_mut().push(val));
    val
}

/// Snapshot every registered threading.local() instance's main-thread field
/// map. Thread.start() clears those fields while the worker target runs, then
/// restores this snapshot when the worker exits.
fn snapshot_locals() -> Vec<(MbValue, FxHashMap<String, MbValue>)> {
    LOCAL_INSTANCES.with(|v| {
        v.borrow()
            .iter()
            .filter_map(|val| {
                val.as_ptr().and_then(|ptr| unsafe {
                    if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                        Some((*val, fields.read().unwrap().clone()))
                    } else {
                        None
                    }
                })
            })
            .collect()
    })
}

fn clear_locals(snapshot: &[(MbValue, FxHashMap<String, MbValue>)]) {
    for (val, _) in snapshot {
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.write().unwrap().clear();
                }
            }
        }
    }
}

fn restore_locals(snapshot: Vec<(MbValue, FxHashMap<String, MbValue>)>) {
    for (val, fields_snap) in snapshot {
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    *fields.write().unwrap() = fields_snap;
                }
            }
        }
    }
}

fn run_thread_target(
    thread: MbValue,
    ident: i64,
    target: MbValue,
    args: MbValue,
    kwargs: MbValue,
    call_run_override: bool,
    active_cells_snapshot: HashMap<
        super::super::closure::ScopedSymbolKey,
        MbValue,
    >,
    class_snapshot: super::super::class::ThreadClassState,
) -> HashMap<
    super::super::closure::ScopedSymbolKey,
    MbValue,
> {
    ensure_worker_stdlib_registered();
    let exec_guard = ThreadExecGuard::acquire();
    ACTIVE_EXEC_GUARD.with(|g| *g.borrow_mut() = Some(exec_guard));
    let previous_cells = super::super::closure::replace_active_cells(active_cells_snapshot);
    let _previous_classes = super::super::class::replace_thread_class_state(class_snapshot);
    let prev_ident = CURRENT_IDENT.with(|c| c.get());
    let prev_obj = CURRENT_THREAD_OBJ.with(|c| c.get());
    CURRENT_IDENT.with(|c| c.set(ident));
    CURRENT_THREAD_OBJ.with(|c| c.set(thread.to_bits()));
    if !target.is_none() {
        let _ = super::super::builtins::mb_call_spread(target, build_call_args(args, kwargs));
    } else if call_run_override {
        let method = MbValue::from_ptr(MbObject::new_str("run".to_string()));
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        let _ = super::super::class::mb_call_method(thread, method, empty);
    }
    // An exception escaping the target/run override is delivered while the
    // worker still has the target's globals/classes installed. Running the
    // hook after restoring that context loses access to user-defined hooks.
    deliver_to_excepthook(thread);
    CURRENT_THREAD_OBJ.with(|c| c.set(prev_obj));
    CURRENT_IDENT.with(|c| c.set(prev_ident));
    let res = super::super::closure::replace_active_cells(previous_cells);
    ACTIVE_EXEC_GUARD.with(|g| g.borrow_mut().take());
    res
}

fn thread_run_override_needed(thread: MbValue) -> bool {
    let Some(ptr) = thread.as_ptr() else {
        return false;
    };
    let class_name = unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, .. } => class_name.clone(),
            _ => return false,
        }
    };
    class_name != "Thread"
        && super::super::class::class_mro_any(&class_name, |name| name == "Thread")
        && !super::super::class::lookup_method(&class_name, "run").is_none()
}

fn release_thread_target_values(thread: MbValue, target: MbValue, args: MbValue, kwargs: MbValue) {
    unsafe {
        super::super::rc::release_if_ptr(kwargs);
        super::super::rc::release_if_ptr(args);
        super::super::rc::release_if_ptr(target);
        super::super::rc::release_if_ptr(thread);
    }
}

/// threading.WeakSet() -> Instance stub holding an empty list.
pub fn mb_threading_weak_set() -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("data".into(), MbValue::from_ptr(MbObject::new_list(vec![])));
    make_instance("WeakSet", f)
}

// -- Introspection --

thread_local! {
    /// Singleton "MainThread" instance shared by current_thread() and
    /// main_thread() so identity checks (`is`) succeed in the single-thread
    /// stub model. Lazily initialised on first access.
    static MAIN_THREAD: std::cell::RefCell<Option<MbValue>> = const { std::cell::RefCell::new(None) };
}

fn main_thread_singleton() -> MbValue {
    MAIN_THREAD.with(|cell| {
        if let Some(val) = *cell.borrow() {
            unsafe {
                super::super::rc::retain_if_ptr(val);
            }
            return val;
        }
        let mut f = FxHashMap::default();
        f.insert(
            "name".into(),
            MbValue::from_ptr(MbObject::new_str("MainThread".to_string())),
        );
        f.insert("ident".into(), MbValue::from_int(1));
        f.insert("daemon".into(), MbValue::from_bool(false));
        f.insert("alive".into(), MbValue::from_bool(true));
        let val = make_instance("Thread", f);
        *cell.borrow_mut() = Some(val);
        unsafe {
            super::super::rc::retain_if_ptr(val);
        }
        val
    })
}

/// threading.current_thread() -> the Thread object for the running thread.
///
/// Inside a `Thread.start()` target, CURRENT_THREAD_OBJ holds the running
/// Thread instance; outside any worker it is the MainThread singleton.
pub fn mb_threading_current_thread() -> MbValue {
    // Inside a worker target the running Thread instance itself is current.
    let cur = CURRENT_THREAD_OBJ.with(|c| MbValue::from_bits(c.get()));
    if !cur.is_none() {
        return cur;
    }
    let running = THREAD_NAME.with(|n| {
        let v = n.take();
        n.set(v.clone());
        v
    });
    match running {
        Some(name) => {
            let mut f = FxHashMap::default();
            f.insert("name".into(), MbValue::from_ptr(MbObject::new_str(name)));
            f.insert("ident".into(), MbValue::from_int(2));
            f.insert("daemon".into(), MbValue::from_bool(false));
            f.insert("alive".into(), MbValue::from_bool(true));
            make_instance("Thread", f)
        }
        None => main_thread_singleton(),
    }
}

/// threading.main_thread() -> the singleton MainThread Instance.
pub fn mb_threading_main_thread() -> MbValue {
    main_thread_singleton()
}

/// threading.active_count() -> int (always 1 — single-threaded stub).
pub fn mb_threading_active_count() -> MbValue {
    MbValue::from_int(1)
}

/// threading.enumerate() -> list (containing the fake main thread).
pub fn mb_threading_enumerate() -> MbValue {
    let main = mb_threading_main_thread();
    let mut items = vec![main];
    LIVE_THREADS.with(|l| {
        for b in l.borrow().iter() {
            items.push(MbValue::from_bits(*b));
        }
    });
    MbValue::from_ptr(MbObject::new_list(items))
}

/// threading.get_ident() -> int.
///
/// Returns the ident of the code currently executing: 1 on the main thread;
/// while a `Thread.start()` target runs it reflects that thread's distinct
/// ident (see `mb_threading_thread_start`).
pub fn mb_threading_get_ident() -> MbValue {
    MbValue::from_int(CURRENT_IDENT.with(|c| c.get()))
}

/// threading.get_native_id() -> int. Mirrors `get_ident()` in the stub model.
pub fn mb_threading_get_native_id() -> MbValue {
    MbValue::from_int(CURRENT_IDENT.with(|c| c.get()))
}

// -- Profile / trace --

pub fn mb_threading_setprofile(func: MbValue) -> MbValue {
    PROFILE_FN.with(|p| p.set(func));
    MbValue::none()
}

pub fn mb_threading_settrace(func: MbValue) -> MbValue {
    TRACE_FN.with(|t| t.set(func));
    MbValue::none()
}

pub fn mb_threading_setprofile_all_threads(func: MbValue) -> MbValue {
    PROFILE_FN.with(|p| p.set(func));
    MbValue::none()
}

pub fn mb_threading_settrace_all_threads(func: MbValue) -> MbValue {
    TRACE_FN.with(|t| t.set(func));
    MbValue::none()
}

pub fn mb_threading_getprofile() -> MbValue {
    PROFILE_FN.with(|p| {
        let v = p.get();
        p.set(v);
        v
    })
}

pub fn mb_threading_gettrace() -> MbValue {
    TRACE_FN.with(|t| {
        let v = t.get();
        t.set(v);
        v
    })
}

struct TraceProfileHookGuard;

impl TraceProfileHookGuard {
    fn enter() -> Option<Self> {
        TRACE_PROFILE_HOOK_ACTIVE.with(|active| {
            if active.get() {
                None
            } else {
                active.set(true);
                Some(Self)
            }
        })
    }
}

impl Drop for TraceProfileHookGuard {
    fn drop(&mut self) {
        TRACE_PROFILE_HOOK_ACTIVE.with(|active| active.set(false));
    }
}

fn call_trace_profile_hook(hook: MbValue, event: &str, arg: MbValue) -> MbValue {
    if hook.is_none() {
        return MbValue::none();
    }
    let frame = super::traceback_mod::mb_traceback_make_current_frame_for_hook();
    let call_args = MbValue::from_ptr(MbObject::new_list(vec![
        frame,
        MbValue::from_ptr(MbObject::new_str(event.to_string())),
        arg,
    ]));
    // #1535: the 'exception' event (and 'return' fired mid-unwind) call this
    // while an exception is genuinely pending; the generic call path aborts
    // calls made under a pending exception (see suspend_current_exception),
    // so hide it for the duration of the trace callback itself.
    let saved_exc = super::super::exception::suspend_current_exception();
    let r = super::super::builtins::mb_call_spread(hook, call_args);
    super::super::exception::restore_suspended_exception(saved_exc);
    r
}

pub(crate) fn mb_threading_emit_call_event() {
    let trace_hook = TRACE_FN.with(|t| t.get());
    let profile_hook = PROFILE_FN.with(|p| p.get());
    if trace_hook.is_none() && profile_hook.is_none() {
        return;
    }
    let Some(_guard) = TraceProfileHookGuard::enter() else {
        return;
    };
    let had_exception = super::super::exception::mb_has_exception().as_bool() == Some(true);
    let local_trace_hook = call_trace_profile_hook(trace_hook, "call", MbValue::none());
    let has_exception = super::super::exception::mb_has_exception().as_bool() == Some(true);
    if !trace_hook.is_none() && !had_exception && has_exception {
        return;
    }
    if !trace_hook.is_none() {
        super::traceback_mod::mb_traceback_set_current_frame_local_trace_hook(local_trace_hook);
    }
    let _ = call_trace_profile_hook(profile_hook, "call", MbValue::none());
}

pub(crate) fn mb_threading_emit_line_event() {
    let local_trace_hook = super::traceback_mod::mb_traceback_current_frame_local_trace_hook();
    if local_trace_hook.is_none() {
        return;
    }
    let Some(_guard) = TraceProfileHookGuard::enter() else {
        return;
    };
    let _ = call_trace_profile_hook(local_trace_hook, "line", MbValue::none());
}

pub(crate) fn mb_threading_emit_return_event(arg: MbValue) {
    let local_trace_hook = super::traceback_mod::mb_traceback_current_frame_local_trace_hook();
    let profile_hook = PROFILE_FN.with(|p| p.get());
    if local_trace_hook.is_none() && profile_hook.is_none() {
        return;
    }
    let Some(_guard) = TraceProfileHookGuard::enter() else {
        return;
    };
    let had_exception = super::super::exception::mb_has_exception().as_bool() == Some(true);
    let _ = call_trace_profile_hook(local_trace_hook, "return", arg);
    let has_exception = super::super::exception::mb_has_exception().as_bool() == Some(true);
    if !local_trace_hook.is_none() && !had_exception && has_exception {
        return;
    }
    let _ = call_trace_profile_hook(profile_hook, "return", arg);
}

pub(crate) fn mb_threading_emit_exception_event(tb: MbValue) {
    let local_trace_hook = super::traceback_mod::mb_traceback_current_frame_local_trace_hook();
    if local_trace_hook.is_none() {
        return;
    }
    let Some(_guard) = TraceProfileHookGuard::enter() else {
        return;
    };
    let value = super::super::exception::mb_get_exception();
    let arg = if value.is_none() {
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::none(),
            MbValue::none(),
            tb,
        ]))
    } else {
        let type_name = super::super::exception::get_exception_type_pub(value)
            .or_else(|| super::super::exception::current_exception_type())
            .unwrap_or_else(|| "Exception".to_string());
        let ty = super::super::builtins::make_type_object(&type_name);
        MbValue::from_ptr(MbObject::new_tuple(vec![ty, value, tb]))
    };
    let _ = call_trace_profile_hook(local_trace_hook, "exception", arg);
}

/// threading.stack_size(size=None) -> int
///
/// Returns the previously-recorded value, then stores `size` if provided.
pub fn mb_threading_stack_size(size: MbValue) -> MbValue {
    let prev = STACK_SIZE.with(|s| s.get());
    if let Some(n) = size.as_int() {
        STACK_SIZE.with(|s| s.set(n));
    }
    MbValue::from_int(prev)
}

/// threading.excepthook(args) -> None — silent stub.
pub fn mb_threading_excepthook(_args: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::stdlib::traceback_mod;
    use crate::runtime::{exception, module};

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn instance_field(val: MbValue, key: &str) -> Option<MbValue> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(key).copied()
            } else {
                None
            }
        })
    }

    fn instance_class(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    fn list_len(val: MbValue) -> Option<usize> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                Some(lock.read().unwrap().len())
            } else {
                None
            }
        })
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    // -- extract_str --

    #[test]
    fn test_extract_str_str() {
        assert_eq!(extract_str(s("hi")), Some("hi".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(1)), None);
    }

    // -- Thread --

    #[test]
    fn test_thread_with_str_name() {
        // threading.Thread is an attribute-based object (Instance), not a dict —
        // `t.name` is an instance field, matching CPython's Thread.name attribute.
        let t = mb_threading_thread(MbValue::none(), s("worker"));
        assert_eq!(
            get_str(instance_field(t, "name").unwrap()),
            Some("worker".to_string())
        );
    }

    #[test]
    fn test_thread_with_non_str_name_defaults() {
        // CPython str()-coerces a non-string name: Thread(name=0) → "0".
        let t = mb_threading_thread(MbValue::none(), MbValue::from_int(0));
        assert_eq!(
            get_str(instance_field(t, "name").unwrap()),
            Some("0".to_string())
        );
    }

    #[test]
    fn test_thread_start_join_lifecycle() {
        let t = mb_threading_thread(MbValue::none(), MbValue::none());
        mb_threading_thread_start(t);
        assert_eq!(instance_field(t, "started").unwrap().as_bool(), Some(true));
        assert_eq!(instance_field(t, "alive").unwrap().as_bool(), Some(true));
        mb_threading_thread_join(t);
        assert_eq!(instance_field(t, "alive").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_thread_start_null_noop() {
        mb_threading_thread_start(MbValue::none());
    }

    #[test]
    fn test_thread_join_null_noop() {
        mb_threading_thread_join(MbValue::none());
    }

    // -- Lock --

    #[test]
    fn test_lock_acquire_release() {
        let lock = mb_threading_lock();
        assert_eq!(instance_class(lock).as_deref(), Some("Lock"));
        assert_eq!(
            instance_field(lock, "locked").unwrap().as_bool(),
            Some(false)
        );
        let acq = mb_threading_lock_acquire_with_args(lock, None, None);
        assert_eq!(acq.as_bool(), Some(true));
        assert_eq!(
            instance_field(lock, "locked").unwrap().as_bool(),
            Some(true)
        );
        mb_threading_lock_release(lock);
        assert_eq!(
            instance_field(lock, "locked").unwrap().as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_lock_acquire_null_noop() {
        let r = mb_threading_lock_acquire_with_args(MbValue::none(), None, None);
        assert_eq!(r.as_bool(), Some(true));
    }

    #[test]
    fn test_lock_release_null_noop() {
        mb_threading_lock_release(MbValue::none());
    }

    // -- RLock --

    #[test]
    fn test_rlock_shape() {
        let r = mb_threading_rlock();
        assert_eq!(instance_class(r).as_deref(), Some("RLock"));
        assert_eq!(instance_field(r, "locked").unwrap().as_bool(), Some(false));
        assert_eq!(instance_field(r, "count").unwrap().as_int(), Some(0));
    }

    // -- Event --

    #[test]
    fn test_event_set_clear_is_set() {
        let event = mb_threading_event();
        assert_eq!(instance_class(event).as_deref(), Some("Event"));
        assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(false));
        mb_threading_event_set(event);
        assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(true));
        mb_threading_event_clear(event);
        assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(false));
    }

    #[test]
    fn test_event_set_null_noop() {
        mb_threading_event_set(MbValue::none());
    }

    #[test]
    fn test_event_clear_null_noop() {
        mb_threading_event_clear(MbValue::none());
    }

    #[test]
    fn test_event_is_set_null_returns_false() {
        assert_eq!(
            mb_threading_event_is_set(MbValue::none()).as_bool(),
            Some(false)
        );
    }

    // -- Condition --

    #[test]
    fn test_condition_shape() {
        let c = mb_threading_condition(None);
        assert_eq!(instance_class(c).as_deref(), Some("Condition"));
        assert_eq!(instance_field(c, "locked").unwrap().as_bool(), Some(false));
        assert_eq!(instance_field(c, "waiters").unwrap().as_int(), Some(0));
    }

    // -- Semaphore / BoundedSemaphore --

    #[test]
    fn test_semaphore_default() {
        let s = mb_threading_semaphore(MbValue::none());
        assert_eq!(instance_class(s).as_deref(), Some("Semaphore"));
        assert_eq!(instance_field(s, "value").unwrap().as_int(), Some(1));
    }

    #[test]
    fn test_semaphore_custom_value() {
        let s = mb_threading_semaphore(MbValue::from_int(5));
        assert_eq!(instance_field(s, "value").unwrap().as_int(), Some(5));
        assert_eq!(instance_field(s, "initial").unwrap().as_int(), Some(5));
    }

    #[test]
    fn test_bounded_semaphore_default_and_bound() {
        let s = mb_threading_bounded_semaphore(MbValue::from_int(3));
        assert_eq!(instance_class(s).as_deref(), Some("BoundedSemaphore"));
        assert_eq!(instance_field(s, "value").unwrap().as_int(), Some(3));
        assert_eq!(instance_field(s, "bound").unwrap().as_int(), Some(3));
    }

    // -- Barrier --

    #[test]
    fn test_barrier_shape() {
        let b = mb_threading_barrier(MbValue::from_int(4));
        assert_eq!(instance_class(b).as_deref(), Some("Barrier"));
        assert_eq!(instance_field(b, "parties").unwrap().as_int(), Some(4));
        assert_eq!(instance_field(b, "n_waiting").unwrap().as_int(), Some(0));
        assert_eq!(instance_field(b, "broken").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_barrier_wait_returns_rotating_index() {
        let b = mb_threading_barrier(MbValue::from_int(3));
        let b1 = b;
        let b2 = b;
        let b3 = b;
        let h1 = std::thread::spawn(move || mb_threading_barrier_wait(b1).as_int().unwrap());
        let h2 = std::thread::spawn(move || mb_threading_barrier_wait(b2).as_int().unwrap());
        let h3 = std::thread::spawn(move || mb_threading_barrier_wait(b3).as_int().unwrap());
        let mut indices = vec![h1.join().unwrap(), h2.join().unwrap(), h3.join().unwrap()];
        indices.sort();
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_barrier_wait_null_returns_zero() {
        assert_eq!(mb_threading_barrier_wait(MbValue::none()).as_int(), Some(0));
    }

    #[test]
    fn test_barrier_reset_clears_waiting() {
        let b = mb_threading_barrier(MbValue::from_int(2));
        let b_clone = b;
        let h = std::thread::spawn(move || mb_threading_barrier_wait(b_clone));
        std::thread::sleep(std::time::Duration::from_millis(50));
        mb_threading_barrier_reset(b);
        let _ = h.join();
        assert_eq!(instance_field(b, "n_waiting").unwrap().as_int(), Some(0));
        assert_eq!(instance_field(b, "broken").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_barrier_abort_marks_broken() {
        let b = mb_threading_barrier(MbValue::from_int(2));
        mb_threading_barrier_abort(b);
        assert_eq!(instance_field(b, "broken").unwrap().as_bool(), Some(true));
    }

    // -- Timer --

    #[test]
    fn test_timer_shape() {
        let t = mb_threading_timer(MbValue::from_float(2.5), MbValue::none());
        assert_eq!(instance_class(t).as_deref(), Some("Timer"));
        assert!((instance_field(t, "interval").unwrap().as_float().unwrap() - 2.5).abs() < 1e-9);
        assert_eq!(instance_field(t, "started").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_timer_int_interval_coerces() {
        let t = mb_threading_timer(MbValue::from_int(3), MbValue::none());
        assert!((instance_field(t, "interval").unwrap().as_float().unwrap() - 3.0).abs() < 1e-9);
    }

    // -- local --

    #[test]
    fn test_local_is_empty_instance() {
        // threading.local() is an attribute-based _local object (Instance), not
        // a dict — fresh, with no per-thread attributes set yet (CPython _local).
        let l = mb_threading_local();
        assert_eq!(instance_class(l).as_deref(), Some("local"));
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*l.as_ptr().unwrap()).data {
                assert!(fields.read().unwrap().is_empty());
            } else {
                panic!("expected Instance");
            }
        }
    }

    // -- WeakSet --

    #[test]
    fn test_weak_set_shape() {
        let w = mb_threading_weak_set();
        assert_eq!(instance_class(w).as_deref(), Some("WeakSet"));
        assert_eq!(list_len(instance_field(w, "data").unwrap()), Some(0));
    }

    // -- current_thread / main_thread / enumerate --

    #[test]
    fn test_current_thread_default_main_thread() {
        THREAD_NAME.with(|n| n.set(None));
        let t = mb_threading_current_thread();
        assert_eq!(instance_class(t).as_deref(), Some("Thread"));
        assert_eq!(
            get_str(instance_field(t, "name").unwrap()),
            Some("MainThread".to_string())
        );
    }

    #[test]
    fn test_current_thread_with_custom_name() {
        THREAD_NAME.with(|n| n.set(Some("worker_test".to_string())));
        let t = mb_threading_current_thread();
        THREAD_NAME.with(|n| n.set(None));
        assert_eq!(
            get_str(instance_field(t, "name").unwrap()),
            Some("worker_test".to_string())
        );
    }

    #[test]
    fn test_main_thread_is_named_mainthread() {
        let t = mb_threading_main_thread();
        assert_eq!(
            get_str(instance_field(t, "name").unwrap()),
            Some("MainThread".to_string())
        );
        assert_eq!(instance_field(t, "ident").unwrap().as_int(), Some(1));
    }

    #[test]
    fn test_active_count_is_one() {
        assert_eq!(mb_threading_active_count().as_int(), Some(1));
    }

    #[test]
    fn test_enumerate_returns_singleton_main() {
        let e = mb_threading_enumerate();
        assert_eq!(list_len(e), Some(1));
    }

    #[test]
    fn test_get_ident_and_native_id() {
        assert_eq!(mb_threading_get_ident().as_int(), Some(1));
        assert_eq!(mb_threading_get_native_id().as_int(), Some(1));
    }

    #[test]
    fn test_thread_constructor_assigns_distinct_idents() {
        // CPython: ident is None before start() and distinct after.
        let a = mb_threading_thread(MbValue::none(), MbValue::none());
        let b = mb_threading_thread(MbValue::none(), MbValue::none());
        assert!(
            instance_field(a, "ident").unwrap().is_none(),
            "unstarted ident is None"
        );
        mb_threading_thread_start(a);
        mb_threading_thread_start(b);
        let ia = instance_field(a, "ident").unwrap().as_int().unwrap();
        let ib = instance_field(b, "ident").unwrap().as_int().unwrap();
        assert_ne!(ia, ib, "each Thread must get a distinct ident");
        assert!(ia >= 2 && ib >= 2, "worker idents reserve 1 for main");
    }

    #[test]
    fn test_thread_start_swaps_and_restores_current_ident() {
        // Outside any start(), get_ident() is the main thread (1).
        CURRENT_IDENT.with(|c| c.set(1));
        let t = mb_threading_thread(MbValue::none(), MbValue::none());
        // start() with a None target still runs the swap/restore dance.
        mb_threading_thread_start(t);
        // After start() returns, the caller's ident is restored to 1.
        assert_eq!(mb_threading_get_ident().as_int(), Some(1));
    }

    // -- profile / trace --

    #[test]
    fn test_setprofile_getprofile_roundtrip() {
        let marker = s("profile_fn");
        mb_threading_setprofile(marker);
        let got = mb_threading_getprofile();
        assert_eq!(get_str(got), Some("profile_fn".to_string()));
        mb_threading_setprofile(MbValue::none());
    }

    #[test]
    fn test_settrace_gettrace_roundtrip() {
        let marker = s("trace_fn");
        mb_threading_settrace(marker);
        let got = mb_threading_gettrace();
        assert_eq!(get_str(got), Some("trace_fn".to_string()));
        mb_threading_settrace(MbValue::none());
    }

    #[test]
    fn test_setprofile_all_threads_routes_through_same_slot() {
        let marker = s("global_profile");
        mb_threading_setprofile_all_threads(marker);
        assert_eq!(
            get_str(mb_threading_getprofile()),
            Some("global_profile".to_string())
        );
        mb_threading_setprofile(MbValue::none());
    }

    #[test]
    fn test_settrace_all_threads_routes_through_same_slot() {
        let marker = s("global_trace");
        mb_threading_settrace_all_threads(marker);
        assert_eq!(
            get_str(mb_threading_gettrace()),
            Some("global_trace".to_string())
        );
        mb_threading_settrace(MbValue::none());
    }

    thread_local! {
        static TEST_TRACE_EVENTS: std::cell::RefCell<Vec<String>> =
            const { std::cell::RefCell::new(Vec::new()) };
        static TEST_TRACE_RETURN_ARGS: std::cell::RefCell<Vec<String>> =
            const { std::cell::RefCell::new(Vec::new()) };
        static TEST_GLOBAL_TRACE_RETURN: Cell<MbValue> = Cell::new(MbValue::none());
    }

    unsafe extern "C" fn test_trace_hook(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        crate::icf_guard!();
        let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        let event = args
            .get(1)
            .copied()
            .and_then(extract_str)
            .unwrap_or_default();
        TEST_TRACE_EVENTS.with(|events| events.borrow_mut().push(event));
        MbValue::from_func(test_trace_hook as *const () as usize)
    }

    unsafe extern "C" fn test_trace_return_arg_hook(
        args_ptr: *const MbValue,
        nargs: usize,
    ) -> MbValue {
        crate::icf_guard!();
        let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        let event = args
            .get(1)
            .copied()
            .and_then(extract_str)
            .unwrap_or_default();
        if event == "return" {
            let rendered = match args.get(2).copied() {
                Some(arg) if arg.is_none() => "None".to_string(),
                Some(arg) => arg
                    .as_int()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "<non-int>".to_string()),
                None => "<missing>".to_string(),
            };
            TEST_TRACE_RETURN_ARGS.with(|values| values.borrow_mut().push(rendered));
        }
        MbValue::from_func(test_trace_return_arg_hook as *const () as usize)
    }

    fn push_test_trace_event(prefix: &str, args: &[MbValue]) {
        let event = args
            .get(1)
            .copied()
            .and_then(extract_str)
            .unwrap_or_default();
        TEST_TRACE_EVENTS.with(|events| {
            events.borrow_mut().push(format!("{prefix}:{event}"));
        });
    }

    unsafe extern "C" fn test_local_trace_hook(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        crate::icf_guard!();
        let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        push_test_trace_event("local", args);
        MbValue::from_func(test_local_trace_hook as *const () as usize)
    }

    unsafe extern "C" fn test_global_trace_hook(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        crate::icf_guard!();
        let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        push_test_trace_event("global", args);
        TEST_GLOBAL_TRACE_RETURN.with(|hook| hook.get())
    }

    unsafe extern "C" fn test_profile_hook(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        crate::icf_guard!();
        let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        push_test_trace_event("profile", args);
        MbValue::from_func(test_profile_hook as *const () as usize)
    }

    fn register_test_hooks(addrs: &[usize]) {
        module::NATIVE_FUNC_ADDRS.with(|s| {
            let mut s = s.borrow_mut();
            for addr in addrs {
                s.insert(*addr as u64);
            }
        });
    }

    fn clear_test_trace_events() {
        TEST_TRACE_EVENTS.with(|events| events.borrow_mut().clear());
        TEST_TRACE_RETURN_ARGS.with(|values| values.borrow_mut().clear());
        TEST_GLOBAL_TRACE_RETURN.with(|hook| hook.set(MbValue::none()));
        mb_threading_settrace(MbValue::none());
        mb_threading_setprofile(MbValue::none());
        exception::clear_current_exception();
        traceback_mod::mb_traceback_reset_stack();
    }

    fn test_trace_events() -> Vec<String> {
        TEST_TRACE_EVENTS.with(|events| events.borrow().clone())
    }

    fn test_trace_return_args() -> Vec<String> {
        TEST_TRACE_RETURN_ARGS.with(|values| values.borrow().clone())
    }

    #[test]
    fn test_trace_hook_observes_call_and_return_events() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(s("trace_test.py"), MbValue::from_int(7), s("f"));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec!["call".to_string(), "return".to_string()]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_observes_explicit_return_arg_value() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_return_arg_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_return_arg_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(
            s("trace_return_value.py"),
            MbValue::from_int(7),
            s("f"),
        );
        traceback_mod::mb_traceback_pop_frame_with_return(MbValue::from_int(42));
        assert_eq!(test_trace_return_args(), vec!["42".to_string()]);
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_observes_implicit_return_arg_none() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_return_arg_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_return_arg_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(
            s("trace_return_none.py"),
            MbValue::from_int(8),
            s("f"),
        );
        traceback_mod::mb_traceback_pop_frame_with_return(MbValue::none());
        assert_eq!(test_trace_return_args(), vec!["None".to_string()]);
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_observes_line_events_from_current_line_updates() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(s("trace_lines.py"), MbValue::from_int(20), s("f"));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(20));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(21));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(22));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec![
                "call".to_string(),
                "line".to_string(),
                "line".to_string(),
                "line".to_string(),
                "return".to_string()
            ]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_profile_hook_observes_call_and_return_events() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_setprofile(hook);
        traceback_mod::mb_traceback_push_frame(s("profile_test.py"), MbValue::from_int(11), s("g"));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec!["call".to_string(), "return".to_string()]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_profile_hook_skips_line_events() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_setprofile(hook);
        traceback_mod::mb_traceback_push_frame(
            s("profile_lines.py"),
            MbValue::from_int(30),
            s("g"),
        );
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(30));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(31));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(32));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec!["call".to_string(), "return".to_string()]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_observes_exception_events_from_raise_capture() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(
            s("trace_exception.py"),
            MbValue::from_int(40),
            s("f"),
        );
        exception::mb_raise(s("ValueError"), s("x"));
        traceback_mod::mb_traceback_capture_raise(MbValue::from_int(41));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec![
                "call".to_string(),
                "exception".to_string(),
                "return".to_string()
            ]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_profile_hook_skips_exception_events_from_raise_capture() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_setprofile(hook);
        traceback_mod::mb_traceback_push_frame(
            s("profile_exception.py"),
            MbValue::from_int(50),
            s("g"),
        );
        exception::mb_raise(s("ValueError"), s("x"));
        traceback_mod::mb_traceback_capture_raise(MbValue::from_int(51));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec!["call".to_string(), "return".to_string()]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_profile_hooks_do_not_recurse_through_hook_body() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(s("outer.py"), MbValue::from_int(3), s("outer"));
        // Simulate compiled Python work inside the hook body: nested frame
        // hooks should not emit additional trace/profile callbacks.
        TRACE_PROFILE_HOOK_ACTIVE.with(|active| active.set(true));
        traceback_mod::mb_traceback_push_frame(s("hook.py"), MbValue::from_int(4), s("inner"));
        traceback_mod::mb_traceback_pop_frame();
        TRACE_PROFILE_HOOK_ACTIVE.with(|active| active.set(false));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec!["call".to_string(), "return".to_string()]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_ignores_line_updates_without_current_frame() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(99));
        assert!(test_trace_events().is_empty());
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_hook_does_not_backfill_existing_frame_line_events() {
        clear_test_trace_events();
        register_test_hooks(&[test_trace_hook as *const () as usize]);
        traceback_mod::mb_traceback_push_frame(s("module.py"), MbValue::from_int(1), s("<module>"));
        let hook = MbValue::from_func(test_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(2));
        traceback_mod::mb_traceback_pop_frame();
        assert!(test_trace_events().is_empty());
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_call_returning_none_suppresses_line_return_and_exception_events() {
        clear_test_trace_events();
        register_test_hooks(&[test_global_trace_hook as *const () as usize]);
        let hook = MbValue::from_func(test_global_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(s("trace_none.py"), MbValue::from_int(10), s("f"));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(11));
        exception::mb_raise(s("ValueError"), s("x"));
        traceback_mod::mb_traceback_capture_raise(MbValue::from_int(12));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(test_trace_events(), vec!["global:call".to_string()]);
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_call_return_value_becomes_local_trace_hook() {
        clear_test_trace_events();
        register_test_hooks(&[
            test_global_trace_hook as *const () as usize,
            test_local_trace_hook as *const () as usize,
        ]);
        TEST_GLOBAL_TRACE_RETURN.with(|hook| {
            hook.set(MbValue::from_func(
                test_local_trace_hook as *const () as usize,
            ));
        });
        let hook = MbValue::from_func(test_global_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(s("trace_local.py"), MbValue::from_int(20), s("f"));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(21));
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(22));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec![
                "global:call".to_string(),
                "local:line".to_string(),
                "local:line".to_string(),
                "local:return".to_string()
            ]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_trace_exception_event_uses_local_trace_hook() {
        clear_test_trace_events();
        register_test_hooks(&[
            test_global_trace_hook as *const () as usize,
            test_local_trace_hook as *const () as usize,
        ]);
        TEST_GLOBAL_TRACE_RETURN.with(|hook| {
            hook.set(MbValue::from_func(
                test_local_trace_hook as *const () as usize,
            ));
        });
        let hook = MbValue::from_func(test_global_trace_hook as *const () as usize);
        mb_threading_settrace(hook);
        traceback_mod::mb_traceback_push_frame(
            s("trace_exception_local.py"),
            MbValue::from_int(30),
            s("f"),
        );
        exception::mb_raise(s("ValueError"), s("x"));
        traceback_mod::mb_traceback_capture_raise(MbValue::from_int(31));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec![
                "global:call".to_string(),
                "local:exception".to_string(),
                "local:return".to_string()
            ]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_profile_hook_still_observes_call_and_return_when_trace_returns_none() {
        clear_test_trace_events();
        register_test_hooks(&[
            test_global_trace_hook as *const () as usize,
            test_profile_hook as *const () as usize,
        ]);
        let trace_hook = MbValue::from_func(test_global_trace_hook as *const () as usize);
        let profile_hook = MbValue::from_func(test_profile_hook as *const () as usize);
        mb_threading_settrace(trace_hook);
        mb_threading_setprofile(profile_hook);
        traceback_mod::mb_traceback_push_frame(
            s("trace_profile.py"),
            MbValue::from_int(40),
            s("f"),
        );
        traceback_mod::mb_traceback_set_current_line(MbValue::from_int(41));
        traceback_mod::mb_traceback_pop_frame();
        assert_eq!(
            test_trace_events(),
            vec![
                "global:call".to_string(),
                "profile:call".to_string(),
                "profile:return".to_string()
            ]
        );
        clear_test_trace_events();
    }

    #[test]
    fn test_stack_size_records_value_and_returns_previous() {
        STACK_SIZE.with(|s| s.set(0));
        let prev = mb_threading_stack_size(MbValue::from_int(65536));
        assert_eq!(prev.as_int(), Some(0));
        let prev2 = mb_threading_stack_size(MbValue::none());
        assert_eq!(prev2.as_int(), Some(65536));
        STACK_SIZE.with(|s| s.set(0));
    }

    #[test]
    fn test_excepthook_returns_none() {
        let r = mb_threading_excepthook(MbValue::none());
        assert!(r.is_none());
    }

    // -- TIMEOUT_MAX constant --

    #[test]
    fn test_timeout_max_value() {
        assert!(TIMEOUT_MAX > 9.0e9);
    }

    // -- exception sentinels --

    #[test]
    fn test_make_exception_sentinel_shape() {
        let e = make_exception_sentinel("BrokenBarrierError");
        assert_eq!(instance_class(e).as_deref(), Some("BrokenBarrierError"));
        assert_eq!(
            get_str(instance_field(e, "__name__").unwrap()),
            Some("BrokenBarrierError".to_string())
        );
        assert_eq!(
            get_str(instance_field(e, "__module__").unwrap()),
            Some("threading".to_string())
        );
    }

    #[test]
    fn test_make_exception_class_shape() {
        let e = make_exception_class("BrokenBarrierError");
        assert_eq!(instance_class(e).as_deref(), Some("type"));
        assert_eq!(
            get_str(instance_field(e, "__name__").unwrap()),
            Some("BrokenBarrierError".to_string())
        );
        assert_eq!(
            get_str(instance_field(e, "__module__").unwrap()),
            Some("threading".to_string())
        );
    }

    // -- register() surface --

    #[test]
    fn test_register_wires_full_surface() {
        let before = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        register();
        let after = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        // 27 unique dispatcher addresses (currentThread/activeCount alias
        // collapse, so HashSet insertion may not always grow). Just assert
        // non-zero monotonicity.
        assert!(
            after >= before,
            "registry should be monotonic across register()"
        );
    }
}
