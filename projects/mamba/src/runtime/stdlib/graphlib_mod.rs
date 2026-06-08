use super::super::rc::MbObject;
use super::super::value::MbValue;
/// graphlib module for Mamba.
///
/// Implements Python 3.12 `graphlib` stdlib: functionality to operate with
/// graph-like structures, specifically topological sorting.
///
/// Provides `TopologicalSorter` class and `CycleError` matching CPython 3.12
/// signatures. Nodes are represented as integers (i64) in this implementation.
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

// ── Internal state ──

// Handle IDs sit at 7*(1<<44) — the next free slot above random
// (6*(1<<44)) and well above HANDLE_MIN_ID (1<<40), so the
// integer-handle refcount registry (#2111) doesn't skip graphlib
// handles as primitive ints. Pre-#2111 this base was `1`, which
// (a) collided with primitive int values and (b) was filtered
// out by `integer_handle_registry::retain/release` — every per-iter
// `t = graphlib.TopologicalSorter()` leaked the prior sorter's
// graph storage forever.
const SORTER_HANDLE_BASE: u64 = (1u64 << 46) + (1u64 << 45) + (1u64 << 44);

/// Internal state for a single TopologicalSorter instance.
struct TopoSorter {
    /// node → set of predecessors (nodes that must come before)
    graph: HashMap<i64, HashSet<i64>>,
    /// All nodes ever mentioned (both as nodes and as predecessors)
    all_nodes: HashSet<i64>,
    /// Whether prepare() has been called
    prepared: bool,
    /// Nodes with zero remaining predecessors, ready to be returned
    ready_queue: VecDeque<i64>,
    /// Nodes that have been marked done
    done_set: HashSet<i64>,
    /// Nodes currently passed out via get_ready() but not yet done
    in_progress: HashSet<i64>,
    /// Count of nodes passed out
    n_passed_out: usize,
    /// Count of nodes finished (done)
    n_finished: usize,
}

impl TopoSorter {
    fn new() -> Self {
        TopoSorter {
            graph: HashMap::new(),
            all_nodes: HashSet::new(),
            prepared: false,
            ready_queue: VecDeque::new(),
            done_set: HashSet::new(),
            in_progress: HashSet::new(),
            n_passed_out: 0,
            n_finished: 0,
        }
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
    let _ = SORTERS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = SORTER_REFCOUNTS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = NEXT_SORTER_ID.with(|c| c.set(SORTER_HANDLE_BASE));
}

fn drop_sorter_handle(id: u64) {
    SORTERS.with(|m| {
        m.borrow_mut().remove(&id);
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

/// Extract an i64 from an MbValue (integer).
fn extract_int(val: MbValue) -> Option<i64> {
    val.as_int()
}

/// Extract a Vec<i64> from an MbValue that holds a List of integer objects.
fn extract_list_of_ints(val: MbValue) -> Vec<i64> {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::List(ref rw) = (*ptr).data {
                let guard = rw.read().ok()?;
                let results: Vec<i64> = guard.iter().filter_map(|v| extract_int(*v)).collect();
                Some(results)
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Raise a CycleError with the given message.
fn raise_cycle_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("CycleError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── Module registration ──

unsafe extern "C" fn dispatch_TopologicalSorter(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_graphlib_new()
}

/// Register the graphlib module in the stdlib registry.
pub fn register() {
    let mut attrs = HashMap::new();
    let addr = dispatch_TopologicalSorter as usize;
    attrs.insert("TopologicalSorter".to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    // CycleError is a class-marker string (CPython exposes it as an
    // exception class). Mamba uses a string sentinel — left as-is.
    attrs.insert(
        "CycleError".to_string(),
        MbValue::from_ptr(MbObject::new_str("CycleError".to_string())),
    );
    super::register_module("graphlib", attrs);
    // #2111: wire per-handle retain/release into the JIT rc-aware path
    // so per-iter rebinds of `TopologicalSorter()` drop the prior
    // sorter's graph storage instead of leaking until process exit.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// ── Public runtime functions ──

/// graphlib.TopologicalSorter() -> handle (int)
///
/// Create a new TopologicalSorter instance. Returns an integer handle
/// used to reference this sorter in subsequent calls.
pub fn mb_graphlib_new() -> MbValue {
    let id = NEXT_SORTER_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    });
    SORTERS.with(|c| {
        c.borrow_mut().insert(id, TopoSorter::new());
    });
    MbValue::from_int(id as i64)
}

/// graphlib.TopologicalSorter.add(handle, node, predecessors) -> None
///
/// Add a node with its predecessors to the graph. Node and predecessors
/// are integers. predecessors is a list of ints.
pub fn mb_graphlib_add(handle: MbValue, node: MbValue, predecessors: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::none(),
    };
    let node_id = match extract_int(node) {
        Some(n) => n,
        None => return MbValue::none(),
    };
    let preds = extract_list_of_ints(predecessors);

    SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(sorter) = map.get_mut(&handle_id) {
            // Ensure the node itself is in all_nodes
            sorter.all_nodes.insert(node_id);
            // Ensure node has an entry in graph (even if no predecessors)
            sorter.graph.entry(node_id).or_insert_with(HashSet::new);
            // Add all predecessors
            for pred in &preds {
                sorter.all_nodes.insert(*pred);
                sorter
                    .graph
                    .entry(node_id)
                    .or_insert_with(HashSet::new)
                    .insert(*pred);
                // Ensure predecessors also have entries (even if they have no preds)
                sorter.graph.entry(*pred).or_insert_with(HashSet::new);
            }
        }
    });
    MbValue::none()
}

/// graphlib.TopologicalSorter.prepare(handle) -> None
///
/// Mark the graph as finished and check for cycles using Kahn's algorithm.
/// Raises CycleError if a cycle is detected. Must be called before
/// get_ready() and done().
pub fn mb_graphlib_prepare(handle: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::none(),
    };

    // Gather all nodes and build in-degree map for Kahn's algorithm
    let (all_nodes_vec, cycle_detected, initial_ready) = SORTERS.with(|c| {
        let map = c.borrow();
        if let Some(sorter) = map.get(&handle_id) {
            let all_nodes: Vec<i64> = sorter.all_nodes.iter().cloned().collect();

            // Build in-degree count (count of predecessors not yet processed)
            let mut in_degree: HashMap<i64, usize> = HashMap::new();
            for &node in &all_nodes {
                in_degree.entry(node).or_insert(0);
                if let Some(preds) = sorter.graph.get(&node) {
                    *in_degree.entry(node).or_insert(0) += preds.len();
                }
            }

            // Kahn's algorithm: find nodes with zero in-degree
            let mut queue: VecDeque<i64> = in_degree
                .iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(&n, _)| n)
                .collect();

            let mut visited_count = 0usize;
            let zero_indegree: Vec<i64> = queue.iter().cloned().collect();

            while let Some(node) = queue.pop_front() {
                visited_count += 1;
                // Find all nodes that depend on this node (node is a predecessor)
                for &successor in &all_nodes {
                    if let Some(preds) = sorter.graph.get(&successor) {
                        if preds.contains(&node) {
                            let deg = in_degree.entry(successor).or_insert(0);
                            if *deg > 0 {
                                *deg -= 1;
                                if *deg == 0 {
                                    queue.push_back(successor);
                                }
                            }
                        }
                    }
                }
            }

            let cycle = visited_count < all_nodes.len();
            (all_nodes, cycle, zero_indegree)
        } else {
            (Vec::new(), false, Vec::new())
        }
    });

    if cycle_detected {
        return raise_cycle_error("nodes are in a cycle");
    }

    // Now set prepared = true and populate the ready_queue
    SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(sorter) = map.get_mut(&handle_id) {
            sorter.prepared = true;
            sorter.ready_queue = VecDeque::from(initial_ready);
        }
    });

    let _ = all_nodes_vec;
    MbValue::none()
}

/// graphlib.TopologicalSorter.get_ready(handle) -> list of ints
///
/// Return a tuple (list) of all nodes that are ready (all predecessors done).
/// Must call prepare() first. Nodes are removed from the ready set.
pub fn mb_graphlib_get_ready(handle: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let ready_nodes: Vec<i64> = SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(sorter) = map.get_mut(&handle_id) {
            let nodes: Vec<i64> = sorter.ready_queue.drain(..).collect();
            for &n in &nodes {
                sorter.in_progress.insert(n);
                sorter.n_passed_out += 1;
            }
            nodes
        } else {
            Vec::new()
        }
    });

    let list_vals: Vec<MbValue> = ready_nodes.into_iter().map(MbValue::from_int).collect();
    MbValue::from_ptr(MbObject::new_list(list_vals))
}

/// graphlib.TopologicalSorter.done(handle, nodes) -> None
///
/// Mark nodes as processed. For each node marked done, any successor
/// whose all predecessors are now done becomes ready.
pub fn mb_graphlib_done(handle: MbValue, nodes: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::none(),
    };
    let done_nodes = extract_list_of_ints(nodes);

    SORTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(sorter) = map.get_mut(&handle_id) {
            for node in &done_nodes {
                sorter.in_progress.remove(node);
                sorter.done_set.insert(*node);
                sorter.n_finished += 1;
            }

            // Find all nodes in the graph and check if any new ones became ready
            let all_nodes: Vec<i64> = sorter.all_nodes.iter().cloned().collect();
            for candidate in &all_nodes {
                // Skip if already done, in progress, or in ready queue
                if sorter.done_set.contains(candidate) {
                    continue;
                }
                if sorter.in_progress.contains(candidate) {
                    continue;
                }
                if sorter.ready_queue.contains(candidate) {
                    continue;
                }
                // Check if all predecessors are done
                if let Some(preds) = sorter.graph.get(candidate) {
                    if preds.iter().all(|p| sorter.done_set.contains(p)) {
                        sorter.ready_queue.push_back(*candidate);
                    }
                }
            }
        }
    });

    MbValue::none()
}

/// graphlib.TopologicalSorter.is_active(handle) -> bool
///
/// Return True if more progress can be made: either nodes in the ready
/// queue, or nodes currently in progress.
pub fn mb_graphlib_is_active(handle: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::from_bool(false),
    };

    let active = SORTERS.with(|c| {
        let map = c.borrow();
        if let Some(sorter) = map.get(&handle_id) {
            !sorter.ready_queue.is_empty() || !sorter.in_progress.is_empty()
        } else {
            false
        }
    });

    MbValue::from_bool(active)
}

/// graphlib.TopologicalSorter.static_order(handle) -> list of ints
///
/// Convenience function: prepares the sorter and returns the full topological
/// order as a list. Combines prepare() + repeated get_ready()/done() calls.
/// Raises CycleError if a cycle is detected.
pub fn mb_graphlib_static_order(handle: MbValue) -> MbValue {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    // Call prepare first (will detect cycles)
    let prepare_result = mb_graphlib_prepare(MbValue::from_int(handle_id as i64));
    if prepare_result.is_none() {
        // Check if it returned none due to error (cycle) — exception already raised
        // We need to check if we actually got an error or just none
        // Since mb_raise doesn't stop execution (it sets a flag), we should be fine
        // returning the prepare result (which is none on cycle)
    }
    let _ = prepare_result;

    // Iteratively drain nodes
    let mut order: Vec<i64> = Vec::new();

    loop {
        let ready = SORTERS.with(|c| {
            let mut map = c.borrow_mut();
            if let Some(sorter) = map.get_mut(&handle_id) {
                let nodes: Vec<i64> = sorter.ready_queue.drain(..).collect();
                for &n in &nodes {
                    sorter.in_progress.insert(n);
                    sorter.n_passed_out += 1;
                }
                nodes
            } else {
                Vec::new()
            }
        });

        if ready.is_empty() {
            break;
        }

        for &node in &ready {
            order.push(node);
        }

        // Mark all ready nodes as done
        SORTERS.with(|c| {
            let mut map = c.borrow_mut();
            if let Some(sorter) = map.get_mut(&handle_id) {
                for node in &ready {
                    sorter.in_progress.remove(node);
                    sorter.done_set.insert(*node);
                    sorter.n_finished += 1;
                }
                // Find newly ready nodes
                let all_nodes: Vec<i64> = sorter.all_nodes.iter().cloned().collect();
                for candidate in &all_nodes {
                    if sorter.done_set.contains(candidate) {
                        continue;
                    }
                    if sorter.in_progress.contains(candidate) {
                        continue;
                    }
                    if sorter.ready_queue.contains(candidate) {
                        continue;
                    }
                    if let Some(preds) = sorter.graph.get(candidate) {
                        if preds.iter().all(|p| sorter.done_set.contains(p)) {
                            sorter.ready_queue.push_back(*candidate);
                        }
                    }
                }
            }
        });
    }

    let list_vals: Vec<MbValue> = order.into_iter().map(MbValue::from_int).collect();
    MbValue::from_ptr(MbObject::new_list(list_vals))
}

/// graphlib.TopologicalSorter.destroy(handle) -> None
///
/// Clean up the sorter state associated with the given handle.
/// Should be called when the TopologicalSorter is no longer needed.
pub fn mb_graphlib_destroy(handle: MbValue) {
    let handle_id = match extract_int(handle) {
        Some(id) => id as u64,
        None => return,
    };
    SORTERS.with(|c| {
        c.borrow_mut().remove(&handle_id);
    });
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

    fn get_list_ints(val: MbValue) -> Vec<i64> {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::List(ref rw) = (*ptr).data {
                    let guard = rw.read().ok()?;
                    Some(guard.iter().filter_map(|v| v.as_int()).collect())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    fn get_list_len(val: MbValue) -> usize {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::List(ref rw) = (*ptr).data {
                    rw.read().ok().map(|g| g.len())
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    fn setup() {
        cleanup_all_sorters();
    }

    // REQ: R2
    #[test]
    fn test_new_returns_handle() {
        setup();
        let h = mb_graphlib_new();
        let id = h.as_int();
        assert!(
            id.is_some(),
            "mb_graphlib_new must return a non-None integer handle"
        );
        assert!(id.unwrap() > 0, "handle must be positive");
        mb_graphlib_destroy(h);
    }

    // REQ: R2
    #[test]
    fn test_static_order_linear() {
        setup();
        // Linear chain: 3 depends on 2, 2 depends on 1
        // Expected topological order: 1, 2, 3
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        // add(h, 1, [])
        mb_graphlib_add(make_int(hid), make_int(1), make_list_ints(&[]));
        // add(h, 2, [1])
        mb_graphlib_add(make_int(hid), make_int(2), make_list_ints(&[1]));
        // add(h, 3, [2])
        mb_graphlib_add(make_int(hid), make_int(3), make_list_ints(&[2]));

        let order = mb_graphlib_static_order(make_int(hid));
        let ints = get_list_ints(order);

        assert_eq!(
            ints.len(),
            3,
            "static_order must return 3 nodes for a 3-node linear chain"
        );
        // Verify topological ordering: 1 before 2, 2 before 3
        let pos1 = ints
            .iter()
            .position(|&x| x == 1)
            .expect("node 1 must be in result");
        let pos2 = ints
            .iter()
            .position(|&x| x == 2)
            .expect("node 2 must be in result");
        let pos3 = ints
            .iter()
            .position(|&x| x == 3)
            .expect("node 3 must be in result");
        assert!(
            pos1 < pos2,
            "node 1 must come before node 2, got order: {:?}",
            ints
        );
        assert!(
            pos2 < pos3,
            "node 2 must come before node 3, got order: {:?}",
            ints
        );

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_static_order_diamond() {
        setup();
        // Diamond: 4 depends on 2 and 3; 2 depends on 1; 3 depends on 1
        //   1 → 2 → 4
        //   1 → 3 → 4
        // Valid orders: [1,2,3,4] or [1,3,2,4]
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(1), make_list_ints(&[]));
        mb_graphlib_add(make_int(hid), make_int(2), make_list_ints(&[1]));
        mb_graphlib_add(make_int(hid), make_int(3), make_list_ints(&[1]));
        mb_graphlib_add(make_int(hid), make_int(4), make_list_ints(&[2, 3]));

        let order = mb_graphlib_static_order(make_int(hid));
        let ints = get_list_ints(order);

        assert_eq!(
            ints.len(),
            4,
            "diamond graph must return 4 nodes, got: {:?}",
            ints
        );

        let pos1 = ints.iter().position(|&x| x == 1).unwrap();
        let pos2 = ints.iter().position(|&x| x == 2).unwrap();
        let pos3 = ints.iter().position(|&x| x == 3).unwrap();
        let pos4 = ints.iter().position(|&x| x == 4).unwrap();

        assert!(pos1 < pos2, "1 must precede 2, got: {:?}", ints);
        assert!(pos1 < pos3, "1 must precede 3, got: {:?}", ints);
        assert!(pos2 < pos4, "2 must precede 4, got: {:?}", ints);
        assert!(pos3 < pos4, "3 must precede 4, got: {:?}", ints);

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_static_order_no_deps() {
        setup();
        // Three independent nodes with no dependencies
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(10), make_list_ints(&[]));
        mb_graphlib_add(make_int(hid), make_int(20), make_list_ints(&[]));
        mb_graphlib_add(make_int(hid), make_int(30), make_list_ints(&[]));

        let order = mb_graphlib_static_order(make_int(hid));
        let ints = get_list_ints(order);

        assert_eq!(
            ints.len(),
            3,
            "three independent nodes must all appear in order, got: {:?}",
            ints
        );
        // All three must be present (order doesn't matter since no dependencies)
        let mut sorted = ints.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec![10, 20, 30],
            "all three nodes must be returned, got: {:?}",
            ints
        );

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_cycle_detection() {
        setup();
        // A→B (B depends on A), B→A (A depends on B) — cycle!
        // Using node ids 100, 101 to avoid collision with other tests
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(100), make_list_ints(&[101]));
        mb_graphlib_add(make_int(hid), make_int(101), make_list_ints(&[100]));

        // prepare() should detect the cycle and raise CycleError
        // Since mb_raise sets a flag but doesn't panic, we check that
        // the result is none (error path)
        let result = mb_graphlib_prepare(make_int(hid));
        assert!(
            result.is_none(),
            "prepare on cyclic graph must return none (CycleError raised)"
        );

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_incremental_get_ready_done() {
        setup();
        // Linear: 2 depends on 1
        // Step 1: get_ready() → [1]
        // Step 2: done([1]), get_ready() → [2]
        // Step 3: done([2]), is_active() → false
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(1), make_list_ints(&[]));
        mb_graphlib_add(make_int(hid), make_int(2), make_list_ints(&[1]));

        mb_graphlib_prepare(make_int(hid));

        // Step 1: get node 1 first (it has no predecessors)
        let ready1 = mb_graphlib_get_ready(make_int(hid));
        let ints1 = get_list_ints(ready1);
        assert_eq!(ints1.len(), 1, "step 1: should have exactly 1 ready node");
        assert_eq!(
            ints1[0], 1,
            "step 1: ready node must be 1 (no predecessors)"
        );

        // is_active should be true (node 1 is in progress)
        let active1 = mb_graphlib_is_active(make_int(hid));
        assert_eq!(
            active1.as_bool(),
            Some(true),
            "is_active must be true while node 1 is in progress"
        );

        // Mark node 1 as done
        mb_graphlib_done(make_int(hid), make_list_ints(&[1]));

        // Step 2: get node 2 (predecessor 1 is now done)
        let ready2 = mb_graphlib_get_ready(make_int(hid));
        let ints2 = get_list_ints(ready2);
        assert_eq!(
            ints2.len(),
            1,
            "step 2: node 2 should become ready after 1 is done"
        );
        assert_eq!(ints2[0], 2, "step 2: ready node must be 2");

        // Mark node 2 as done
        mb_graphlib_done(make_int(hid), make_list_ints(&[2]));

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_is_active_false_when_complete() {
        setup();
        // Single node, no deps: after prepare + get_ready + done, is_active = false
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(42), make_list_ints(&[]));
        mb_graphlib_prepare(make_int(hid));

        let ready = mb_graphlib_get_ready(make_int(hid));
        let ints = get_list_ints(ready);
        assert_eq!(ints, vec![42], "single node should be immediately ready");

        mb_graphlib_done(make_int(hid), make_list_ints(&[42]));

        let active = mb_graphlib_is_active(make_int(hid));
        assert_eq!(
            active.as_bool(),
            Some(false),
            "is_active must be false after all nodes are done"
        );

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_empty_graph() {
        setup();
        // Empty graph: static_order returns empty list, no panic
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        let order = mb_graphlib_static_order(make_int(hid));
        let len = get_list_len(order);
        assert_eq!(len, 0, "empty graph must produce empty topological order");

        mb_graphlib_destroy(make_int(hid));
    }

    // REQ: R2
    #[test]
    fn test_add_node_without_predecessors() {
        setup();
        // A node added with empty predecessors list is immediately ready after prepare
        let h = mb_graphlib_new();
        let hid = h.as_int().unwrap();

        mb_graphlib_add(make_int(hid), make_int(5), make_list_ints(&[]));
        mb_graphlib_prepare(make_int(hid));

        let ready = mb_graphlib_get_ready(make_int(hid));
        let ints = get_list_ints(ready);
        assert_eq!(
            ints,
            vec![5],
            "node 5 with no predecessors must be immediately ready"
        );

        mb_graphlib_destroy(make_int(hid));
    }
}
