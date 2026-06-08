//! Integer-handle refcount registry for #2111 (Subset A iteration-retention amplifier).
//!
//! Stdlib modules that follow the integer-handle pattern (array, hashlib,
//! BytesIO, etc. — see `project_mamba_integer_handle_pattern`) store their
//! state in a `thread_local!` table keyed by an integer ID, and return that
//! ID NaN-boxed as `MbValue::from_int`. Because integer values have no
//! heap pointer, the JIT's `mb_retain_value` / `mb_release_value` calls
//! were no-ops, so per-iter rebinds (`a = make_array(...)`) leaked the
//! prior handle's backing storage until process exit.
//!
//! This registry lets each handle-pattern module register a `(retain,
//! release)` pair that the rc-aware JIT path dispatches to for integer
//! values. Hooks return `true` if they owned the id and applied the
//! operation, allowing `mb_retain_value` / `mb_release_value` to stop
//! iterating early.

use std::cell::RefCell;

/// Minimum handle id eligible for refcount dispatch. JIT-compiled code
/// constantly creates and discards small primitive `int` values (`x = 1`,
/// `i in range(...)`, etc.), and `MbValue::from_int(1)` is bit-identical
/// to a handle id of 1. If a module allocated handle id `1`, every
/// primitive-int release of `1` would spuriously decrement that handle.
///
/// Modules following the integer-handle pattern MUST start their
/// `NEXT_<NAME>_ID` counter at or above this threshold so that no
/// realistic primitive int collides with a live handle. `2^40` is high
/// enough to leave room for ~1 trillion handle allocations while
/// remaining well below the NaN-boxed `MbValue::from_int` capacity
/// (≈51 bits). Per-module names: `QUEUE_HANDLE_BASE`,
/// `FRACTION_HANDLE_BASE`, `UUID_HANDLE_BASE`, `IP_HANDLE_BASE`, etc.
pub const HANDLE_MIN_ID: u64 = 1u64 << 40;

/// Hook pair for one handle-pattern module.
#[derive(Clone, Copy)]
pub struct IntegerHandleHooks {
    /// Bump the per-handle refcount if `id` is one of mine. Returns
    /// `true` on hit, `false` if `id` is not in my table.
    pub retain: fn(u64) -> bool,
    /// Decrement the per-handle refcount if `id` is one of mine; drop
    /// the entry when the count reaches zero. Returns `true` on hit.
    pub release: fn(u64) -> bool,
}

thread_local! {
    static HOOKS: RefCell<Vec<IntegerHandleHooks>> = const { RefCell::new(Vec::new()) };
}

/// Register a module's retain/release hook pair. Called once from each
/// handle-pattern module's `register()`.
pub fn register(hooks: IntegerHandleHooks) {
    HOOKS.with(|h| h.borrow_mut().push(hooks));
}

/// Dispatch retain to the first hook that owns `id`. No-op if `id` is
/// below `HANDLE_MIN_ID` (primitive int) or not in any handle table.
#[inline]
pub fn retain(id: u64) {
    if id < HANDLE_MIN_ID {
        return;
    }
    HOOKS.with(|h| {
        for hook in h.borrow().iter() {
            if (hook.retain)(id) {
                return;
            }
        }
    });
}

/// Dispatch release to the first hook that owns `id`. No-op if `id` is
/// below `HANDLE_MIN_ID` (primitive int) or not in any handle table.
#[inline]
pub fn release(id: u64) {
    if id < HANDLE_MIN_ID {
        return;
    }
    HOOKS.with(|h| {
        for hook in h.borrow().iter() {
            if (hook.release)(id) {
                return;
            }
        }
    });
}
