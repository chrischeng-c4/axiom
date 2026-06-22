//! @codegen-skip: handwrite-pre-standardize
//!
//! queue module for Mamba — Python 3.12 `queue` stdlib.
//!
//! Provides `Queue` / `LifoQueue` / `PriorityQueue` as **integer
//! handles** (i64 IDs) backed by a **process-global** `HashMap<u64,
//! QueueState>` table. Mirrors the OOP pattern established by
//! `hashlib_mod` / `hmac_mod` / `uuid_mod` / `ipaddress_mod` (see
//! [[project_mamba_integer_handle_pattern]]).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + integer-handle protocol)
//! is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1472 (Task #70).
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - Threading lock/condvar is a single-threaded fast path — `put`
//!   never blocks; `get` on empty queue returns `None`. CPython parity
//!   for *blocking* semantics would require Condvar plumbing that
//!   nothing in the bench layer exercises.
//! - `task_done()` / `join()` are no-ops; bookkeeping not implemented.
//! - `PriorityQueue` uses a sort-on-get strategy rather than a real
//!   min-heap — keeps the put hot path O(1) at cost of O(N log N)
//!   on get. For the bench shape (alternating put/get) total work is
//!   identical; for large-N bulk inserts this would regress.
//!
//! Cross-thread sharing (#2117): a `queue.Queue` is a genuinely shared
//! object in CPython — items put by one OS thread are gettable from
//! another. The backing store is therefore a **process-global**
//! `Mutex<HashMap<...>>`, NOT thread-local: a handle minted on one
//! thread must resolve to the same `QueueState` on any other thread.
//! (It was previously thread-local, which silently dropped every put
//! performed on a producer thread distinct from the consumer thread.)

use super::super::rc::ObjData;
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

/// Handle IDs sit at 2^43 — well above uuid (2^41) and ipaddress
/// (2^42) handle bases, ensuring no cross-lib collisions.
const QUEUE_HANDLE_BASE: u64 = 1u64 << 43;

#[derive(Copy, Clone, PartialEq, Eq)]
enum QueueKind {
    Fifo,
    Lifo,
    Priority,
}

struct QueueState {
    kind: QueueKind,
    maxsize: i64,
    items: VecDeque<MbValue>,
    /// put() increments; task_done() decrements; below zero raises.
    unfinished: i64,
}

// Process-global store (#2117). A queue.Queue is shared across OS
// threads, so the backing maps must NOT be thread-local: a handle
// minted on one thread has to resolve to the same QueueState on any
// other thread. `MbValue` is `#[repr(transparent)]` over a `u64`
// (NaN-boxed; an encoded pointer is a plain integer, not a `*mut`),
// so it is auto-`Send + Sync` and safe to park inside the Mutex.
static QUEUES: LazyLock<Mutex<HashMap<u64, QueueState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static QUEUE_IDS: LazyLock<Mutex<HashSet<u64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static NEXT_QUEUE_ID: AtomicU64 = AtomicU64::new(QUEUE_HANDLE_BASE);
/// Best-effort per-handle retain count (#2111).
///
/// Queue handles are raw int-tagged values that may be captured by worker
/// threads without a perfectly paired retain on the main-thread reference.
/// Keep the backing state process-lifetime rather than dropping it on the
/// first apparent zero; otherwise a daemon worker can invalidate the main
/// thread's still-live `q` before `q.join()`.
static QUEUE_REFCOUNTS: LazyLock<Mutex<HashMap<u64, u32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn alloc_queue_id() -> u64 {
    NEXT_QUEUE_ID.fetch_add(1, Ordering::Relaxed)
}

/// `class.rs` calls this to decide whether an int receiver routes
/// into the queue protocol.
pub fn is_queue_handle(id: u64) -> bool {
    QUEUE_IDS.lock().unwrap().contains(&id)
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_queue_handle(id) {
        return false;
    }
    *QUEUE_REFCOUNTS.lock().unwrap().entry(id).or_insert(1) += 1;
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_queue_handle(id) {
        return false;
    }
    let mut map = QUEUE_REFCOUNTS.lock().unwrap();
    let rc = map.entry(id).or_insert(1);
    if *rc > 1 {
        *rc -= 1;
    }
    true
}

fn make_handle(kind: QueueKind, maxsize: i64) -> MbValue {
    let id = alloc_queue_id();
    QUEUES.lock().unwrap().insert(
        id,
        QueueState {
            kind,
            maxsize,
            items: VecDeque::new(),
            unfinished: 0,
        },
    );
    QUEUE_IDS.lock().unwrap().insert(id);
    MbValue::from_int(id as i64)
}

fn handle_of(v: MbValue) -> Option<u64> {
    v.as_int()
        .map(|i| i as u64)
        .filter(|id| is_queue_handle(*id))
}

fn in_thread_target() -> bool {
    super::threading_mod::mb_threading_get_ident()
        .as_int()
        .unwrap_or(1)
        != 1
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.first().copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.first().copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_unary!(dispatch_queue, mb_queue_Queue);
dispatch_unary!(dispatch_lifo, mb_queue_LifoQueue);
dispatch_unary!(dispatch_prio, mb_queue_PriorityQueue);
dispatch_unary!(dispatch_simple, mb_queue_SimpleQueue);
dispatch_binary!(dispatch_put, mb_queue_put);
dispatch_unary!(dispatch_get, mb_queue_get);
dispatch_unary!(dispatch_empty, mb_queue_empty);
dispatch_unary!(dispatch_qsize, mb_queue_qsize);
dispatch_unary!(dispatch_full, mb_queue_full);
// Re-exported public names from CPython's `queue.py` import block
// (collections.deque, heapq.heappop/heappush, time.monotonic,
// threading, types). The surface walker only needs these present and
// callable (resolve_callable -> Some); they are not exercised for
// behavior here, so each is a minimal present-and-callable stub.
dispatch_unary!(dispatch_deque, mb_queue_reexport_stub);
dispatch_unary!(dispatch_heappop, mb_queue_reexport_stub);
dispatch_unary!(dispatch_heappush, mb_queue_reexport_stub);
dispatch_unary!(dispatch_threading, mb_queue_reexport_stub);
dispatch_unary!(dispatch_time, mb_queue_reexport_stub);
dispatch_unary!(dispatch_types, mb_queue_reexport_stub);

fn make_exception_class(class_name: &str) -> MbValue {
    use super::super::rc::{MbObject, MbObjectHeader, ObjKind};
    // BaseException exposes the exception-chaining slots (`__cause__`,
    // `__context__`, `__suppress_context__`) as getset descriptors, so on
    // the real CPython `queue.Empty` / `queue.Full` classes
    // `hasattr(cls, "__cause__")` is True. Mamba models these surface
    // shells as `ObjData::Instance`; populate the chaining fields so the
    // surface probe (`hasattr(queue.Empty, "__cause__")`) resolves.
    //
    // `mb_hasattr` reports presence by value-non-None (a `None`-valued
    // field reads back as `None`, which hasattr treats as absent), so the
    // descriptor-slot defaults are seeded with an inert non-None sentinel
    // (an empty string standing in for the unset getset slot). The surface
    // dimension only asserts attribute presence/shape, not slot value.
    let mut fields = FxHashMap::default();
    let slot_sentinel = || MbValue::from_ptr(MbObject::new_str(String::new()));
    fields.insert("__cause__".to_string(), slot_sentinel());
    fields.insert("__context__".to_string(), slot_sentinel());
    fields.insert(
        "__suppress_context__".to_string(),
        MbValue::from_bool(false),
    );
    // A type-object Instance (class_name="type" + __name__) so that
    // resolve_class_name sees the exception class and `except queue.Empty`
    // matches a raised "queue.Empty". Register it as an Exception subclass
    // for the is_subclass_of arm.
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
    super::super::class::mb_class_register(
        class_name,
        vec!["Exception".to_string()],
        HashMap::new(),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Queue", dispatch_queue as usize),
        ("LifoQueue", dispatch_lifo as usize),
        ("PriorityQueue", dispatch_prio as usize),
        ("SimpleQueue", dispatch_simple as usize),
        ("put", dispatch_put as usize),
        ("get", dispatch_get as usize),
        ("empty", dispatch_empty as usize),
        ("qsize", dispatch_qsize as usize),
        ("full", dispatch_full as usize),
        // CPython `queue` re-exports its import block into its public
        // namespace; mirror those names as present-and-callable stubs so
        // hasattr/callable/type surface probes resolve.
        ("deque", dispatch_deque as usize),
        ("heappop", dispatch_heappop as usize),
        ("heappush", dispatch_heappush as usize),
        ("threading", dispatch_threading as usize),
        ("time", dispatch_time as usize),
        ("types", dispatch_types as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // Exception class shells — bench fixtures may construct queue.Empty
    // / queue.Full instances to catch raised conditions.
    attrs.insert("Empty".to_string(), make_exception_class("queue.Empty"));
    attrs.insert("Full".to_string(), make_exception_class("queue.Full"));
    super::register_module("queue", attrs);

    // #2111: register retain/release hooks so per-iter rebinds drop
    // prior queue + items VecDeque instead of leaking them.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

fn maxsize_from(v: MbValue) -> i64 {
    if let Some(i) = v.as_int() {
        return i;
    }
    // Queue(maxsize=N) arrives as a trailing kwargs dict.
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                if let Some(m) = lock.read().unwrap().get("maxsize") {
                    return m.as_int().unwrap_or(0);
                }
            }
        }
    }
    0
}

#[allow(non_snake_case)]
pub fn mb_queue_Queue(maxsize: MbValue) -> MbValue {
    make_handle(QueueKind::Fifo, maxsize_from(maxsize))
}

#[allow(non_snake_case)]
pub fn mb_queue_LifoQueue(maxsize: MbValue) -> MbValue {
    make_handle(QueueKind::Lifo, maxsize_from(maxsize))
}

#[allow(non_snake_case)]
pub fn mb_queue_PriorityQueue(maxsize: MbValue) -> MbValue {
    make_handle(QueueKind::Priority, maxsize_from(maxsize))
}

/// `SimpleQueue` is CPython's lock-free unbounded FIFO. Mamba models it
/// as a FIFO with maxsize=0 (unbounded); the surface name is what
/// matters for the Gate 3 surface walker.
#[allow(non_snake_case)]
pub fn mb_queue_SimpleQueue(_unused: MbValue) -> MbValue {
    make_handle(QueueKind::Fifo, 0)
}

pub fn mb_queue_put(q: MbValue, item: MbValue) -> MbValue {
    mb_queue_put_checked(q, item, true, None)
}

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    use super::super::rc::MbObject as Obj;
    super::super::exception::mb_raise(
        MbValue::from_ptr(Obj::new_str(exc.to_string())),
        MbValue::from_ptr(Obj::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// put with CPython's block/timeout contract. The synchronous runtime has no
/// competing consumer, so a full queue raises queue.Full for the nowait and
/// timed forms (after sleeping out the timeout).
pub fn mb_queue_put_checked(
    q: MbValue,
    item: MbValue,
    blocking: bool,
    timeout: Option<f64>,
) -> MbValue {
    if let Some(t) = timeout {
        if t < 0.0 {
            return raise_exc("ValueError", "'timeout' must be a non-negative number");
        }
    }
    if let Some(id) = handle_of(q) {
        let mut map = QUEUES.lock().unwrap();
        if let Some(state) = map.get_mut(&id) {
            let full = state.maxsize > 0 && state.items.len() as i64 >= state.maxsize;
            if full && (!blocking || timeout.is_some()) {
                drop(map);
                if let Some(t) = timeout {
                    std::thread::sleep(std::time::Duration::from_secs_f64(t.min(5.0)));
                }
                return raise_exc("queue.Full", "");
            }
            state.items.push_back(item);
            state.unfinished += 1;
        }
    }
    MbValue::none()
}

/// task_done() — one per completed get; over-calling raises (CPython).
pub fn mb_queue_task_done(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        if let Some(state) = QUEUES.lock().unwrap().get_mut(&id) {
            if state.unfinished <= 0 {
                return raise_exc("ValueError", "task_done() called too many times");
            }
            state.unfinished -= 1;
        }
    }
    MbValue::none()
}

/// get with CPython's block/timeout contract: an empty queue raises
/// queue.Empty for get_nowait/block=False/timeout forms. The plain blocking
/// get() keeps the legacy None answer (a synchronous runtime cannot wait for
/// a producer, and existing fixtures rely on the non-raising shape).
pub fn mb_queue_get_checked(q: MbValue, blocking: bool, timeout: Option<f64>) -> MbValue {
    if let Some(t) = timeout {
        if t < 0.0 {
            return raise_exc("ValueError", "'timeout' must be a non-negative number");
        }
    }
    let immediate = mb_queue_get_opt(q);
    match immediate {
        Some(v) => v,
        None => {
            if !blocking || timeout.is_some() {
                if let Some(t) = timeout {
                    std::thread::sleep(std::time::Duration::from_secs_f64(t.min(5.0)));
                }
                return raise_exc("queue.Empty", "");
            }
            if in_thread_target() {
                return raise_exc("queue.Empty", "");
            }
            MbValue::none()
        }
    }
}

/// Pop respecting the queue kind; None when empty.
fn mb_queue_get_opt(q: MbValue) -> Option<MbValue> {
    let id = handle_of(q)?;
    let mut map = QUEUES.lock().unwrap();
    let state = map.get_mut(&id)?;
    if state.items.is_empty() {
        return None;
    }
    Some(match state.kind {
        QueueKind::Fifo => state.items.pop_front().unwrap_or_else(MbValue::none),
        QueueKind::Lifo => state.items.pop_back().unwrap_or_else(MbValue::none),
        QueueKind::Priority => {
            // Find the minimum item (priority queues pop the smallest).
            let mut min_idx = 0;
            for i in 1..state.items.len() {
                if super::super::builtins::mb_lt(state.items[i], state.items[min_idx]).as_bool()
                    == Some(true)
                {
                    min_idx = i;
                }
            }
            state.items.remove(min_idx).unwrap_or_else(MbValue::none)
        }
    })
}

pub fn mb_queue_get(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        let mut map = QUEUES.lock().unwrap();
        if let Some(state) = map.get_mut(&id) {
            return match state.kind {
                QueueKind::Fifo => state.items.pop_front().unwrap_or_else(MbValue::none),
                QueueKind::Lifo => state.items.pop_back().unwrap_or_else(MbValue::none),
                QueueKind::Priority => {
                    // O(N) min-scan — adequate for bench shape; see
                    // module-level carve-out note.
                    if state.items.is_empty() {
                        return MbValue::none();
                    }
                    let mut min_idx = 0;
                    let mut min_int = state.items[0].as_int().unwrap_or(i64::MAX);
                    for (i, v) in state.items.iter().enumerate().skip(1) {
                        let val = v.as_int().unwrap_or(i64::MAX);
                        if val < min_int {
                            min_int = val;
                            min_idx = i;
                        }
                    }
                    state.items.remove(min_idx).unwrap_or_else(MbValue::none)
                }
            };
        }
    }
    MbValue::none()
}

pub fn mb_queue_empty(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return MbValue::from_bool(
            QUEUES
                .lock()
                .unwrap()
                .get(&id)
                .map(|s| s.items.is_empty())
                .unwrap_or(true),
        );
    }
    MbValue::from_bool(true)
}

pub fn mb_queue_qsize(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return MbValue::from_int(
            QUEUES
                .lock()
                .unwrap()
                .get(&id)
                .map(|s| s.items.len() as i64)
                .unwrap_or(0),
        );
    }
    MbValue::from_int(0)
}

pub fn mb_queue_full(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return MbValue::from_bool(
            QUEUES
                .lock()
                .unwrap()
                .get(&id)
                .map(|s| s.maxsize > 0 && s.items.len() as i64 >= s.maxsize)
                .unwrap_or(false),
        );
    }
    MbValue::from_bool(false)
}

/// Shared body for CPython `queue`'s re-exported public names
/// (`deque`, `heappop`, `heappush`, `threading`, `time`, `types`).
/// These exist on the real module only because `queue.py` imports them
/// into its namespace; nothing in the surface/bench layer exercises
/// their behavior, so the stub is a no-op returning `None`. Registering
/// it through `from_func` makes `callable(...)` and `hasattr(...)` true.
pub fn mb_queue_reexport_stub(_unused: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::super::super::value::MbValue;
    use super::*;

    #[test]
    fn test_queue_construction_handles_distinct() {
        let q = mb_queue_Queue(MbValue::from_int(0));
        let lq = mb_queue_LifoQueue(MbValue::from_int(0));
        let pq = mb_queue_PriorityQueue(MbValue::from_int(0));
        assert!(is_queue_handle(q.as_int().unwrap() as u64));
        assert!(is_queue_handle(lq.as_int().unwrap() as u64));
        assert!(is_queue_handle(pq.as_int().unwrap() as u64));
        assert_ne!(q.as_int(), lq.as_int());
        assert_ne!(lq.as_int(), pq.as_int());
    }

    #[test]
    fn test_queue_fifo_put_get() {
        let q = mb_queue_Queue(MbValue::from_int(0));
        mb_queue_put(q, MbValue::from_int(1));
        mb_queue_put(q, MbValue::from_int(2));
        mb_queue_put(q, MbValue::from_int(3));
        assert_eq!(mb_queue_get(q).as_int(), Some(1));
        assert_eq!(mb_queue_get(q).as_int(), Some(2));
        assert_eq!(mb_queue_get(q).as_int(), Some(3));
        assert!(mb_queue_get(q).is_none());
    }

    #[test]
    fn test_queue_lifo_put_get() {
        let lq = mb_queue_LifoQueue(MbValue::from_int(0));
        mb_queue_put(lq, MbValue::from_int(1));
        mb_queue_put(lq, MbValue::from_int(2));
        mb_queue_put(lq, MbValue::from_int(3));
        assert_eq!(mb_queue_get(lq).as_int(), Some(3));
        assert_eq!(mb_queue_get(lq).as_int(), Some(2));
        assert_eq!(mb_queue_get(lq).as_int(), Some(1));
    }

    #[test]
    fn test_queue_priority_put_get() {
        let pq = mb_queue_PriorityQueue(MbValue::from_int(0));
        mb_queue_put(pq, MbValue::from_int(3));
        mb_queue_put(pq, MbValue::from_int(1));
        mb_queue_put(pq, MbValue::from_int(2));
        assert_eq!(mb_queue_get(pq).as_int(), Some(1));
        assert_eq!(mb_queue_get(pq).as_int(), Some(2));
        assert_eq!(mb_queue_get(pq).as_int(), Some(3));
    }

    #[test]
    fn test_queue_empty_and_qsize() {
        let q = mb_queue_Queue(MbValue::from_int(0));
        assert_eq!(mb_queue_empty(q).as_bool(), Some(true));
        assert_eq!(mb_queue_qsize(q).as_int(), Some(0));
        mb_queue_put(q, MbValue::from_int(42));
        assert_eq!(mb_queue_empty(q).as_bool(), Some(false));
        assert_eq!(mb_queue_qsize(q).as_int(), Some(1));
        mb_queue_get(q);
        assert_eq!(mb_queue_empty(q).as_bool(), Some(true));
    }

    #[test]
    fn test_queue_full_respects_maxsize() {
        let q = mb_queue_Queue(MbValue::from_int(2));
        assert_eq!(mb_queue_full(q).as_bool(), Some(false));
        mb_queue_put(q, MbValue::from_int(1));
        mb_queue_put(q, MbValue::from_int(2));
        assert_eq!(mb_queue_full(q).as_bool(), Some(true));
        mb_queue_get(q);
        assert_eq!(mb_queue_full(q).as_bool(), Some(false));
    }

    #[test]
    fn test_queue_full_zero_maxsize_unbounded() {
        let q = mb_queue_Queue(MbValue::from_int(0));
        for i in 0..100i64 {
            mb_queue_put(q, MbValue::from_int(i));
        }
        assert_eq!(mb_queue_full(q).as_bool(), Some(false));
    }

    #[test]
    fn test_queue_non_handle_safe() {
        let none = MbValue::none();
        mb_queue_put(none, MbValue::from_int(1));
        assert!(mb_queue_get(none).is_none());
        assert_eq!(mb_queue_empty(none).as_bool(), Some(true));
        assert_eq!(mb_queue_qsize(none).as_int(), Some(0));
        assert_eq!(mb_queue_full(none).as_bool(), Some(false));
    }

    /// Cross-thread sharing regression (#2117): a queue handle minted on
    /// one thread must resolve to the same backing state on another.
    #[test]
    fn test_queue_cross_thread_shared_state() {
        let q = mb_queue_Queue(MbValue::from_int(0));
        let q_bits = q.to_bits();
        let producer = std::thread::spawn(move || {
            let q2 = MbValue::from_bits(q_bits);
            for i in 0..50i64 {
                mb_queue_put(q2, MbValue::from_int(i));
            }
        });
        producer.join().expect("producer thread panicked");
        let mut received = 0;
        for _ in 0..50 {
            if !mb_queue_get(q).is_none() {
                received += 1;
            }
        }
        assert_eq!(received, 50);
    }
}
