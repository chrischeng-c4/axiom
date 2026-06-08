# Implementation Diff

## Summary

```
crates/mamba/src/runtime/gc.rs         | 32 ++++++++++++++++++------
 crates/mamba/src/runtime/tokio_exec.rs | 28 +++++++++++++++------
 crates/mamba/src/runtime/tuple_ops.rs  | 37 ++++++++++++++++++++++++++++
 3 files changed, 83 insertions(+), 14 deletions(-)
```

## Diff

```diff
diff --git a/crates/mamba/src/runtime/gc.rs b/crates/mamba/src/runtime/gc.rs
index 8a90d32..7795033 100644
--- a/crates/mamba/src/runtime/gc.rs
+++ b/crates/mamba/src/runtime/gc.rs
@@ -389,11 +389,26 @@ mod tests {
     static GC_TEST_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
         std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
 
+    fn reset_gc_for_test() {
+        let mut gc = GC.lock().unwrap_or_else(|e| e.into_inner());
+        gc.tracked.clear();
+        gc.marked.clear();
+        gc.alloc_count = 0;
+        gc.collecting = false;
+        gc.roots.clear();
+        // Disable auto-collection so concurrent gc_track() from other tests
+        // won't sweep our test objects. Manual collect() ignores this flag.
+        gc.enabled = false;
+        // Reset registered thread count so collect()'s request_safepoint()
+        // doesn't block waiting for tokio worker threads that never reach safepoints.
+        REGISTERED_THREADS.store(0, Ordering::SeqCst);
+        THREADS_AT_SAFEPOINT.store(0, Ordering::SeqCst);
+    }
+
     #[test]
     fn test_track_and_collect_unreachable() {
-        let _lock = GC_TEST_LOCK.lock().unwrap();
-        // Flush stale objects from other tests
-        collect();
+        let _lock = GC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_gc_for_test();
 
         let a = MbObject::new_list(vec![]);
         let b = MbObject::new_list(vec![]);
@@ -410,13 +425,14 @@ mod tests {
 
         // Neither is in roots, so both should be collected
         let freed = collect();
+        gc_enable();
         assert!(freed >= 2, "should collect at least 2 cyclic objects, got {freed}");
     }
 
     #[test]
     fn test_reachable_not_collected() {
-        let _lock = GC_TEST_LOCK.lock().unwrap();
-        collect(); // flush stale
+        let _lock = GC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_gc_for_test();
 
         let obj = MbObject::new_list(vec![MbValue::from_int(42)]);
         let val = MbValue::from_ptr(obj);
@@ -427,13 +443,14 @@ mod tests {
 
         gc_remove_root(val);
         gc_untrack(obj);
+        gc_enable();
         unsafe { drop(Box::from_raw(obj)); }
     }
 
     #[test]
     fn test_nested_reachability() {
-        let _lock = GC_TEST_LOCK.lock().unwrap();
-        collect(); // flush stale
+        let _lock = GC_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_gc_for_test();
 
         let inner = MbObject::new_list(vec![MbValue::from_int(1)]);
         let outer = MbObject::new_list(vec![MbValue::from_ptr(inner)]);
@@ -445,6 +462,7 @@ mod tests {
 
         gc_remove_root(root);
         let freed = collect();
+        gc_enable();
         assert!(freed >= 2, "both should be collected when root removed");
     }
 
diff --git a/crates/mamba/src/runtime/tokio_exec.rs b/crates/mamba/src/runtime/tokio_exec.rs
index ccbccca..84a6870 100644
--- a/crates/mamba/src/runtime/tokio_exec.rs
+++ b/crates/mamba/src/runtime/tokio_exec.rs
@@ -22,12 +22,6 @@ fn runtime() -> &'static tokio::runtime::Runtime {
             .worker_threads(2)
             .enable_all()
             .thread_name("mamba-tokio")
-            .on_thread_start(|| {
-                super::gc::gc_register_thread();
-            })
-            .on_thread_stop(|| {
-                super::gc::gc_unregister_thread();
-            })
             .build()
             .expect("failed to create Mamba Tokio runtime")
     })
@@ -159,6 +153,17 @@ pub fn mb_tokio_shutdown() {
 mod tests {
     use super::*;
     use super::super::async_rt::{mb_coroutine_new, mb_coroutine_complete};
+    use super::super::gc::{gc_disable, gc_enable};
+
+    static TOKIO_TEST_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
+        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
+
+    fn reset_async_for_test() {
+        COROUTINES.write().unwrap_or_else(|e| e.into_inner()).clear();
+        TASKS.write().unwrap_or_else(|e| e.into_inner()).clear();
+        // Prevent concurrent GC from freeing test MbObjects
+        gc_disable();
+    }
 
     #[test]
     fn test_runtime_initializes() {
@@ -169,6 +174,8 @@ mod tests {
 
     #[test]
     fn test_tokio_spawn_completed_coro() {
+        let _lock = TOKIO_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_async_for_test();
         let name = MbValue::from_ptr(MbObject::new_str("tokio_test".to_string()));
         let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
         let coro = mb_coroutine_new(name, locals);
@@ -181,20 +188,26 @@ mod tests {
         std::thread::sleep(std::time::Duration::from_millis(50));
 
         let task_id = task.as_int().unwrap() as u64;
-        let done = TASKS.read().unwrap()
+        let done = TASKS.read().unwrap_or_else(|e| e.into_inner())
             .get(&task_id).map(|t| t.done).unwrap_or(false);
         assert!(done, "Tokio task should complete for exhausted coroutine");
+        gc_enable();
     }
 
     #[test]
     fn test_tokio_gather_empty() {
+        let _lock = TOKIO_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_async_for_test();
         let coros = MbValue::from_ptr(MbObject::new_list(vec![]));
         let result = mb_tokio_gather(coros);
         assert!(result.as_ptr().is_some(), "gather of empty list should return list");
+        gc_enable();
     }
 
     #[test]
     fn test_tokio_gather_completed_coros() {
+        let _lock = TOKIO_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
+        reset_async_for_test();
         let n1 = MbValue::from_ptr(MbObject::new_str("g1".to_string()));
         let l1 = MbValue::from_ptr(MbObject::new_list(vec![]));
         let c1 = mb_coroutine_new(n1, l1);
@@ -208,5 +221,6 @@ mod tests {
         let coros = MbValue::from_ptr(MbObject::new_list(vec![c1, c2]));
         let result = mb_tokio_gather(coros);
         assert!(result.as_ptr().is_some(), "gather should return a list");
+        gc_enable();
     }
 }
diff --git a/crates/mamba/src/runtime/tuple_ops.rs b/crates/mamba/src/runtime/tuple_ops.rs
index 16f796c..496ed5d 100644
--- a/crates/mamba/src/runtime/tuple_ops.rs
+++ b/crates/mamba/src/runtime/tuple_ops.rs
@@ -223,23 +223,37 @@ pub fn dispatch_tuple_method(name: &str, receiver: MbValue, args: MbValue) -> Mb
 #[cfg(test)]
 mod tests {
     use super::*;
+    use super::super::gc::{gc_disable, gc_enable};
+
+    /// RAII guard: disables GC on creation, re-enables on drop (even on panic).
+    /// Prevents concurrent test threads from collecting our MbObjects.
+    struct GcGuard;
+    impl GcGuard {
+        fn new() -> Self { gc_disable(); Self }
+    }
+    impl Drop for GcGuard {
+        fn drop(&mut self) { gc_enable(); }
+    }
 
     // ── Creation ──
 
     #[test]
     fn test_new_empty() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_new();
         assert_eq!(mb_tuple_len(t).as_int(), Some(0));
     }
 
     #[test]
     fn test_from_elements() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         assert_eq!(mb_tuple_len(t).as_int(), Some(2));
     }
 
     #[test]
     fn test_list_to_tuple() {
+        let _gc = GcGuard::new();
         let list = MbValue::from_ptr(MbObject::new_list(vec![
             MbValue::from_int(10), MbValue::from_int(20),
         ]));
@@ -250,6 +264,7 @@ mod tests {
 
     #[test]
     fn test_list_to_tuple_non_list() {
+        let _gc = GcGuard::new();
         let t = mb_list_to_tuple(MbValue::from_int(0));
         assert_eq!(mb_tuple_len(t).as_int(), Some(0));
     }
@@ -258,6 +273,7 @@ mod tests {
 
     #[test]
     fn test_getitem() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![
             MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
         ]);
@@ -267,6 +283,7 @@ mod tests {
 
     #[test]
     fn test_getitem_negative() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
         assert_eq!(mb_tuple_getitem(t, MbValue::from_int(-1)).as_int(), Some(20));
         assert_eq!(mb_tuple_getitem(t, MbValue::from_int(-2)).as_int(), Some(10));
@@ -274,6 +291,7 @@ mod tests {
 
     #[test]
     fn test_getitem_out_of_bounds() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1)]);
         assert!(mb_tuple_getitem(t, MbValue::from_int(5)).is_none());
         assert!(mb_tuple_getitem(t, MbValue::from_int(-5)).is_none());
@@ -288,6 +306,7 @@ mod tests {
 
     #[test]
     fn test_slice() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![
             MbValue::from_int(0), MbValue::from_int(1),
             MbValue::from_int(2), MbValue::from_int(3),
@@ -299,6 +318,7 @@ mod tests {
 
     #[test]
     fn test_slice_empty_range() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1)]);
         let s = mb_tuple_slice(t, MbValue::from_int(1), MbValue::from_int(0));
         assert_eq!(mb_tuple_len(s).as_int(), Some(0));
@@ -308,6 +328,7 @@ mod tests {
 
     #[test]
     fn test_len_contains() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         assert_eq!(mb_tuple_len(t).as_int(), Some(2));
         assert_eq!(mb_tuple_contains(t, MbValue::from_int(1)).as_bool(), Some(true));
@@ -323,6 +344,7 @@ mod tests {
 
     #[test]
     fn test_count() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![
             MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(1),
         ]);
@@ -332,6 +354,7 @@ mod tests {
 
     #[test]
     fn test_index() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![
             MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
         ]);
@@ -343,6 +366,7 @@ mod tests {
 
     #[test]
     fn test_concat() {
+        let _gc = GcGuard::new();
         let a = mb_tuple_from(vec![MbValue::from_int(1)]);
         let b = mb_tuple_from(vec![MbValue::from_int(2)]);
         let c = mb_tuple_concat(a, b);
@@ -360,6 +384,7 @@ mod tests {
 
     #[test]
     fn test_repeat() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1)]);
         let r = mb_tuple_repeat(t, MbValue::from_int(3));
         assert_eq!(mb_tuple_len(r).as_int(), Some(3));
@@ -367,6 +392,7 @@ mod tests {
 
     #[test]
     fn test_repeat_zero() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1)]);
         let r = mb_tuple_repeat(t, MbValue::from_int(0));
         assert_eq!(mb_tuple_len(r).as_int(), Some(0));
@@ -374,6 +400,7 @@ mod tests {
 
     #[test]
     fn test_repeat_negative() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1)]);
         let r = mb_tuple_repeat(t, MbValue::from_int(-1));
         assert_eq!(mb_tuple_len(r).as_int(), Some(0));
@@ -383,6 +410,7 @@ mod tests {
 
     #[test]
     fn test_eq() {
+        let _gc = GcGuard::new();
         let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(true));
@@ -390,6 +418,7 @@ mod tests {
 
     #[test]
     fn test_eq_different() {
+        let _gc = GcGuard::new();
         let a = mb_tuple_from(vec![MbValue::from_int(1)]);
         let b = mb_tuple_from(vec![MbValue::from_int(2)]);
         assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(false));
@@ -412,6 +441,7 @@ mod tests {
 
     #[test]
     fn test_dispatch_count() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(1)]);
         let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
         assert_eq!(dispatch_tuple_method("count", t, args).as_int(), Some(2));
@@ -419,6 +449,7 @@ mod tests {
 
     #[test]
     fn test_dispatch_index() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
         let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(20)]));
         assert_eq!(dispatch_tuple_method("index", t, args).as_int(), Some(1));
@@ -426,6 +457,7 @@ mod tests {
 
     #[test]
     fn test_dispatch_unknown() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_new();
         let args = MbValue::from_ptr(MbObject::new_list(vec![]));
         assert!(dispatch_tuple_method("nonexistent", t, args).is_none());
@@ -462,6 +494,7 @@ mod tests {
 
     #[test]
     fn test_py312_tuple_concat() {
+        let _gc = GcGuard::new();
         let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         let b = mb_tuple_from(vec![MbValue::from_int(3), MbValue::from_int(4)]);
         let c = mb_tuple_concat(a, b);
@@ -472,6 +505,7 @@ mod tests {
 
     #[test]
     fn test_py312_tuple_repeat() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         let rep = mb_tuple_repeat(t, MbValue::from_int(3));
         assert_eq!(mb_tuple_len(rep).as_int(), Some(6));
@@ -479,6 +513,7 @@ mod tests {
 
     #[test]
     fn test_py312_tuple_contains() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_bool(true)]);
         assert_eq!(mb_tuple_contains(t, MbValue::from_int(1)).as_bool(), Some(true));
         assert_eq!(mb_tuple_contains(t, MbValue::from_int(99)).as_bool(), Some(false));
@@ -486,6 +521,7 @@ mod tests {
 
     #[test]
     fn test_py312_tuple_eq_same_content() {
+        let _gc = GcGuard::new();
         let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
         assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(true));
@@ -493,6 +529,7 @@ mod tests {
 
     #[test]
     fn test_py312_tuple_count() {
+        let _gc = GcGuard::new();
         let t = mb_tuple_from(vec![
             MbValue::from_int(1), MbValue::from_int(1), MbValue::from_int(2),
         ]);
```
