// HANDWRITE-BEGIN gap="missing-generator:mamba-strict-type-provenance-static-inventory" tracker="#1453" reason="The strict provenance proof needs runtime-level reentrancy probes and an explicit inventory boundary."
//! Runtime probes for the strict raw-or-boxed Int provenance proof (#1453).
//!
//! This module deliberately tests the public dynamic gateways rather than
//! recreating their ownership logic.  The emitted JIT tests cover static MIR
//! producers; these probes cover the thread-local argument/return frames that
//! remain at dynamic and reentrant ABI boundaries.

#[cfg(test)]
mod tests {
    use crate::runtime::argument_owner;
    use crate::runtime::bigint_ops;
    use crate::runtime::builtins;
    use crate::runtime::rc::{self, MbObject};
    use crate::runtime::return_owner;
    use crate::runtime::value::MbValue;

    extern "C" fn published_bigint() -> MbValue {
        let value = bigint_ops::bigint_from_i128(1_i128 << 70);
        return_owner::publish_return_owner(value, value);
        value
    }

    extern "C" fn owner_echo(value: MbValue) -> MbValue {
        let owner = argument_owner::matching_argument_owner_slot(0, value);
        return_owner::publish_return_owner(value, owner);
        value
    }

    extern "C" fn reentrant_published_bigint() -> MbValue {
        let outer = bigint_ops::bigint_from_i128(1_i128 << 70);
        return_owner::publish_return_owner(outer, outer);

        // This is the same central dynamic dispatch used by profile/weakref
        // callbacks: the inner callback must consume its token before the
        // outer callback returns to its caller.
        let inner = builtins::dispatch_jit_frame(published_bigint as usize, &[], false);
        unsafe { rc::release_if_ptr(inner) };
        outer
    }

    fn assert_owned_bigint_once(value: MbValue) {
        let object = value.as_ptr().expect("dynamic gateway must return BigInt");
        assert_eq!(unsafe { rc::mb_refcount(object) }, 1);
        unsafe { rc::release_if_ptr(value) };
    }

    fn reset_frames() {
        argument_owner::clear_argument_owner_frames_for_test();
        return_owner::clear_return_owner_frames_for_test();
    }

    #[test]
    fn dynamic_return_routes_finalize_one_matching_owner() {
        reset_frames();

        let direct = builtins::dispatch_jit_frame(published_bigint as usize, &[], false);
        assert_owned_bigint_once(direct);

        let callable = MbValue::from_func(published_bigint as *const () as usize);
        let args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let spread = builtins::mb_call_spread(callable, args);
        unsafe { rc::release_if_ptr(args) };
        assert_owned_bigint_once(spread);

        let input = bigint_ops::bigint_from_i128((1_i128 << 70) + 7);
        let forwarded = builtins::dispatch_jit_frame(owner_echo as usize, &[input], false);
        assert_eq!(forwarded.to_bits(), input.to_bits());
        assert_owned_bigint_once(forwarded);

        assert_eq!(argument_owner::argument_owner_frame_depth(), 0);
        assert_eq!(return_owner::return_owner_frame_depth(), 0);
    }

    #[test]
    fn nested_profile_weakref_callbacks_restore_owner_frames() {
        reset_frames();

        // The profile and weakref implementations invoke dynamic callables;
        // their source contracts must stay routed through the same gateways
        // exercised by this nested callback.  Keeping these assertions here
        // makes a future direct raw callable route visible to the proof suite.
        let profile = include_str!("stdlib/threading_mod.rs");
        let weakref = include_str!("stdlib/weakref_mod.rs");
        assert!(profile.contains("call_trace_profile_hook"));
        assert!(weakref.contains("mb_weakref_finalize"));

        for _ in 0..3 {
            let result =
                builtins::dispatch_jit_frame(reentrant_published_bigint as usize, &[], false);
            assert_owned_bigint_once(result);
            assert_eq!(argument_owner::argument_owner_frame_depth(), 0);
            assert_eq!(return_owner::return_owner_frame_depth(), 0);
        }
    }
}

// marker: missing-generator:mamba-strict-type-provenance-static-inventory
// HANDWRITE-END
