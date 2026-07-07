/// Shared re-entrancy guard for container repr/print formatting (#1061).
///
/// Mirrors CPython's `Py_ReprEnter`/`Py_ReprLeave` (Objects/object.c): a
/// thread-local stack of the raw pointers of containers currently being
/// formatted. Every repr/print path that recurses into a dict/list/tuple/
/// set/frozenset's elements (`mb_repr` → `value_to_string`, `mb_print`,
/// `print_repr`) pushes the container's pointer before descending into its
/// elements and pops it on the way back out. A container whose pointer is
/// already on the stack has been re-entered through a cycle (e.g.
/// `d['x'] = d`) and must short-circuit to CPython's ellipsis marker instead
/// of recursing again — recursing would otherwise stack-overflow.
use std::cell::RefCell;

thread_local! {
    static IN_PROGRESS: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

/// Attempt to enter `ptr`'s repr/print. Returns `true` the first time — the
/// caller should format normally and call [`leave`] with the same `ptr`
/// afterward. Returns `false` when `ptr` is already being formatted higher
/// up the call stack (a cycle); the caller must emit the cycle marker
/// instead and must NOT call [`leave`].
pub fn enter(ptr: usize) -> bool {
    IN_PROGRESS.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.contains(&ptr) {
            false
        } else {
            stack.push(ptr);
            true
        }
    })
}

/// Leave `ptr`'s repr/print, pairing a `true`-returning [`enter`]. Callers
/// always enter/leave in strict LIFO order (the recursive descent is fully
/// nested within the enter/leave pair), so popping the stack's top is
/// always the matching entry.
pub fn leave(ptr: usize) {
    IN_PROGRESS.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert_eq!(popped, Some(ptr), "repr_guard::leave/enter mismatch");
    });
}
