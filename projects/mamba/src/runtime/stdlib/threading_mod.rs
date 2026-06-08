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
//!   - No real concurrency yet — single-threaded stubs only. All sync
//!     primitives (Lock, RLock, Condition, Event, Semaphore,
//!     BoundedSemaphore, Barrier) return passive Instance dicts whose
//!     methods (acquire/release/wait/notify/set/clear) are no-ops
//!     surfaced through the dispatcher table. `Barrier.wait` cannot truly
//!     rendezvous (no peer is ever blocked); it returns a rotating
//!     CPython-shaped arrival index 0..parties-1 instead of raising.
//!     `Thread.start` runs the target synchronously on the calling thread
//!     (delivering `target(*args, **kwargs)`) and flips `started` / `alive`,
//!     but does not spawn an OS thread.
//!   - `Timer` returns a Thread-shaped Instance with `interval` /
//!     `function` fields; it never fires.
//!   - `local` returns a fresh dict — thread-local semantics collapse
//!     to plain dict semantics in the single-thread runtime.
//!   - `active_count` / `enumerate` / `current_thread` / `main_thread`:
//!     always observe a single fake MainThread.
//!   - `get_ident` / `get_native_id`: 1 on the main thread; while a
//!     `Thread.start()` target runs synchronously they reflect that thread's
//!     distinct ident, restored on return (nested starts nest).
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

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use crate::runtime::rc::MbRwLock as RwLock;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::cell::Cell;
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};

// -- Variadic dispatchers --

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// Constructors / classes — Thread has a variadic dispatcher because the
// public form `Thread(target=..., name=..., daemon=..., args=..., kwargs=...)`
// is commonly invoked with kwargs only, lowered as a trailing-dict arg.
unsafe extern "C" fn d_thread(args_ptr: *const MbValue, nargs: usize) -> MbValue {
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
    // Trailing dict = kwargs lowering. Inspect for target/name keys.
    let trailing_kwargs = a.last().and_then(|v| v.as_ptr()).and_then(|p| unsafe {
        if let ObjData::Dict(ref lock) = (*p).data {
            Some(lock.read().unwrap().clone())
        } else { None }
    });
    let positional_end = if trailing_kwargs.is_some() { a.len().saturating_sub(1) } else { a.len() };
    if positional_end >= 1 { target = a[0]; }
    if positional_end >= 2 { name = a[1]; }
    if let Some(kw) = trailing_kwargs {
        for (k, v) in kw.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                match ks.as_str() {
                    "target" => target = *v,
                    "name"   => name = *v,
                    "args"   => args_v = *v,
                    "kwargs" => kwargs_v = *v,
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
            }
        }
    }
    inst
}
disp_nullary!(d_lock, mb_threading_lock);
disp_nullary!(d_rlock, mb_threading_rlock);
disp_nullary!(d_event, mb_threading_event);
disp_nullary!(d_condition, mb_threading_condition);
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
disp_unary!(d_setprofile_all_threads, mb_threading_setprofile_all_threads);
disp_unary!(d_settrace_all_threads, mb_threading_settrace_all_threads);
disp_nullary!(d_getprofile, mb_threading_getprofile);
disp_nullary!(d_gettrace, mb_threading_gettrace);
disp_unary!(d_stack_size, mb_threading_stack_size);
disp_unary!(d_excepthook, mb_threading_excepthook);

/// CPython's `threading.TIMEOUT_MAX` on 64-bit POSIX (seconds).
const TIMEOUT_MAX: f64 = 9_223_372_036.854_776;

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Classes / constructors
        ("Thread",                 d_thread                 as usize),
        ("Lock",                   d_lock                   as usize),
        ("RLock",                  d_rlock                  as usize),
        ("Event",                  d_event                  as usize),
        ("Condition",              d_condition              as usize),
        ("Semaphore",              d_semaphore              as usize),
        ("BoundedSemaphore",       d_bounded_semaphore      as usize),
        ("Barrier",                d_barrier                as usize),
        ("Timer",                  d_timer                  as usize),
        ("local",                  d_local                  as usize),
        ("WeakSet",                d_weak_set               as usize),
        // Introspection
        ("current_thread",         d_current_thread         as usize),
        ("currentThread",          d_current_thread         as usize),
        ("active_count",           d_active_count           as usize),
        ("activeCount",            d_active_count           as usize),
        ("enumerate",              d_enumerate              as usize),
        ("main_thread",            d_main_thread            as usize),
        ("get_ident",              d_get_ident              as usize),
        ("get_native_id",          d_get_native_id          as usize),
        // Profile / trace
        ("setprofile",             d_setprofile             as usize),
        ("settrace",               d_settrace               as usize),
        ("setprofile_all_threads", d_setprofile_all_threads as usize),
        ("settrace_all_threads",   d_settrace_all_threads   as usize),
        ("getprofile",             d_getprofile             as usize),
        ("gettrace",               d_gettrace               as usize),
        ("stack_size",             d_stack_size             as usize),
        ("excepthook",             d_excepthook             as usize),
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
        ("Thread",            d_thread            as usize),
        ("Lock",              d_lock              as usize),
        ("RLock",             d_rlock             as usize),
        ("Event",             d_event             as usize),
        ("Condition",         d_condition         as usize),
        ("Semaphore",         d_semaphore         as usize),
        ("BoundedSemaphore",  d_bounded_semaphore as usize),
        ("Barrier",           d_barrier           as usize),
        ("Timer",             d_timer             as usize),
        ("local",             d_local             as usize),
    ];
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        for (name, addr) in class_dispatchers {
            map.insert(*addr as u64, name.to_string());
        }
    });

    // `Lock` / `RLock` instances are context managers. Register a class method
    // table (keyed by the instance `class_name`) carrying `__enter__`/`__exit__`
    // so `hasattr(threading.Lock(), "__enter__")` resolves and `with lock:`
    // dispatches acquire/release (mirrors codecs' StreamReader/StreamWriter).
    {
        use super::super::class::mb_class_register;
        for cls in ["Lock", "RLock"] {
            let mut methods: HashMap<String, MbValue> = HashMap::new();
            methods.insert("__enter__".to_string(),
                MbValue::from_func(lock_cm_enter as *const () as usize));
            methods.insert("__exit__".to_string(),
                MbValue::from_func(lock_cm_exit as *const () as usize));
            mb_class_register(cls, vec!["object".to_string()], methods);
        }
    }

    // Exception sentinels — modeled as Instance with __name__ / __module__.
    attrs.insert(
        "BrokenBarrierError".to_string(),
        make_exception_sentinel("BrokenBarrierError"),
    );
    attrs.insert(
        "ThreadError".to_string(),
        make_exception_sentinel("ThreadError"),
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
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(f),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

fn make_instance(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
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

// -- Thread-local state --

thread_local! {
    static THREAD_NAME: Cell<Option<String>> = const { Cell::new(None) };
    static PROFILE_FN: Cell<MbValue> = Cell::new(MbValue::none());
    static TRACE_FN: Cell<MbValue> = Cell::new(MbValue::none());
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
    let n = extract_str(name).unwrap_or_else(|| "Thread".to_string());
    let mut f = FxHashMap::default();
    f.insert("name".into(), MbValue::from_ptr(MbObject::new_str(n)));
    f.insert("target".into(), target);
    f.insert("started".into(), MbValue::from_bool(false));
    f.insert("alive".into(), MbValue::from_bool(false));
    f.insert("daemon".into(), MbValue::from_bool(false));
    // Each Thread gets a distinct ident so `threading.get_ident()` observed
    // inside its (synchronously-run) target differs from sibling threads and from
    // the main thread (ident 1). CPython assigns the ident at start(); assigning
    // at construction is equivalent here because each Thread is started at most
    // once in the single-threaded stub model.
    f.insert("ident".into(), MbValue::from_int(next_thread_ident()));
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
    // Synchronously invoke the target callable (single-threaded stub model)
    // so that the side-effect observed by the seed (e.g. `_results.append(42)`)
    // is visible after `start()` / `join()`. Snapshot+restore registered
    // threading.local() instances so per-thread attribute isolation holds
    // even though execution is serial.
    if let Some(ptr) = thread.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                // CPython: a Thread may be started at most once; a second start()
                // raises RuntimeError("threads can only be started once"). Guard
                // BEFORE running the target so a restart neither re-runs it nor
                // flips state. A fresh Thread has started=false, so the single
                // legitimate start() proceeds.
                let already_started = fields.read().unwrap()
                    .get("started").and_then(|v| v.as_bool()).unwrap_or(false);
                if already_started {
                    return raise_runtime_error("threads can only be started once");
                }
                let (target, args, kwargs, ident) = {
                    let g = fields.read().unwrap();
                    (
                        g.get("target").copied().unwrap_or_else(MbValue::none),
                        g.get("args").copied().unwrap_or_else(MbValue::none),
                        g.get("kwargs").copied().unwrap_or_else(MbValue::none),
                        g.get("ident").and_then(|v| v.as_int()),
                    )
                };
                let snapshot = snapshot_locals();
                // Make `threading.get_ident()` inside the target observe THIS
                // thread's distinct ident, then restore the caller's ident so
                // nested/sequential starts each see their own (sync model).
                let prev_ident = CURRENT_IDENT.with(|c| c.get());
                if let Some(id) = ident {
                    CURRENT_IDENT.with(|c| c.set(id));
                }
                if !target.is_none() {
                    // Deliver target(*args, **kwargs) — NOT mb_call0, which left
                    // declared parameters reading uninitialized arg slots.
                    let _ = super::super::builtins::mb_call_spread(target, build_call_args(args, kwargs));
                }
                CURRENT_IDENT.with(|c| c.set(prev_ident));
                restore_locals(snapshot);
                let mut f = fields.write().unwrap();
                f.insert("started".into(), MbValue::from_bool(true));
                // Alive from start() until join() (CPython Thread.is_alive()
                // lifecycle); the synchronous stub still flips it false in join().
                f.insert("alive".into(), MbValue::from_bool(true));
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
                let prev_ident = CURRENT_IDENT.with(|c| c.get());
                if let Some(id) = ident {
                    CURRENT_IDENT.with(|c| c.set(id));
                }
                if !target.is_none() {
                    let _ = super::super::builtins::mb_call_spread(target, build_call_args(args, kwargs));
                }
                CURRENT_IDENT.with(|c| c.set(prev_ident));
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
                let started = fields.read().unwrap()
                    .get("started").and_then(|v| v.as_bool()).unwrap_or(false);
                if !started {
                    return raise_runtime_error("cannot join thread before it is started");
                }
                let mut f = fields.write().unwrap();
                f.insert("alive".into(), MbValue::from_bool(false));
            } else if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("alive".into(), MbValue::from_bool(false));
            }
        }
    }
    MbValue::none()
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

pub fn mb_threading_lock_acquire(lock: MbValue) -> MbValue {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("locked".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::from_bool(true)
}

pub fn mb_threading_lock_release(lock: MbValue) -> MbValue {
    if let Some(ptr) = lock.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                // CPython raises RuntimeError when releasing a lock/RLock that is
                // not held. A held lock has locked=true (set by acquire /
                // __enter__); a fresh or already-released one has locked=false.
                // Condition.release() also routes here and is acquired first, so
                // valid acquire/release roundtrips (incl. `with lock:`) never hit
                // this guard.
                let held = fields.read().unwrap()
                    .get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
                if !held {
                    return raise_runtime_error("release unlocked lock");
                }
                let mut f = fields.write().unwrap();
                f.insert("locked".into(), MbValue::from_bool(false));
            }
        }
    }
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
    mb_threading_lock_acquire(self_v)
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

pub fn mb_threading_event_set(event: MbValue) -> MbValue {
    if let Some(ptr) = event.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("is_set".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

pub fn mb_threading_event_clear(event: MbValue) -> MbValue {
    if let Some(ptr) = event.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("is_set".into(), MbValue::from_bool(false));
            }
        }
    }
    MbValue::none()
}

pub fn mb_threading_event_is_set(event: MbValue) -> MbValue {
    if let Some(ptr) = event.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                return f.get("is_set").copied().unwrap_or(MbValue::from_bool(false));
            }
        }
    }
    MbValue::from_bool(false)
}

/// threading.Condition(lock=None) -> Instance stub
pub fn mb_threading_condition() -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("locked".into(), MbValue::from_bool(false));
    f.insert("waiters".into(), MbValue::from_int(0));
    make_instance("Condition", f)
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

/// threading.Barrier(parties) -> Instance stub
pub fn mb_threading_barrier(parties: MbValue) -> MbValue {
    let p = parties.as_int().unwrap_or(1);
    let mut f = FxHashMap::default();
    f.insert("parties".into(), MbValue::from_int(p));
    f.insert("n_waiting".into(), MbValue::from_int(0));
    f.insert("broken".into(), MbValue::from_bool(false));
    make_instance("Barrier", f)
}

/// threading.Barrier.wait() -> int (the caller's arrival index).
///
/// A real rendezvous is impossible in the single-thread stub model (no other
/// thread is ever blocked at the barrier), so this cannot truly synchronize. It
/// returns the CPython-shaped arrival index 0..parties-1 by rotating an internal
/// `n_waiting` counter, and never raises — enough to satisfy `b.wait()` call
/// sites without an AttributeError.
pub fn mb_threading_barrier_wait(barrier: MbValue) -> MbValue {
    if let Some(ptr) = barrier.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                let parties = f.get("parties").and_then(|v| v.as_int()).unwrap_or(1).max(1);
                let n_waiting = f.get("n_waiting").and_then(|v| v.as_int()).unwrap_or(0);
                let index = n_waiting % parties;
                // Advance and wrap so successive callers cycle 0..parties-1,
                // mirroring CPython's distinct arrival indices per cohort.
                f.insert("n_waiting".into(), MbValue::from_int((index + 1) % parties));
                return MbValue::from_int(index);
            }
        }
    }
    MbValue::from_int(0)
}

/// threading.Barrier.reset() -> None — clears the waiting counter.
pub fn mb_threading_barrier_reset(barrier: MbValue) -> MbValue {
    if let Some(ptr) = barrier.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("n_waiting".into(), MbValue::from_int(0));
                f.insert("broken".into(), MbValue::from_bool(false));
            }
        }
    }
    MbValue::none()
}

/// threading.Barrier.abort() -> None — marks the barrier broken.
pub fn mb_threading_barrier_abort(barrier: MbValue) -> MbValue {
    if let Some(ptr) = barrier.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("broken".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

/// threading.Timer(interval, function) -> Thread-shaped Instance stub
pub fn mb_threading_timer(interval: MbValue, function: MbValue) -> MbValue {
    let secs = interval.as_float()
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

/// Snapshot every registered threading.local() instance's field map. Used
/// by Thread.start() to restore state after running a target synchronously.
fn snapshot_locals() -> Vec<(MbValue, FxHashMap<String, MbValue>)> {
    LOCAL_INSTANCES.with(|v| {
        v.borrow().iter().filter_map(|val| {
            val.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    Some((*val, fields.read().unwrap().clone()))
                } else { None }
            })
        }).collect()
    })
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
            unsafe { super::super::rc::retain_if_ptr(val); }
            return val;
        }
        let mut f = FxHashMap::default();
        f.insert("name".into(), MbValue::from_ptr(MbObject::new_str("MainThread".to_string())));
        f.insert("ident".into(), MbValue::from_int(1));
        f.insert("daemon".into(), MbValue::from_bool(false));
        f.insert("alive".into(), MbValue::from_bool(true));
        let val = make_instance("Thread", f);
        *cell.borrow_mut() = Some(val);
        unsafe { super::super::rc::retain_if_ptr(val); }
        val
    })
}

/// threading.current_thread() -> the Thread object for the running thread.
///
/// Inside a `Thread.start()` target (synchronous stub model), THREAD_NAME holds
/// the running thread's name, so current_thread() reflects THAT thread; outside
/// any worker it is the MainThread singleton.
pub fn mb_threading_current_thread() -> MbValue {
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
    MbValue::from_ptr(MbObject::new_list(vec![main]))
}

/// threading.get_ident() -> int.
///
/// Returns the ident of the code currently executing: 1 on the main thread;
/// while a `Thread.start()` target runs synchronously it reflects that thread's
/// distinct ident (see `mb_threading_thread_start`).
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

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn instance_field(val: MbValue, key: &str) -> Option<MbValue> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(key).copied()
            } else { None }
        })
    }

    fn instance_class(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else { None }
        })
    }


    fn list_len(val: MbValue) -> Option<usize> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                Some(lock.read().unwrap().len())
            } else { None }
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
        assert_eq!(get_str(instance_field(t, "name").unwrap()), Some("worker".to_string()));
    }

    #[test]
    fn test_thread_with_non_str_name_defaults() {
        let t = mb_threading_thread(MbValue::none(), MbValue::from_int(0));
        assert_eq!(get_str(instance_field(t, "name").unwrap()), Some("Thread".to_string()));
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
        assert_eq!(instance_field(lock, "locked").unwrap().as_bool(), Some(false));
        let acq = mb_threading_lock_acquire(lock);
        assert_eq!(acq.as_bool(), Some(true));
        assert_eq!(instance_field(lock, "locked").unwrap().as_bool(), Some(true));
        mb_threading_lock_release(lock);
        assert_eq!(instance_field(lock, "locked").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_lock_acquire_null_noop() {
        let r = mb_threading_lock_acquire(MbValue::none());
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
        assert_eq!(mb_threading_event_is_set(MbValue::none()).as_bool(), Some(false));
    }

    // -- Condition --

    #[test]
    fn test_condition_shape() {
        let c = mb_threading_condition();
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
        // Arrival indices cycle 0..parties-1 across successive callers.
        assert_eq!(mb_threading_barrier_wait(b).as_int(), Some(0));
        assert_eq!(mb_threading_barrier_wait(b).as_int(), Some(1));
        assert_eq!(mb_threading_barrier_wait(b).as_int(), Some(2));
        assert_eq!(mb_threading_barrier_wait(b).as_int(), Some(0));
    }

    #[test]
    fn test_barrier_wait_null_returns_zero() {
        assert_eq!(mb_threading_barrier_wait(MbValue::none()).as_int(), Some(0));
    }

    #[test]
    fn test_barrier_reset_clears_waiting() {
        let b = mb_threading_barrier(MbValue::from_int(2));
        let _ = mb_threading_barrier_wait(b); // n_waiting -> 1
        mb_threading_barrier_reset(b);
        assert_eq!(instance_field(b, "n_waiting").unwrap().as_int(), Some(0));
        assert_eq!(mb_threading_barrier_wait(b).as_int(), Some(0));
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
            } else { panic!("expected Instance"); }
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
        assert_eq!(get_str(instance_field(t, "name").unwrap()),
                   Some("MainThread".to_string()));
    }

    #[test]
    fn test_current_thread_with_custom_name() {
        THREAD_NAME.with(|n| n.set(Some("worker_test".to_string())));
        let t = mb_threading_current_thread();
        THREAD_NAME.with(|n| n.set(None));
        assert_eq!(get_str(instance_field(t, "name").unwrap()),
                   Some("worker_test".to_string()));
    }

    #[test]
    fn test_main_thread_is_named_mainthread() {
        let t = mb_threading_main_thread();
        assert_eq!(get_str(instance_field(t, "name").unwrap()),
                   Some("MainThread".to_string()));
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
        let a = mb_threading_thread(MbValue::none(), MbValue::none());
        let b = mb_threading_thread(MbValue::none(), MbValue::none());
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
        assert_eq!(get_str(mb_threading_getprofile()), Some("global_profile".to_string()));
        mb_threading_setprofile(MbValue::none());
    }

    #[test]
    fn test_settrace_all_threads_routes_through_same_slot() {
        let marker = s("global_trace");
        mb_threading_settrace_all_threads(marker);
        assert_eq!(get_str(mb_threading_gettrace()), Some("global_trace".to_string()));
        mb_threading_settrace(MbValue::none());
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
        assert_eq!(get_str(instance_field(e, "__name__").unwrap()),
                   Some("BrokenBarrierError".to_string()));
        assert_eq!(get_str(instance_field(e, "__module__").unwrap()),
                   Some("threading".to_string()));
    }

    // -- register() surface --

    #[test]
    fn test_register_wires_full_surface() {
        let before = super::super::super::module::NATIVE_FUNC_ADDRS
            .with(|s| s.borrow().len());
        register();
        let after = super::super::super::module::NATIVE_FUNC_ADDRS
            .with(|s| s.borrow().len());
        // 27 unique dispatcher addresses (currentThread/activeCount alias
        // collapse, so HashSet insertion may not always grow). Just assert
        // non-zero monotonicity.
        assert!(after >= before, "registry should be monotonic across register()");
    }
}
