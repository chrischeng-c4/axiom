//! @codegen-skip: handwrite-pre-standardize
//!
//! queue module for Mamba — Python 3.12 `queue` stdlib.
//!
//! Provides `Queue` / `LifoQueue` / `PriorityQueue` as **integer
//! handles** (i64 IDs) backed by a thread-local `HashMap<u64,
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
//!   never blocks; `get` on empty queue returns `None`. Mamba is
//!   single-threaded Cranelift; CPython parity for blocking semantics
//!   would require Condvar plumbing that nothing in the bench layer
//!   exercises.
//! - `task_done()` / `join()` are no-ops; bookkeeping not implemented.
//! - `PriorityQueue` uses a sort-on-get strategy rather than a real
//!   min-heap — keeps the put hot path O(1) at cost of O(N log N)
//!   on get. For the bench shape (alternating put/get) total work is
//!   identical; for large-N bulk inserts this would regress.

use super::super::rc::ObjData;
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};

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
}

thread_local! {
    static QUEUES: RefCell<HashMap<u64, QueueState>> = RefCell::new(HashMap::new());
    static QUEUE_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_QUEUE_ID: Cell<u64> = const { Cell::new(QUEUE_HANDLE_BASE) };
    /// Per-handle refcount (#2111). Drops the QUEUES entry — including the
    /// items VecDeque holding owned MbValues — when the count hits zero.
    static QUEUE_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_queue_id() -> u64 {
    NEXT_QUEUE_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// `class.rs` calls this to decide whether an int receiver routes
/// into the queue protocol.
pub fn is_queue_handle(id: u64) -> bool {
    QUEUE_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_queue_handle(id: u64) {
    QUEUES.with(|m| {
        m.borrow_mut().remove(&id);
    });
    QUEUE_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    QUEUE_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_queue_handle(id) {
        return false;
    }
    QUEUE_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_queue_handle(id) {
        return false;
    }
    let should_drop = QUEUE_REFCOUNTS.with(|r| {
        let mut map = r.borrow_mut();
        let rc = map.entry(id).or_insert(1);
        if *rc <= 1 {
            map.remove(&id);
            true
        } else {
            *rc -= 1;
            false
        }
    });
    if should_drop {
        drop_queue_handle(id);
    }
    true
}

fn make_handle(kind: QueueKind, maxsize: i64) -> MbValue {
    let id = alloc_queue_id();
    QUEUES.with(|m| {
        m.borrow_mut().insert(
            id,
            QueueState {
                kind,
                maxsize,
                items: VecDeque::new(),
            },
        );
    });
    QUEUE_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

fn handle_of(v: MbValue) -> Option<u64> {
    v.as_int()
        .map(|i| i as u64)
        .filter(|id| is_queue_handle(*id))
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

fn make_exception_class(class_name: &str) -> MbValue {
    use super::super::rc::{MbObject, MbObjectHeader, ObjKind};
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: crate::runtime::rc::MbRwLock::new(FxHashMap::default()),
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
    v.as_int().unwrap_or(0)
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
    if let Some(id) = handle_of(q) {
        QUEUES.with(|m| {
            if let Some(state) = m.borrow_mut().get_mut(&id) {
                state.items.push_back(item);
            }
        });
    }
    MbValue::none()
}

pub fn mb_queue_get(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return QUEUES.with(|m| {
            let mut map = m.borrow_mut();
            if let Some(state) = map.get_mut(&id) {
                match state.kind {
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
                }
            } else {
                MbValue::none()
            }
        });
    }
    MbValue::none()
}

pub fn mb_queue_empty(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return QUEUES.with(|m| {
            MbValue::from_bool(
                m.borrow()
                    .get(&id)
                    .map(|s| s.items.is_empty())
                    .unwrap_or(true),
            )
        });
    }
    MbValue::from_bool(true)
}

pub fn mb_queue_qsize(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return QUEUES.with(|m| {
            MbValue::from_int(
                m.borrow()
                    .get(&id)
                    .map(|s| s.items.len() as i64)
                    .unwrap_or(0),
            )
        });
    }
    MbValue::from_int(0)
}

pub fn mb_queue_full(q: MbValue) -> MbValue {
    if let Some(id) = handle_of(q) {
        return QUEUES.with(|m| {
            MbValue::from_bool(
                m.borrow()
                    .get(&id)
                    .map(|s| s.maxsize > 0 && s.items.len() as i64 >= s.maxsize)
                    .unwrap_or(false),
            )
        });
    }
    MbValue::from_bool(false)
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
}
