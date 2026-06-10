pub mod value;
pub mod rc;
pub mod integer_handle_registry;
pub mod output;
pub mod builtins;
pub mod bigint_ops;
pub mod string_ops;
pub mod list_ops;
pub mod dict_ops;
pub mod tuple_ops;
pub mod set_ops;
pub mod bytes_ops;
pub mod exception;
pub mod class;
pub mod iter;
pub mod generator;
pub mod closure;
pub mod module;
pub mod registry_bridge;
pub mod async_rt;
pub mod async_task;
pub mod tokio_exec;
pub mod gc;
#[allow(non_snake_case)]
pub mod stdlib;
pub mod file_io;
pub mod symbols;

pub use value::MbValue;
pub use rc::{MbObject, MbObjectHeader};

/// Centralized runtime cleanup: reset all thread_local state in dependency order.
///
/// Order: iterators → generators → closures → classes → exceptions → files →
/// modules → async → **GC clear** → module JIT backend cleanup.
///
/// Notable design decisions:
///
/// - **Generator close skipped**: `cleanup_all_generators()` calls
///   `mb_generator_close()` and resumes coroutines to run finally blocks.
///   Runtime teardown cannot safely resume JIT frames after partial cleanup,
///   so it discards the generator registry and coroutine stacks directly.
///
/// - **GC state cleared (not collected)**: The cycle collector lacks stack
///   scanning, so it cannot distinguish live JIT stack values from garbage.
///   Running collect() here would double-free objects referenced by both
///   GC-tracked containers and leaked runtime registries. Instead, the tracked
///   set is discarded entirely.
pub fn cleanup_all_runtime_state() {
    // Phase 1: Clear runtime registries that hold MbValues.
    // Order matters: iterators may reference closures/classes, so clear them first.
    iter::cleanup_all_iterators();
    generator::cleanup_generator_state_for_runtime_reset();
    closure::cleanup_all_closures();
    class::cleanup_all_classes();
    stdlib::dataclasses_mod::cleanup_all_dataclasses();
    exception::cleanup_all_exceptions();
    file_io::cleanup_all_files();
    module::cleanup_all_modules();
    async_rt::cleanup_all_async();
    // Phase 2: Clear GC tracking (objects may already be freed by phase 1 releases).
    gc::gc_clear_all_state();
    // Phase 3: Drop module JIT backend handles last. Imported module values
    // have been detached before this point, so backend drops cannot cascade
    // through mixed-owned module attrs.
    module::cleanup_module_jit_backends();
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;
    use std::collections::HashMap;

    // ── S1: Full runtime cleanup resets all thread_local statics ──

    #[test]
    fn test_cleanup_all_runtime_state_resets_closures() {
        // Populate closure state
        let name = MbValue::from_ptr(rc::MbObject::new_str("test_fn".into()));
        let func = MbValue::from_int(1);
        let caps = MbValue::from_ptr(rc::MbObject::new_list(vec![MbValue::from_int(42)]));
        let _handle = closure::mb_closure_new(name, func, caps);

        // Populate cell state
        let _cell = closure::mb_cell_new(MbValue::from_int(99));

        // Populate global namespace
        let gname = MbValue::from_ptr(rc::MbObject::new_str("my_global".into()));
        closure::mb_global_set(gname, MbValue::from_int(7));

        cleanup_all_runtime_state();

        // After cleanup, closure handle should resolve to none
        let bad = MbValue::from_int(1); // the ID that was allocated
        assert!(closure::mb_closure_get_func(bad).is_none(),
            "CLOSURES should be empty after cleanup");

        // Global should be gone
        let gname2 = MbValue::from_ptr(rc::MbObject::new_str("my_global".into()));
        assert!(closure::mb_global_get(gname2).is_none(),
            "GLOBAL_NAMESPACE should be empty after cleanup");
    }

    #[test]
    fn test_cleanup_all_runtime_state_resets_iterators() {
        // Create an iterator
        let list = MbValue::from_ptr(rc::MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ]));
        let it = iter::mb_iter(list);
        assert!(it.is_int(), "should get a valid iterator handle");

        cleanup_all_runtime_state();

        // After cleanup, next on the old handle should return none (handle gone)
        let val = iter::mb_next(it);
        assert!(val.is_none(), "ITERATORS should be empty after cleanup");
    }

    #[test]
    fn test_cleanup_all_runtime_state_discards_generator_handles() {
        let name = MbValue::from_ptr(rc::MbObject::new_str("cleanup_gen".into()));
        let gen = generator::mb_generator_create(name, MbValue::none());
        assert!(generator::is_known_generator(gen), "generator should be registered");

        cleanup_all_runtime_state();

        assert!(
            !generator::is_known_generator(gen),
            "GENERATOR registry should be empty after cleanup"
        );
        let method = MbValue::from_ptr(rc::MbObject::new_str("bit_length".into()));
        let args = MbValue::from_ptr(rc::MbObject::new_list(vec![]));
        assert_eq!(
            class::mb_call_method(MbValue::from_int(1), method, args).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_cleanup_all_runtime_state_resets_classes() {
        class::mb_class_register("CleanupTestClass", vec![], HashMap::new());

        cleanup_all_runtime_state();

        // After cleanup, isinstance check on a fresh instance should still
        // be false for the old class (registry cleared)
        // We verify by checking that a new instance lookup won't find methods
        let class_name = MbValue::from_ptr(rc::MbObject::new_str("CleanupTestClass".into()));
        let inst = class::mb_instance_new(class_name, MbValue::none());
        // The instance is created but class registry is empty, so isinstance
        // against a non-registered class yields false for non-matching type
        let check_type = MbValue::from_ptr(rc::MbObject::new_str("SomeOtherClass".into()));
        assert_eq!(class::mb_isinstance(inst, check_type).as_bool(), Some(false));
    }

    #[test]
    fn test_cleanup_all_runtime_state_resets_exceptions() {
        let typ = MbValue::from_ptr(rc::MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(rc::MbObject::new_str("test".into()));
        exception::mb_raise(typ, msg);
        assert_eq!(exception::mb_has_exception().as_bool(), Some(true));

        cleanup_all_runtime_state();

        assert_eq!(exception::mb_has_exception().as_bool(), Some(false),
            "CURRENT_EXCEPTION should be None after cleanup");
    }

    #[test]
    fn test_cleanup_all_runtime_state_resets_modules() {
        let mut attrs = HashMap::new();
        attrs.insert("x".to_string(), MbValue::from_int(42));
        module::mb_module_register("cleanup_test_mod", attrs);

        cleanup_all_runtime_state();

        // After cleanup, import should fail (module no longer registered)
        let name = MbValue::from_ptr(rc::MbObject::new_str("cleanup_test_mod".into()));
        let result = module::mb_import(name);
        assert!(result.is_none(), "MODULES should be empty after cleanup");
    }

    // ── S2: Cleanup ordering (verified structurally) ──

    #[test]
    fn test_cleanup_ordering_generators_before_iterators_before_closures() {
        // This test verifies the function doesn't panic when called
        // with state in multiple modules simultaneously.
        // The ordering guarantee (generators → iterators → closures → ...)
        // is structural from the code, but we verify no panics occur
        // when all modules have state.

        // Populate iterators
        let list = MbValue::from_ptr(rc::MbObject::new_list(vec![MbValue::from_int(1)]));
        let _it = iter::mb_iter(list);

        // Populate closures
        let name = MbValue::from_ptr(rc::MbObject::new_str("fn".into()));
        let func = MbValue::from_int(1);
        let caps = MbValue::from_ptr(rc::MbObject::new_list(vec![]));
        let _cl = closure::mb_closure_new(name, func, caps);

        // Populate classes
        class::mb_class_register("OrderTestClass", vec![], HashMap::new());

        // Populate exceptions
        let typ = MbValue::from_ptr(rc::MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(rc::MbObject::new_str("order test".into()));
        exception::mb_raise(typ, msg);

        // Populate modules
        let mut attrs = HashMap::new();
        attrs.insert("v".to_string(), MbValue::from_int(1));
        module::mb_module_register("order_test_mod", attrs);

        // Full cleanup should not panic and should clear everything
        cleanup_all_runtime_state();

        // Verify all cleared
        assert_eq!(exception::mb_has_exception().as_bool(), Some(false));
        let mod_name = MbValue::from_ptr(rc::MbObject::new_str("order_test_mod".into()));
        assert!(module::mb_import(mod_name).is_none());
    }

    // ── S4: Panic safety ──

    #[test]
    fn test_cleanup_is_panic_safe_independent_modules() {
        // Verify that cleanup of one module does not prevent others.
        // We populate exceptions and modules, then clean up.
        // Even if one module had issues, the other should be cleaned.
        let typ = MbValue::from_ptr(rc::MbObject::new_str("TypeError".into()));
        let msg = MbValue::from_ptr(rc::MbObject::new_str("panic test".into()));
        exception::mb_raise(typ, msg);

        let mut attrs = HashMap::new();
        attrs.insert("k".to_string(), MbValue::from_int(1));
        module::mb_module_register("panic_safe_mod", attrs);

        cleanup_all_runtime_state();

        // Both should be cleared
        assert_eq!(exception::mb_has_exception().as_bool(), Some(false),
            "exceptions should be cleaned regardless of other modules");
        let name = MbValue::from_ptr(rc::MbObject::new_str("panic_safe_mod".into()));
        assert!(module::mb_import(name).is_none(),
            "modules should be cleaned regardless of other modules");
    }

    // ── S5: Conformance runner uses cleanup_all_runtime_state (structural) ──

    #[test]
    fn test_cleanup_all_runtime_state_is_callable() {
        // Simple smoke test: calling cleanup on empty state should not panic
        cleanup_all_runtime_state();
    }

    #[test]
    fn test_cleanup_idempotent() {
        // Calling cleanup multiple times should not panic
        cleanup_all_runtime_state();
        cleanup_all_runtime_state();
        cleanup_all_runtime_state();
    }

    // ── S6: Multi-threaded cleanup ──

    #[test]
    fn test_cleanup_per_thread_isolation() {
        // Verify that cleanup on one thread does not affect another thread's state.
        // Each thread has its own thread_locals.
        use std::sync::{Arc, Barrier};

        let barrier = Arc::new(Barrier::new(2));
        let b1 = barrier.clone();
        let b2 = barrier.clone();

        let t1 = std::thread::spawn(move || {
            // Thread 1: populate state
            let name = MbValue::from_ptr(rc::MbObject::new_str("t1_global".into()));
            closure::mb_global_set(name, MbValue::from_int(111));
            b1.wait(); // sync: both threads have set state
            b1.wait(); // sync: wait for t2 to cleanup
            // Thread 1's state should still be present (t2's cleanup is independent)
            let name2 = MbValue::from_ptr(rc::MbObject::new_str("t1_global".into()));
            let val = closure::mb_global_get(name2);
            assert_eq!(val.as_int(), Some(111),
                "thread 1's state should survive thread 2's cleanup");
        });

        let t2 = std::thread::spawn(move || {
            // Thread 2: populate and cleanup
            let name = MbValue::from_ptr(rc::MbObject::new_str("t2_global".into()));
            closure::mb_global_set(name, MbValue::from_int(222));
            b2.wait(); // sync: both threads have set state
            cleanup_all_runtime_state();
            b2.wait(); // sync: signal t1 that cleanup is done
            // Thread 2's state should be gone
            let name2 = MbValue::from_ptr(rc::MbObject::new_str("t2_global".into()));
            assert!(closure::mb_global_get(name2).is_none(),
                "thread 2's state should be cleared after its own cleanup");
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    mod async_gen_event_loop_interleaving_gate;
    mod base64_memory_gate;
    mod container_lock_perf;
    mod generator_runtime_type_gate;
    mod jit_refcount_audit;
    mod list_literal_perf;
    mod list_sort_builtin_perf_gate;
    mod pymalloc_freelist;
    mod runtime_core;
    mod runtime_integration;
    mod stdlib_coverage_lower;
    mod stdlib_coverage_remaining;
    mod string_concat_perf_gate;
    mod thread_safety;
}
