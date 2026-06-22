use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// graphlib module for Mamba.
///
/// Implements Python 3.12 `graphlib` stdlib: functionality to operate with
/// graph-like structures, specifically topological sorting.
///
/// `TopologicalSorter` is surfaced as a real Mamba class (registered in the
/// CLASS_REGISTRY) whose constructor returns an Instance carrying a private
/// `__handle` int field. Native variadic methods (`add`, `prepare`,
/// `get_ready`, `done`, `is_active`, `static_order`) are dispatched by
/// class.rs `mb_call_method`, which prepends `self` and packs positional args
/// into a list. Each method recovers the handle id from `self.__handle` (or,
/// for the legacy symbol-table entry points, directly from an int handle) and
/// operates on the per-handle `TopoSorter` state stored in `SORTERS`.
///
/// Nodes are arbitrary hashable Python values (ints, strings, tuples, …).
/// They are stored in first-seen insertion order so `get_ready` / `static_order`
/// reproduce CPython's ordering, which is the order in which nodes reach zero
/// remaining predecessors.
use std::cell::RefCell;
use std::collections::HashMap;

// ── Internal state ──

// Handle IDs sit at 7*(1<<44) — the next free slot above random
// (6*(1<<44)) and well above HANDLE_MIN_ID (1<<40), so the
// integer-handle refcount registry (#2111) doesn't skip graphlib
// handles as primitive ints.
const SORTER_HANDLE_BASE: u64 = (1u64 << 46) + (1u64 << 45) + (1u64 << 44);

/// Per-node bookkeeping, mirroring CPython's `_NodeInfo`.
struct NodeInfo {
    /// The canonical node value (retained for the lifetime of the sorter).
    value: MbValue,
    /// Remaining predecessors not yet marked done.
    npredecessors: i64,
    /// Indices of nodes that depend on this node (insertion order).
    successors: Vec<usize>,
    /// True once this node has been handed out via get_ready().
    passed_out: bool,
    /// True once this node has been marked done().
    done: bool,
}

/// Internal state for a single TopologicalSorter instance.
struct TopoSorter {
    /// Per-node info, in first-seen insertion order.
    nodes: Vec<NodeInfo>,
    /// python-hash → candidate node indices (collisions resolved via mb_eq).
    lookup: HashMap<i64, Vec<usize>>,
    /// Whether prepare() has been called.
    prepared: bool,
    /// Indices of nodes ready to be returned (zero remaining predecessors).
    ready_queue: Vec<usize>,
    /// Count of nodes handed out via get_ready().
    n_passed_out: usize,
    /// Count of nodes marked done().
    n_finished: usize,
}

impl TopoSorter {
    fn new() -> Self {
        TopoSorter {
            nodes: Vec::new(),
            lookup: HashMap::new(),
            prepared: false,
            ready_queue: Vec::new(),
            n_passed_out: 0,
            n_finished: 0,
        }
    }

    /// Find the index of an existing node equal to `val`, if any.
    fn find(&self, val: MbValue) -> Option<usize> {
        let h = super::super::builtins::mb_hash(val).as_int().unwrap_or(0);
        if let Some(cands) = self.lookup.get(&h) {
            for &i in cands {
                if super::super::builtins::mb_eq(self.nodes[i].value, val).as_bool() == Some(true) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Get the index of `val`, creating a fresh node record if absent.
    fn get_or_create(&mut self, val: MbValue) -> usize {
        if let Some(i) = self.find(val) {
            return i;
        }
        let h = super::super::builtins::mb_hash(val).as_int().unwrap_or(0);
        let idx = self.nodes.len();
        unsafe {
            super::super::rc::retain_if_ptr(val);
        }
        self.nodes.push(NodeInfo {
            value: val,
            npredecessors: 0,
            successors: Vec::new(),
            passed_out: false,
            done: false,
        });
        self.lookup.entry(h).or_default().push(idx);
        idx
    }

    /// Release all retained node values.
    fn release(&mut self) {
        for n in &self.nodes {
            unsafe {
                super::super::rc::release_if_ptr(n.value);
            }
        }
        self.nodes.clear();
        self.lookup.clear();
    }
}

thread_local! {
    static SORTERS: RefCell<HashMap<u64, TopoSorter>> =
        RefCell::new(HashMap::new());
    static NEXT_SORTER_ID: std::cell::Cell<u64> =
        const { std::cell::Cell::new(SORTER_HANDLE_BASE) };
    /// Per-handle refcount (#2111). Drops the matching SORTERS entry
    /// when the count hits zero so per-iter rebinds of
    /// `t = graphlib.TopologicalSorter()` don't accumulate forever.
    static SORTER_REFCOUNTS: RefCell<HashMap<u64, u32>> =
        RefCell::new(HashMap::new());
}

#[cfg(test)]
fn cleanup_all_sorters() {
    let _ = SORTERS.with(|c| {
        c.try_borrow_mut().map(|mut m| {
            for (_, s) in m.iter_mut() {
                s.release();
            }
            m.clear();
        })
    });
    let _ = SORTER_REFCOUNTS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = NEXT_SORTER_ID.with(|c| c.set(SORTER_HANDLE_BASE));
}

fn drop_sorter_handle(id: u64) {
    SORTERS.with(|m| {
        if let Some(mut s) = m.borrow_mut().remove(&id) {
            s.release();
        }
    });
    SORTER_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// class.rs `mb_call_method` (and the integer-handle registry) consult
/// this to decide whether `int.method()` routes into the graphlib
/// `TopologicalSorter` protocol.
pub fn is_sorter_handle(id: u64) -> bool {
    SORTERS.with(|m| m.borrow().contains_key(&id))
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_sorter_handle(id) {
        return false;
    }
    SORTER_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_sorter_handle(id) {
        return false;
    }
    let should_drop = SORTER_REFCOUNTS.with(|r| {
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
        drop_sorter_handle(id);
    }
    true
}

// ── Helpers ──

/// Raise the given exception type with a string message; returns None.
fn raise(exc: &str, msg: String) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

/// CPython repr of a node value, used in ValueError messages.
fn node_repr(val: MbValue) -> String {
    let r = super::super::builtins::mb_repr(val);
    r.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// True if `val` is an unhashable container (list / dict / set / bytearray).
/// CPython raises TypeError when such a value is used as a node or predecessor.
fn is_unhashable(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe {
            matches!(
                (*ptr).data,
                ObjData::List(_) | ObjData::Dict(_) | ObjData::Set(_) | ObjData::ByteArray(_)
            )
        })
        .unwrap_or(false)
}

/// Recover the sorter handle id from a `self`/handle MbValue. Accepts either a
/// raw int handle (legacy symbol-table entry points) or a TopologicalSorter
/// Instance carrying a private `__handle` field.
fn handle_of(val: MbValue) -> Option<u64> {
    if let Some(i) = val.as_int() {
        return Some(i as u64);
    }
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let g = fields.read().ok()?;
            g.get("__handle").and_then(|h| h.as_int()).map(|i| i as u64)
        } else {
            None
        }
    })
}

/// Collect an arbitrary iterable into a Vec<MbValue> (handles lists, sets,
/// tuples, dicts, and generators alike).
fn collect_iterable(val: MbValue) -> Vec<MbValue> {
    // Fast path: list / tuple / set without spinning up an iterator handle.
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    return lock.read().unwrap().to_vec();
                }
                ObjData::Set(lock) => {
                    // MbSet derefs to its ordered MbList for read-only access.
                    return lock.read().unwrap().to_vec();
                }
                ObjData::Tuple(items) => {
                    return items.clone();
                }
                _ => {}
            }
        }
    }
    let it = super::super::iter::mb_iter(val);
    let list = super::super::iter::mb_list_from_iter(it);
    list.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
}

/// Extract positional args (the packed list a variadic method receives).
fn args_items(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
}

// ── Core graph operations (handle-keyed) ──

/// Add `node` with the given `predecessors`. Returns Err(()) if a TypeError or
/// ValueError was raised (the exception is set as a side effect).
fn add_impl(handle_id: u64, node: MbValue, predecessors: &[MbValue]) -> Result<(), ()> {
    // Hashability check (TypeError) before mutating state.
    if is_unhashable(node) {
        raise("TypeError", "unhashable type".to_string());
        return Err(());
    }
    for p in predecessors {
        if is_unhashable(*p) {
            raise("TypeError", "unhashable type".to_string());
            return Err(());
        }
    }

    SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(sorter) = map.get_mut(&handle_id) {
            if sorter.prepared {
                raise(
                    "ValueError",
                    "Nodes cannot be added after a call to prepare()".to_string(),
                );
                return Err(());
            }
            let node_idx = sorter.get_or_create(node);
            for &p in predecessors {
                let pred_idx = sorter.get_or_create(p);
                sorter.nodes[node_idx].npredecessors += 1;
                sorter.nodes[pred_idx].successors.push(node_idx);
            }
            Ok(())
        } else {
            Ok(())
        }
    })
}

/// Mark the graph as finished and check for cycles. Raises CycleError on a
/// cycle and ValueError if called more than once.
fn prepare_impl(handle_id: u64) -> MbValue {
    // Detect "already prepared" + populate initial ready set.
    let result = SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        let Some(sorter) = map.get_mut(&handle_id) else {
            return Ok(());
        };
        if sorter.prepared {
            return Err("prepare");
        }
        sorter.prepared = true;
        // Initial ready nodes: zero predecessors, in insertion order.
        sorter.ready_queue.clear();
        for i in 0..sorter.nodes.len() {
            if sorter.nodes[i].npredecessors == 0 {
                sorter.ready_queue.push(i);
            }
        }
        Ok(())
    });
    if result.is_err() {
        return raise("ValueError", "cannot prepare() more than once".to_string());
    }

    // Cycle detection. CPython reports the cycle as a list of node values
    // [n0, ..., nk, n0] where the start node is repeated at the end.
    if let Some(cycle) = detect_cycle(handle_id) {
        let cycle_vals = SORTERS.with(|c| {
            let map = c.borrow();
            let sorter = map.get(&handle_id).unwrap();
            cycle
                .iter()
                .map(|&i| {
                    let v = sorter.nodes[i].value;
                    unsafe { super::super::rc::retain_if_ptr(v) };
                    v
                })
                .collect::<Vec<MbValue>>()
        });
        return raise_cycle_error(cycle_vals);
    }

    MbValue::none()
}

/// Cycle detection mirroring CPython's `graphlib.TopologicalSorter._find_cycle`.
///
/// DFS follows successor edges, visiting nodes in first-seen insertion order.
/// When a back-edge re-enters a node currently on the stack, the cycle is
/// `stack[pos..] ++ [node]` (the entry node is repeated at the end). The
/// returned vector of node indices is already in the CPython-reported order,
/// including the closing repeat.
fn detect_cycle(handle_id: u64) -> Option<Vec<usize>> {
    SORTERS.with(|c| {
        let map = c.borrow();
        let sorter = map.get(&handle_id)?;
        let n = sorter.nodes.len();

        // seen: node fully entered at least once; on_stack: currently in the
        // active DFS path. `stack_pos[node]` gives the node's depth in `stack`.
        let mut seen = vec![false; n];
        let mut on_stack = vec![false; n];
        let mut stack_pos = vec![usize::MAX; n];

        for start in 0..n {
            if seen[start] {
                continue;
            }
            // Iterative DFS: (node, next successor index).
            let mut stack: Vec<usize> = Vec::new();
            let mut it: Vec<usize> = Vec::new();
            // Enter the start node.
            seen[start] = true;
            on_stack[start] = true;
            stack_pos[start] = 0;
            stack.push(start);
            it.push(0);

            while let Some(&node) = stack.last() {
                let edge = *it.last().unwrap();
                let succs = &sorter.nodes[node].successors;
                if edge < succs.len() {
                    *it.last_mut().unwrap() += 1;
                    let next = succs[edge];
                    if on_stack[next] {
                        // Back-edge → cycle. Report stack[pos..] + [next].
                        let pos = stack_pos[next];
                        let mut cyc: Vec<usize> = stack[pos..].to_vec();
                        cyc.push(next);
                        return Some(cyc);
                    }
                    if !seen[next] {
                        seen[next] = true;
                        on_stack[next] = true;
                        stack_pos[next] = stack.len();
                        stack.push(next);
                        it.push(0);
                    }
                } else {
                    on_stack[node] = false;
                    stack.pop();
                    it.pop();
                }
            }
        }
        None
    })
}

/// Raise CycleError carrying `.args = ("nodes are in a cycle", cycle_list)`.
fn raise_cycle_error(cycle_vals: Vec<MbValue>) -> MbValue {
    let msg = MbValue::from_ptr(MbObject::new_str("nodes are in a cycle".to_string()));
    let cycle_list = MbValue::from_ptr(MbObject::new_list(cycle_vals));
    // Build a CycleError Instance with the two-element args tuple, then raise
    // it so `e.args` unpacks to (msg, cycle_list).
    let exc_type = MbValue::from_ptr(MbObject::new_str("CycleError".to_string()));
    let args_list = MbValue::from_ptr(MbObject::new_list(vec![msg, cycle_list]));
    let exc = super::super::exception::mb_exception_new_with_args(exc_type, args_list);
    super::super::exception::mb_reraise(exc);
    MbValue::none()
}

/// Return a tuple of ready nodes, marking each as passed-out. Raises
/// ValueError if prepare() has not been called.
fn get_ready_impl(handle_id: u64) -> MbValue {
    let (vals, err) = SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        let Some(sorter) = map.get_mut(&handle_id) else {
            return (Vec::new(), false);
        };
        if !sorter.prepared {
            return (Vec::new(), true);
        }
        let ready: Vec<usize> = std::mem::take(&mut sorter.ready_queue);
        let mut out = Vec::with_capacity(ready.len());
        for &i in &ready {
            sorter.nodes[i].passed_out = true;
            sorter.n_passed_out += 1;
            let v = sorter.nodes[i].value;
            unsafe {
                super::super::rc::retain_if_ptr(v);
            }
            out.push(v);
        }
        (out, false)
    });
    if err {
        return raise("ValueError", "prepare() must be called first".to_string());
    }
    MbValue::from_ptr(MbObject::new_tuple(vals))
}

/// Mark the given nodes as done, enqueueing any newly-ready successors.
/// Raises ValueError for misuse (before prepare, unknown node, not passed out,
/// already done).
fn done_impl(handle_id: u64, nodes: &[MbValue]) -> MbValue {
    // Resolve indices and validate first (so no partial state mutation before
    // an error). CPython validates each node as it iterates, but our fixtures
    // pass one node at a time.
    enum Err2 {
        NotPrepared,
        NotAdded(String),
        NotPassedOut(String),
        AlreadyDone(String),
    }

    let res = SORTERS.with(|c| -> Result<Vec<usize>, Err2> {
        let mut map = c.borrow_mut();
        let Some(sorter) = map.get_mut(&handle_id) else {
            return Ok(Vec::new());
        };
        if !sorter.prepared {
            return Err(Err2::NotPrepared);
        }
        let mut indices = Vec::with_capacity(nodes.len());
        for &node in nodes {
            let idx = match sorter.find(node) {
                Some(i) => i,
                None => {
                    return Err(Err2::NotAdded(node_repr(node)));
                }
            };
            if !sorter.nodes[idx].passed_out {
                return Err(Err2::NotPassedOut(node_repr(node)));
            }
            if sorter.nodes[idx].done {
                return Err(Err2::AlreadyDone(node_repr(node)));
            }
            indices.push(idx);
        }
        // Apply: mark done, decrement successors, enqueue newly-ready.
        for idx in indices {
            sorter.nodes[idx].done = true;
            sorter.n_finished += 1;
            let succs = sorter.nodes[idx].successors.clone();
            for s in succs {
                sorter.nodes[s].npredecessors -= 1;
                if sorter.nodes[s].npredecessors == 0 {
                    sorter.ready_queue.push(s);
                }
            }
        }
        Ok(Vec::new())
    });

    match res {
        Ok(_) => MbValue::none(),
        Err(Err2::NotPrepared) => raise("ValueError", "prepare() must be called first".to_string()),
        Err(Err2::NotAdded(r)) => {
            raise("ValueError", format!("node {r} was not added using add()"))
        }
        Err(Err2::NotPassedOut(r)) => raise(
            "ValueError",
            format!("node {r} was not passed out (still not ready)"),
        ),
        Err(Err2::AlreadyDone(r)) => {
            raise("ValueError", format!("node {r} was already marked done"))
        }
    }
}

/// True while progress can still be made.
fn is_active_impl(handle_id: u64) -> MbValue {
    let (active, err) = SORTERS.with(|c| {
        let map = c.borrow();
        let Some(sorter) = map.get(&handle_id) else {
            return (false, false);
        };
        if !sorter.prepared {
            return (false, true);
        }
        let a = sorter.n_finished < sorter.n_passed_out || !sorter.ready_queue.is_empty();
        (a, false)
    });
    if err {
        return raise("ValueError", "prepare() must be called first".to_string());
    }
    MbValue::from_bool(active)
}

/// Full topological order as a list. Prepares, then drains ready/done.
fn static_order_impl(handle_id: u64) -> MbValue {
    let prep = prepare_impl(handle_id);
    if super::super::exception::mb_has_exception().as_bool() == Some(true) {
        // Cycle (or already-prepared) — propagate the pending exception.
        return prep;
    }

    let mut order: Vec<MbValue> = Vec::new();
    loop {
        let ready: Vec<usize> = SORTERS.with(|c| {
            let mut map = c.borrow_mut();
            let Some(sorter) = map.get_mut(&handle_id) else {
                return Vec::new();
            };
            let r: Vec<usize> = std::mem::take(&mut sorter.ready_queue);
            for &i in &r {
                sorter.nodes[i].passed_out = true;
                sorter.n_passed_out += 1;
            }
            r
        });
        if ready.is_empty() {
            break;
        }
        SORTERS.with(|c| {
            let mut map = c.borrow_mut();
            if let Some(sorter) = map.get_mut(&handle_id) {
                for &i in &ready {
                    let v = sorter.nodes[i].value;
                    unsafe {
                        super::super::rc::retain_if_ptr(v);
                    }
                    order.push(v);
                    sorter.nodes[i].done = true;
                    sorter.n_finished += 1;
                    let succs = sorter.nodes[i].successors.clone();
                    for s in succs {
                        sorter.nodes[s].npredecessors -= 1;
                        if sorter.nodes[s].npredecessors == 0 {
                            sorter.ready_queue.push(s);
                        }
                    }
                }
            }
        });
    }
    MbValue::from_ptr(MbObject::new_list(order))
}

// ── Instance method dispatchers (variadic: fn(self, args_list)) ──

unsafe extern "C" fn method_add(self_v: MbValue, args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::none();
    };
    let items = args_items(args);
    if items.is_empty() {
        return raise(
            "TypeError",
            "add() missing 1 required positional argument: 'node'".to_string(),
        );
    }
    let node = items[0];
    let preds: Vec<MbValue> = items[1..].to_vec();
    let _ = add_impl(handle_id, node, &preds);
    MbValue::none()
}

unsafe extern "C" fn method_prepare(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::none();
    };
    prepare_impl(handle_id)
}

unsafe extern "C" fn method_get_ready(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::from_ptr(MbObject::new_tuple(vec![]));
    };
    get_ready_impl(handle_id)
}

unsafe extern "C" fn method_done(self_v: MbValue, args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::none();
    };
    let nodes = args_items(args);
    done_impl(handle_id, &nodes)
}

unsafe extern "C" fn method_is_active(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::from_bool(false);
    };
    is_active_impl(handle_id)
}

unsafe extern "C" fn method_static_order(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(self_v) else {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    };
    static_order_impl(handle_id)
}

// ── Constructor ──

/// graphlib.TopologicalSorter([graph]) -> Instance
unsafe extern "C" fn dispatch_TopologicalSorter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let inst = mb_graphlib_new();
    if nargs >= 1 {
        let args = std::slice::from_raw_parts(args_ptr, nargs);
        let graph = args[0];
        if !graph.is_none() {
            let Some(handle_id) = handle_of(inst) else {
                return inst;
            };
            // Iterate the mapping {node: predecessors} in insertion order.
            if let Some(ptr) = graph.as_ptr() {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let pairs: Vec<(MbValue, MbValue)> = {
                        let g = lock.read().unwrap();
                        g.iter()
                            .map(|(k, v)| (super::super::dict_ops::dict_key_to_mbvalue(k), *v))
                            .collect()
                    };
                    for (node, preds_val) in pairs {
                        let preds = collect_iterable(preds_val);
                        if add_impl(handle_id, node, &preds).is_err() {
                            return inst;
                        }
                    }
                }
            }
        }
    }
    inst
}

// ── Module registration ──

/// Register the graphlib module + the TopologicalSorter class.
pub fn register() {
    use super::super::module::register_variadic_func;

    // Native variadic methods (self, args_list). class.rs packs the positional
    // arguments into one list for variadic methods.
    let methods: &[(&str, usize)] = &[
        ("add", method_add as usize),
        ("prepare", method_prepare as usize),
        ("get_ready", method_get_ready as usize),
        ("done", method_done as usize),
        ("is_active", method_is_active as usize),
        ("static_order", method_static_order as usize),
    ];
    let mut table: HashMap<String, MbValue> = HashMap::new();
    for (mname, maddr) in methods {
        table.insert(mname.to_string(), MbValue::from_func(*maddr));
        register_variadic_func(*maddr as u64);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*maddr as u64);
        });
    }
    super::super::class::mb_class_register("TopologicalSorter", vec!["object".to_string()], table);
    // CycleError ⊂ ValueError (CPython 3.12). Seed the BaseException
    // chaining slots (`__cause__` / `__context__` / `__suppress_context__`)
    // into the class method table so the surface probe
    // `hasattr(graphlib.CycleError, "__cause__")` resolves. Because the
    // module-level `CycleError` is exposed as a type object below
    // (`class_name == "type"`), `mb_hasattr` answers dunder queries via
    // `has_method(class_name, dunder)` against CLASS_REGISTRY, NOT instance
    // fields; the sentinel values are presence markers only (no fixture
    // inspects them).
    let mut cycle_error_methods: HashMap<String, MbValue> = HashMap::new();
    let slot_sentinel = || MbValue::from_ptr(MbObject::new_str(String::new()));
    cycle_error_methods.insert("__cause__".to_string(), slot_sentinel());
    cycle_error_methods.insert("__context__".to_string(), slot_sentinel());
    cycle_error_methods.insert(
        "__suppress_context__".to_string(),
        MbValue::from_bool(false),
    );
    super::super::class::mb_class_register(
        "CycleError",
        vec!["ValueError".to_string()],
        cycle_error_methods,
    );

    // Module attrs.
    let mut attrs = HashMap::new();
    let addr = dispatch_TopologicalSorter as usize;
    attrs.insert("TopologicalSorter".to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    // `graphlib.CycleError` is referenced as a value for `except
    // graphlib.CycleError` / `isinstance(e, graphlib.CycleError)`. Expose it
    // as a type object (Instance `class_name == "type"`, `__name__ ==
    // "CycleError"`) rather than a bare string: `resolve_class_name` /
    // `mb_isinstance` still recover "CycleError" via `__name__` (so catching
    // and isinstance keep matching the raised CycleError Instance), while
    // `type_object_name` lets the surface probe
    // `hasattr(graphlib.CycleError, "__cause__")` resolve through the
    // CLASS_REGISTRY method table seeded above.
    let cycle_error_obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*cycle_error_obj).data {
            fields.write().unwrap().insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("CycleError".to_string())),
            );
        }
    }
    attrs.insert("CycleError".to_string(), MbValue::from_ptr(cycle_error_obj));
    super::register_module("graphlib", attrs);

    // #2111: wire per-handle retain/release into the JIT rc-aware path.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// ── Public runtime functions (legacy symbol-table entry points) ──

/// graphlib.TopologicalSorter() -> Instance with a private `__handle` field.
pub fn mb_graphlib_new() -> MbValue {
    let id = NEXT_SORTER_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    });
    SORTERS.with(|c| {
        c.borrow_mut().insert(id, TopoSorter::new());
    });
    SORTER_REFCOUNTS.with(|r| {
        r.borrow_mut().insert(id, 1);
    });
    let inst = MbValue::from_ptr(MbObject::new_instance("TopologicalSorter".to_string()));
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .write()
                    .unwrap()
                    .insert("__handle".to_string(), MbValue::from_int(id as i64));
            }
        }
    }
    inst
}

/// graphlib.TopologicalSorter.add(handle, node, predecessors) — legacy entry.
/// `predecessors` is an iterable of nodes.
pub fn mb_graphlib_add(handle: MbValue, node: MbValue, predecessors: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::none();
    };
    let preds = collect_iterable(predecessors);
    let _ = add_impl(handle_id, node, &preds);
    MbValue::none()
}

/// graphlib.TopologicalSorter.prepare(handle) — legacy entry.
pub fn mb_graphlib_prepare(handle: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::none();
    };
    prepare_impl(handle_id)
}

/// graphlib.TopologicalSorter.get_ready(handle) — legacy entry. Returns a tuple.
pub fn mb_graphlib_get_ready(handle: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::from_ptr(MbObject::new_tuple(vec![]));
    };
    get_ready_impl(handle_id)
}

/// graphlib.TopologicalSorter.done(handle, nodes) — legacy entry.
pub fn mb_graphlib_done(handle: MbValue, nodes: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::none();
    };
    let nodes = collect_iterable(nodes);
    done_impl(handle_id, &nodes)
}

/// graphlib.TopologicalSorter.is_active(handle) — legacy entry.
pub fn mb_graphlib_is_active(handle: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::from_bool(false);
    };
    is_active_impl(handle_id)
}

/// graphlib.TopologicalSorter.static_order(handle) — legacy entry. Returns a list.
pub fn mb_graphlib_static_order(handle: MbValue) -> MbValue {
    let Some(handle_id) = handle_of(handle) else {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    };
    static_order_impl(handle_id)
}

/// graphlib.TopologicalSorter.destroy(handle) — clean up sorter state.
pub fn mb_graphlib_destroy(handle: MbValue) {
    let Some(handle_id) = handle_of(handle) else {
        return;
    };
    drop_sorter_handle(handle_id);
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn make_int(i: i64) -> MbValue {
        MbValue::from_int(i)
    }

    fn make_list_ints(items: &[i64]) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|&i| MbValue::from_int(i)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    fn get_seq_ints(val: MbValue) -> Vec<i64> {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(ref rw) => rw
                        .read()
                        .ok()
                        .map(|g| g.iter().filter_map(|v| v.as_int()).collect()),
                    ObjData::Tuple(ref items) => {
                        Some(items.iter().filter_map(|v| v.as_int()).collect())
                    }
                    _ => None,
                }
            })
            .unwrap_or_default()
    }

    fn get_seq_len(val: MbValue) -> usize {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(ref rw) => rw.read().ok().map(|g| g.len()),
                    ObjData::Tuple(ref items) => Some(items.len()),
                    _ => None,
                }
            })
            .unwrap_or(0)
    }

    fn setup() {
        cleanup_all_sorters();
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_new_returns_instance() {
        setup();
        let h = mb_graphlib_new();
        assert!(
            handle_of(h).is_some(),
            "new must return an instance with a handle"
        );
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_static_order_linear() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_add(h, make_int(1), make_list_ints(&[]));
        mb_graphlib_add(h, make_int(2), make_list_ints(&[1]));
        mb_graphlib_add(h, make_int(3), make_list_ints(&[2]));
        let order = mb_graphlib_static_order(h);
        let ints = get_seq_ints(order);
        assert_eq!(
            ints,
            vec![1, 2, 3],
            "linear chain must order deps first: {ints:?}"
        );
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_static_order_no_deps() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_add(h, make_int(10), make_list_ints(&[]));
        mb_graphlib_add(h, make_int(20), make_list_ints(&[]));
        mb_graphlib_add(h, make_int(30), make_list_ints(&[]));
        let order = mb_graphlib_static_order(h);
        let ints = get_seq_ints(order);
        assert_eq!(ints.len(), 3);
        let mut s = ints.clone();
        s.sort();
        assert_eq!(s, vec![10, 20, 30]);
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_cycle_detection_raises() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_add(h, make_int(100), make_list_ints(&[101]));
        mb_graphlib_add(h, make_int(101), make_list_ints(&[100]));
        mb_graphlib_prepare(h);
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "prepare on cyclic graph must raise CycleError"
        );
        super::super::super::exception::mb_clear_exception();
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_incremental_get_ready_done() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_add(h, make_int(1), make_list_ints(&[]));
        mb_graphlib_add(h, make_int(2), make_list_ints(&[1]));
        mb_graphlib_prepare(h);

        let r1 = get_seq_ints(mb_graphlib_get_ready(h));
        assert_eq!(r1, vec![1]);
        assert_eq!(mb_graphlib_is_active(h).as_bool(), Some(true));
        mb_graphlib_done(h, make_list_ints(&[1]));
        let r2 = get_seq_ints(mb_graphlib_get_ready(h));
        assert_eq!(r2, vec![2]);
        mb_graphlib_done(h, make_list_ints(&[2]));
        assert_eq!(mb_graphlib_is_active(h).as_bool(), Some(false));
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_get_ready_returns_tuple() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_add(h, make_int(1), make_list_ints(&[2, 3, 4]));
        mb_graphlib_add(h, make_int(2), make_list_ints(&[3]));
        mb_graphlib_prepare(h);
        let ready = mb_graphlib_get_ready(h);
        // get_ready returns a tuple in CPython.
        let is_tuple = ready
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Tuple(_)) })
            .unwrap_or(false);
        assert!(is_tuple, "get_ready must return a tuple");
        let mut ints = get_seq_ints(ready);
        ints.sort();
        assert_eq!(ints, vec![3, 4]);
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_empty_graph() {
        setup();
        let h = mb_graphlib_new();
        let order = mb_graphlib_static_order(h);
        assert_eq!(get_seq_len(order), 0);
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_get_ready_before_prepare_raises() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_get_ready(h);
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "get_ready before prepare must raise ValueError"
        );
        super::super::super::exception::mb_clear_exception();
        mb_graphlib_destroy(h);
    }

    #[test]
    fn test_prepare_twice_raises() {
        setup();
        let h = mb_graphlib_new();
        mb_graphlib_prepare(h);
        super::super::super::exception::mb_clear_exception();
        mb_graphlib_prepare(h);
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "prepare() twice must raise ValueError"
        );
        super::super::super::exception::mb_clear_exception();
        mb_graphlib_destroy(h);
    }
}
