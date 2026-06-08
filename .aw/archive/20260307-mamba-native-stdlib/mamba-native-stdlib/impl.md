# Implementation Diff

## Summary

```
Cargo.lock                                         |  79 ++---
 Cargo.toml                                         |   2 +-
 crates/mamba/Cargo.toml                      |   1 +
 crates/mamba/src/runtime/async_rt.rs         | 172 ++++++-----
 crates/mamba/src/runtime/async_task.rs       | 341 ++++++++-------------
 crates/mamba/src/runtime/class.rs            |   4 +
 crates/mamba/src/runtime/gc.rs               | 113 ++++++-
 crates/mamba/src/runtime/iter.rs             |   2 +
 crates/mamba/src/runtime/mod.rs              |   1 +
 crates/mamba/src/runtime/symbols.rs          |   4 +
 crates/mamba/src/runtime/tokio_exec.rs       | 212 +++++++++++++
 crates/mamba/tests/thread_safety_tests.rs    | 140 +++++++++
 crates/cclab-sdd/src/mcp/tools/agent.rs            |  10 +
 .../cclab-sdd/src/mcp/tools/change_impl/create.rs  | 128 +++++++-
 .../cclab-sdd/src/mcp/tools/change_spec/common.rs  |  26 ++
 crates/cclab-sdd/src/mcp/tools/phase_transition.rs |   2 +
 crates/cclab-sdd/src/workflow/advance.rs           |  11 +
 17 files changed, 913 insertions(+), 335 deletions(-)
```

## Diff

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 005f80f..c3d5b90 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1200,7 +1200,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1209,7 +1209,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "bson",
@@ -1227,7 +1227,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1254,7 +1254,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1281,7 +1281,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1294,7 +1294,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bitvec",
  "regex",
@@ -1321,7 +1321,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1331,7 +1331,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1339,7 +1339,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1363,7 +1363,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1381,7 +1381,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1410,7 +1410,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1422,7 +1422,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "clap",
@@ -1439,12 +1439,13 @@ dependencies = [
  "sha2",
  "target-lexicon",
  "thiserror 2.0.18",
+ "tokio",
  "toml",
 ]
 
 [[package]]
 name = "cclab-mamba-tests"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-mamba",
  "datatest-stable",
@@ -1454,7 +1455,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "image",
  "pyo3",
@@ -1465,7 +1466,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1487,7 +1488,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1518,7 +1519,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1548,7 +1549,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "serde",
@@ -1558,7 +1559,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1587,7 +1588,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1619,7 +1620,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1660,7 +1661,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1686,7 +1687,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1701,7 +1702,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1714,7 +1715,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1762,7 +1763,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1787,7 +1788,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1799,14 +1800,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bytemuck",
  "env_logger",
@@ -1836,7 +1837,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1853,7 +1854,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-asset"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "image",
@@ -1864,7 +1865,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-bundler"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1886,7 +1887,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-dev-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1905,7 +1906,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-pkg-manager"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "reqwest",
@@ -1922,7 +1923,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-resolver"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "node-resolve",
@@ -1935,7 +1936,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-transform"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "regex",
diff --git a/Cargo.toml b/Cargo.toml
index 811c830..013af6e 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -47,7 +47,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.28"
+version = "0.3.29"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/mamba/Cargo.toml b/crates/mamba/Cargo.toml
index 6713019..fb1ca2a 100644
--- a/crates/mamba/Cargo.toml
+++ b/crates/mamba/Cargo.toml
@@ -26,6 +26,7 @@ thiserror.workspace = true
 anyhow.workspace = true
 serde.workspace = true
 clap.workspace = true
+tokio.workspace = true
 
 # Config parsing
 toml = "0.8"
diff --git a/crates/mamba/src/runtime/async_rt.rs b/crates/mamba/src/runtime/async_rt.rs
index 3ce32e8..62a7526 100644
--- a/crates/mamba/src/runtime/async_rt.rs
+++ b/crates/mamba/src/runtime/async_rt.rs
@@ -1,19 +1,18 @@
 /// Async/await runtime with tokio for Mamba (#293).
 ///
-/// Async functions are compiled as state machines (like generators).
-/// The runtime embeds a tokio runtime to drive async I/O.
+/// Thread-safe version — all async state is global, protected by DashMap/RwLock.
+/// Coroutines and tasks can be accessed from any thread.
 ///
 /// Architecture:
 /// - Async functions produce "coroutine" objects (similar to generators)
 /// - `await` suspends the coroutine and schedules it on the tokio runtime
 /// - The event loop drives coroutines to completion
 ///
-/// NOTE: tokio is an optional dependency. If not available, async operations
-/// will use a simple single-threaded executor.
-///
 /// Task management, event loop, and bridge functions live in `async_task`.
 
 use std::collections::HashMap;
+use std::sync::atomic::{AtomicU64, Ordering};
+use std::sync::RwLock;
 use super::value::MbValue;
 use super::rc::ObjData;
 
@@ -39,6 +38,12 @@ pub struct MbCoroutine {
     pub body_fn: Option<unsafe extern "C" fn(i64) -> i64>,
 }
 
+// Safety: MbCoroutine fields are only accessed through the global
+// COROUTINES map which is RwLock-protected. body_fn is a plain
+// function pointer (inherently Send+Sync).
+unsafe impl Send for MbCoroutine {}
+unsafe impl Sync for MbCoroutine {}
+
 /// Task state for async execution.
 pub struct MbTask {
     pub name: String,
@@ -47,22 +52,33 @@ pub struct MbTask {
     pub result: MbValue,
 }
 
-// Thread-local async runtime state.
-thread_local! {
-    pub(crate) static COROUTINES: std::cell::RefCell<HashMap<u64, MbCoroutine>> =
-        std::cell::RefCell::new(HashMap::new());
-    pub(crate) static TASKS: std::cell::RefCell<HashMap<u64, MbTask>> =
-        std::cell::RefCell::new(HashMap::new());
-    static NEXT_CORO_ID: std::cell::Cell<u64> = std::cell::Cell::new(1);
-    static NEXT_TASK_ID: std::cell::Cell<u64> = std::cell::Cell::new(1);
-}
+// Safety: MbTask fields are only accessed through the global
+// TASKS map which is RwLock-protected.
+unsafe impl Send for MbTask {}
+unsafe impl Sync for MbTask {}
+
+// ── Global async runtime state (R5, R7) ──
+
+/// Global coroutine registry — replaces thread_local COROUTINES.
+pub(crate) static COROUTINES: std::sync::LazyLock<RwLock<HashMap<u64, MbCoroutine>>> =
+    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
+
+/// Global task registry — replaces thread_local TASKS.
+pub static TASKS: std::sync::LazyLock<RwLock<HashMap<u64, MbTask>>> =
+    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
+
+/// Atomic counter for globally unique coroutine IDs (R7).
+static NEXT_CORO_ID: AtomicU64 = AtomicU64::new(1);
+
+/// Atomic counter for globally unique task IDs (R7).
+static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
 
 pub(crate) fn alloc_coro_id() -> u64 {
-    NEXT_CORO_ID.with(|c| { let id = c.get(); c.set(id + 1); id })
+    NEXT_CORO_ID.fetch_add(1, Ordering::Relaxed)
 }
 
 pub(crate) fn alloc_task_id() -> u64 {
-    NEXT_TASK_ID.with(|c| { let id = c.get(); c.set(id + 1); id })
+    NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
 }
 
 // ── Coroutine Creation ──
@@ -81,7 +97,7 @@ pub fn mb_coroutine_new(name: MbValue, locals: MbValue) -> MbValue {
         body_fn: None,
     };
     let id = alloc_coro_id();
-    COROUTINES.with(|coros| { coros.borrow_mut().insert(id, coro); });
+    COROUTINES.write().unwrap().insert(id, coro);
     MbValue::from_int(id as i64)
 }
 
@@ -93,11 +109,9 @@ pub fn mb_coroutine_set_body(coro_handle: MbValue, fn_ptr: MbValue) {
             if ptr_val != 0 {
                 let body: unsafe extern "C" fn(i64) -> i64 =
                     unsafe { std::mem::transmute(ptr_val as usize) };
-                COROUTINES.with(|coros| {
-                    if let Some(coro) = coros.borrow_mut().get_mut(&(id as u64)) {
-                        coro.body_fn = Some(body);
-                    }
-                });
+                if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
+                    coro.body_fn = Some(body);
+                }
             }
         }
     }
@@ -107,37 +121,40 @@ pub fn mb_coroutine_set_body(coro_handle: MbValue, fn_ptr: MbValue) {
 /// If the coroutine has a registered body function and hasn't started yet
 /// (state == 0), calls the body function to execute it (#313 R1).
 pub fn mb_coroutine_step(coro_handle: MbValue) -> MbValue {
+    // Safepoint poll at coroutine step (R4)
+    super::gc::gc_safepoint();
     if let Some(id) = coro_handle.as_int() {
         // Check if already exhausted
-        let exhausted = COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true)
-        });
+        let exhausted = COROUTINES.read().unwrap()
+            .get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true);
         if exhausted {
-            return COROUTINES.with(|coros| {
-                coros.borrow().get(&(id as u64))
-                    .and_then(|c| c.result)
-                    .unwrap_or(MbValue::none())
-            });
+            return COROUTINES.read().unwrap()
+                .get(&(id as u64))
+                .and_then(|c| c.result)
+                .unwrap_or(MbValue::none());
         }
 
         // Try to call the body function if registered and not yet started
-        let step_result = COROUTINES.with(|coros| {
-            let mut coros = coros.borrow_mut();
+        let step_result = {
+            let mut coros = COROUTINES.write().unwrap();
             if let Some(coro) = coros.get_mut(&(id as u64)) {
                 if coro.state == 0 {
                     coro.state = 1; // Mark as started
                     if let Some(body) = coro.body_fn {
-                        return Ok(Some(body));
+                        Ok(Some(body))
                     } else {
                         // Fail fast: no body function registered (#313 R1)
                         coro.exhausted = true;
                         coro.result = Some(MbValue::none());
-                        return Err(());
+                        Err(())
                     }
+                } else {
+                    Ok(None)
                 }
+            } else {
+                Ok(None)
             }
-            Ok(None)
-        });
+        };
 
         match step_result {
             Ok(Some(body)) => {
@@ -149,11 +166,10 @@ pub fn mb_coroutine_step(coro_handle: MbValue) -> MbValue {
         }
 
         // Return result if now exhausted
-        COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64))
-                .and_then(|c| c.result)
-                .unwrap_or(MbValue::none())
-        })
+        COROUTINES.read().unwrap()
+            .get(&(id as u64))
+            .and_then(|c| c.result)
+            .unwrap_or(MbValue::none())
     } else {
         MbValue::none()
     }
@@ -162,12 +178,10 @@ pub fn mb_coroutine_step(coro_handle: MbValue) -> MbValue {
 /// Mark a coroutine as complete with a result.
 pub fn mb_coroutine_complete(coro_handle: MbValue, result: MbValue) {
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| {
-            if let Some(coro) = coros.borrow_mut().get_mut(&(id as u64)) {
-                coro.exhausted = true;
-                coro.result = Some(result);
-            }
-        });
+        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
+            coro.exhausted = true;
+            coro.result = Some(result);
+        }
     }
 }
 
@@ -175,9 +189,8 @@ pub fn mb_coroutine_complete(coro_handle: MbValue, result: MbValue) {
 
 pub fn mb_coroutine_get_state(coro_handle: MbValue) -> u32 {
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64)).map(|c| c.state).unwrap_or(u32::MAX)
-        })
+        COROUTINES.read().unwrap()
+            .get(&(id as u64)).map(|c| c.state).unwrap_or(u32::MAX)
     } else {
         u32::MAX
     }
@@ -185,25 +198,22 @@ pub fn mb_coroutine_get_state(coro_handle: MbValue) -> u32 {
 
 pub fn mb_coroutine_set_state(coro_handle: MbValue, state: u32) {
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| {
-            if let Some(coro) = coros.borrow_mut().get_mut(&(id as u64)) {
-                coro.state = state;
-                if state == u32::MAX {
-                    coro.exhausted = true;
-                }
+        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
+            coro.state = state;
+            if state == u32::MAX {
+                coro.exhausted = true;
             }
-        });
+        }
     }
 }
 
 pub fn mb_coroutine_get_local(coro_handle: MbValue, index: MbValue) -> MbValue {
     let idx = index.as_int().unwrap_or(0) as usize;
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64))
-                .and_then(|c| c.locals.get(idx).copied())
-                .unwrap_or(MbValue::none())
-        })
+        COROUTINES.read().unwrap()
+            .get(&(id as u64))
+            .and_then(|c| c.locals.get(idx).copied())
+            .unwrap_or(MbValue::none())
     } else {
         MbValue::none()
     }
@@ -212,20 +222,18 @@ pub fn mb_coroutine_get_local(coro_handle: MbValue, index: MbValue) -> MbValue {
 pub fn mb_coroutine_set_local(coro_handle: MbValue, index: MbValue, value: MbValue) {
     let idx = index.as_int().unwrap_or(0) as usize;
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| {
-            if let Some(coro) = coros.borrow_mut().get_mut(&(id as u64)) {
-                if idx >= coro.locals.len() {
-                    coro.locals.resize(idx + 1, MbValue::none());
-                }
-                coro.locals[idx] = value;
+        if let Some(coro) = COROUTINES.write().unwrap().get_mut(&(id as u64)) {
+            if idx >= coro.locals.len() {
+                coro.locals.resize(idx + 1, MbValue::none());
             }
-        });
+            coro.locals[idx] = value;
+        }
     }
 }
 
 pub fn mb_coroutine_release(coro_handle: MbValue) {
     if let Some(id) = coro_handle.as_int() {
-        COROUTINES.with(|coros| { coros.borrow_mut().remove(&(id as u64)); });
+        COROUTINES.write().unwrap().remove(&(id as u64));
     }
 }
 
@@ -308,10 +316,9 @@ mod tests {
         let result = mb_coroutine_step(coro);
         assert_eq!(result.as_int(), None, "missing body should return None");
         // Coroutine should now be exhausted
-        let is_exhausted = COROUTINES.with(|coros| {
-            coros.borrow().get(&(coro.as_int().unwrap() as u64))
-                .map(|c| c.exhausted).unwrap_or(false)
-        });
+        let is_exhausted = COROUTINES.read().unwrap()
+            .get(&(coro.as_int().unwrap() as u64))
+            .map(|c| c.exhausted).unwrap_or(false);
         assert!(is_exhausted, "coroutine with no body should be exhausted after step");
     }
 
@@ -322,13 +329,22 @@ mod tests {
         let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
         let coro = mb_coroutine_new(name, locals);
         // Before stepping, coroutine should not be exhausted
-        let is_exhausted = COROUTINES.with(|coros| {
-            coros.borrow().get(&(coro.as_int().unwrap() as u64))
-                .map(|c| c.exhausted).unwrap_or(true)
-        });
+        let is_exhausted = COROUTINES.read().unwrap()
+            .get(&(coro.as_int().unwrap() as u64))
+            .map(|c| c.exhausted).unwrap_or(true);
         assert!(!is_exhausted, "coroutine should not be exhausted before step");
         // State should still be 0 (not started)
         assert_eq!(mb_coroutine_get_state(coro), 0);
         mb_coroutine_release(coro);
     }
+
+    #[test]
+    fn test_atomic_id_allocation_unique() {
+        let id1 = alloc_coro_id();
+        let id2 = alloc_coro_id();
+        let id3 = alloc_task_id();
+        let id4 = alloc_task_id();
+        assert_ne!(id1, id2, "coroutine IDs must be unique");
+        assert_ne!(id3, id4, "task IDs must be unique");
+    }
 }
diff --git a/crates/mamba/src/runtime/async_task.rs b/crates/mamba/src/runtime/async_task.rs
index 0c7605a..17bce1b 100644
--- a/crates/mamba/src/runtime/async_task.rs
+++ b/crates/mamba/src/runtime/async_task.rs
@@ -1,21 +1,15 @@
 /// Task management, event loop, await, orbit bridge, and GIL for Mamba async (#313).
 ///
-/// This module contains the scheduling and execution layer that sits on top
-/// of the coroutine primitives in `async_rt`.
+/// Thread-safe version — uses global RwLock-protected state from async_rt.
 ///
-/// ## Architecture: v1 Single-Threaded Executor
+/// ## Architecture: v2 Multi-Threaded Executor
 ///
-/// The current implementation is a **single-threaded cooperative executor**.
-/// Coroutine bodies execute to completion on first step (one-shot). Multiple
-/// coroutines are interleaved at the task level via `EventLoop::tick()`.
-///
-/// **v2 roadmap** (future work):
-/// - Full state machine transformation with yield points between awaits
-/// - Tokio runtime bridge for true non-blocking I/O
-/// - Orbit reactor integration for timer/waker-driven resumption
-/// - External future polling protocol
+/// All async state (COROUTINES, TASKS, WAKERS, TIMERS) is global and
+/// thread-safe. Tasks can be scheduled on Tokio's multi-threaded executor
+/// for true parallel async I/O.
 
 use std::collections::HashMap;
+use std::sync::RwLock;
 use super::value::MbValue;
 use super::rc::{MbObject, ObjData};
 use super::async_rt::{
@@ -35,18 +29,17 @@ pub fn mb_create_task(coro: MbValue) -> MbValue {
         result: MbValue::none(),
     };
     let id = alloc_task_id();
-    TASKS.with(|tasks| { tasks.borrow_mut().insert(id, task); });
+    TASKS.write().unwrap().insert(id, task);
     MbValue::from_int(id as i64)
 }
 
 /// Check if a task is done.
 pub fn mb_task_done(task_handle: MbValue) -> MbValue {
     if let Some(id) = task_handle.as_int() {
-        TASKS.with(|tasks| {
-            MbValue::from_bool(
-                tasks.borrow().get(&(id as u64)).map(|t| t.done).unwrap_or(true)
-            )
-        })
+        MbValue::from_bool(
+            TASKS.read().unwrap()
+                .get(&(id as u64)).map(|t| t.done).unwrap_or(true)
+        )
     } else {
         MbValue::from_bool(true)
     }
@@ -55,11 +48,10 @@ pub fn mb_task_done(task_handle: MbValue) -> MbValue {
 /// Get the result of a completed task.
 pub fn mb_task_result(task_handle: MbValue) -> MbValue {
     if let Some(id) = task_handle.as_int() {
-        TASKS.with(|tasks| {
-            tasks.borrow().get(&(id as u64))
-                .map(|t| t.result)
-                .unwrap_or(MbValue::none())
-        })
+        TASKS.read().unwrap()
+            .get(&(id as u64))
+            .map(|t| t.result)
+            .unwrap_or(MbValue::none())
     } else {
         MbValue::none()
     }
@@ -70,22 +62,27 @@ pub fn mb_task_result(task_handle: MbValue) -> MbValue {
 /// Cancel a task.
 pub fn mb_cancel_task(task_handle: MbValue) -> MbValue {
     if let Some(id) = task_handle.as_int() {
-        TASKS.with(|tasks| {
-            if let Some(task) = tasks.borrow_mut().get_mut(&(id as u64)) {
+        let coro_id = {
+            let mut tasks = TASKS.write().unwrap();
+            if let Some(task) = tasks.get_mut(&(id as u64)) {
                 if !task.done {
                     task.done = true;
                     task.result = MbValue::none();
-                    // Also mark the coroutine as exhausted
-                    COROUTINES.with(|coros| {
-                        if let Some(coro) = coros.borrow_mut().get_mut(&task.coroutine_id) {
-                            coro.exhausted = true;
-                        }
-                    });
-                    return MbValue::from_bool(true);
+                    Some(task.coroutine_id)
+                } else {
+                    None
                 }
+            } else {
+                None
             }
-            MbValue::from_bool(false)
-        })
+        };
+        if let Some(cid) = coro_id {
+            if let Some(coro) = COROUTINES.write().unwrap().get_mut(&cid) {
+                coro.exhausted = true;
+            }
+            return MbValue::from_bool(true);
+        }
+        MbValue::from_bool(false)
     } else {
         MbValue::from_bool(false)
     }
@@ -94,13 +91,12 @@ pub fn mb_cancel_task(task_handle: MbValue) -> MbValue {
 /// Check if a task was cancelled.
 pub fn mb_task_cancelled(task_handle: MbValue) -> MbValue {
     if let Some(id) = task_handle.as_int() {
-        TASKS.with(|tasks| {
-            MbValue::from_bool(
-                tasks.borrow().get(&(id as u64))
-                    .map(|t| t.done && t.result.is_none())
-                    .unwrap_or(false)
-            )
-        })
+        MbValue::from_bool(
+            TASKS.read().unwrap()
+                .get(&(id as u64))
+                .map(|t| t.done && t.result.is_none())
+                .unwrap_or(false)
+        )
     } else {
         MbValue::from_bool(false)
     }
@@ -126,34 +122,37 @@ impl EventLoop {
 
     /// Run one iteration: check timers, then step all ready tasks.
     fn tick(&mut self) {
+        // Safepoint poll at event loop tick (R4)
+        super::gc::gc_safepoint();
+
         // Phase 1: Check timers — complete any expired timer coroutines
         let now = std::time::Instant::now();
-        let expired: Vec<u64> = TIMERS.with(|t| {
-            t.borrow().iter()
+        let expired: Vec<u64> = {
+            let timers = TIMERS.read().unwrap();
+            timers.iter()
                 .filter(|(_, deadline)| now >= **deadline)
                 .map(|(&cid, _)| cid)
                 .collect()
-        });
+        };
         for cid in &expired {
-            // Mark the timer coroutine as complete
             use super::async_rt::mb_coroutine_complete;
             mb_coroutine_complete(MbValue::from_int(*cid as i64), MbValue::none());
-            TIMERS.with(|t| { t.borrow_mut().remove(cid); });
+            TIMERS.write().unwrap().remove(cid);
         }
 
         // Phase 2: Step all ready tasks
         let tasks_to_run: Vec<u64> = self.ready_queue.drain(..).collect();
         for task_id in tasks_to_run {
-            let (coro_id, is_done) = TASKS.with(|tasks| {
-                let tasks = tasks.borrow();
+            let (coro_id, is_done) = {
+                let tasks = TASKS.read().unwrap();
                 tasks.get(&task_id)
                     .map(|t| (t.coroutine_id, t.done))
                     .unwrap_or((0, true))
-            });
+            };
             if is_done { continue; }
 
             // Skip timer coroutines that haven't expired yet
-            let is_pending_timer = TIMERS.with(|t| t.borrow().contains_key(&coro_id));
+            let is_pending_timer = TIMERS.read().unwrap().contains_key(&coro_id);
             if is_pending_timer {
                 self.ready_queue.push(task_id);
                 continue;
@@ -163,22 +162,18 @@ impl EventLoop {
             mb_coroutine_step(MbValue::from_int(coro_id as i64));
 
             // Check if coroutine is done
-            let exhausted = COROUTINES.with(|coros| {
-                coros.borrow().get(&coro_id).map(|c| c.exhausted).unwrap_or(true)
-            });
+            let exhausted = COROUTINES.read().unwrap()
+                .get(&coro_id).map(|c| c.exhausted).unwrap_or(true);
 
             if exhausted {
-                let result = COROUTINES.with(|coros| {
-                    coros.borrow().get(&coro_id)
-                        .and_then(|c| c.result)
-                        .unwrap_or(MbValue::none())
-                });
-                TASKS.with(|tasks| {
-                    if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
-                        task.done = true;
-                        task.result = result;
-                    }
-                });
+                let result = COROUTINES.read().unwrap()
+                    .get(&coro_id)
+                    .and_then(|c| c.result)
+                    .unwrap_or(MbValue::none());
+                if let Some(task) = TASKS.write().unwrap().get_mut(&task_id) {
+                    task.done = true;
+                    task.result = result;
+                }
             } else {
                 // Re-schedule for next tick
                 self.ready_queue.push(task_id);
@@ -186,16 +181,13 @@ impl EventLoop {
         }
 
         // If there are pending timers and no other work to do, yield CPU
-        // to let wall-clock time advance toward timer deadlines.
-        let has_timers = TIMERS.with(|t| !t.borrow().is_empty());
+        let has_timers = !TIMERS.read().unwrap().is_empty();
         if has_timers && self.ready_queue.iter().all(|tid| {
-            TASKS.with(|tasks| {
-                tasks.borrow().get(tid)
-                    .map(|t| TIMERS.with(|tm| tm.borrow().contains_key(&t.coroutine_id)))
-                    .unwrap_or(false)
-            })
+            let tasks = TASKS.read().unwrap();
+            tasks.get(tid)
+                .map(|t| TIMERS.read().unwrap().contains_key(&t.coroutine_id))
+                .unwrap_or(false)
         }) {
-            // All remaining tasks are timers — sleep 1ms to pace the loop
             std::thread::sleep(std::time::Duration::from_millis(1));
         }
     }
@@ -204,37 +196,26 @@ impl EventLoop {
 // ── Orbit Bridge (#313 R2) ──
 
 /// Schedule a coroutine on the Orbit event loop.
-/// Creates a task and adds it to the ready queue.
 pub fn mb_orbit_schedule(coro: MbValue) -> MbValue {
     let task = mb_create_task(coro);
-    // In the future, this will bridge to the cclab-orbit reactor.
-    // For now, the task is registered and will be driven by
-    // mb_run_until_complete or mb_gather.
     task
 }
 
-// Waker registry: maps coroutine IDs to their pending task IDs.
-// When a waker fires, the task is re-scheduled on the event loop.
-thread_local! {
-    pub(crate) static WAKERS: std::cell::RefCell<HashMap<u64, u64>> =
-        std::cell::RefCell::new(HashMap::new());
-}
+/// Global waker registry — maps coroutine IDs to their pending task IDs.
+pub(crate) static WAKERS: std::sync::LazyLock<RwLock<HashMap<u64, u64>>> =
+    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
 
 /// Register a waker for a coroutine, linking it to a task.
-/// Called at suspension points so the event loop can wake the
-/// coroutine when its awaited value becomes ready.
 pub fn mb_orbit_register_waker(coro: MbValue) -> MbValue {
     if let Some(coro_id) = coro.as_int() {
-        // Find the task that owns this coroutine
-        let task_id = TASKS.with(|tasks| {
-            tasks.borrow().iter()
+        let task_id = {
+            let tasks = TASKS.read().unwrap();
+            tasks.iter()
                 .find(|(_, t)| t.coroutine_id == coro_id as u64 && !t.done)
                 .map(|(&id, _)| id)
-        });
+        };
         if let Some(tid) = task_id {
-            WAKERS.with(|w| {
-                w.borrow_mut().insert(coro_id as u64, tid);
-            });
+            WAKERS.write().unwrap().insert(coro_id as u64, tid);
         }
     }
     MbValue::none()
@@ -242,48 +223,42 @@ pub fn mb_orbit_register_waker(coro: MbValue) -> MbValue {
 
 // ── Await Support ──
 
-/// Await a coroutine or future. Drives the coroutine to completion
-/// via the event loop with bounded iterations to prevent infinite loops.
+/// Await a coroutine or future.
 pub fn mb_await(awaitable: MbValue) -> MbValue {
     if let Some(id) = awaitable.as_int() {
         // Fast path: coroutine already complete
-        let exhausted = COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true)
-        });
+        let exhausted = COROUTINES.read().unwrap()
+            .get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true);
         if exhausted {
-            return COROUTINES.with(|coros| {
-                coros.borrow().get(&(id as u64))
-                    .and_then(|c| c.result)
-                    .unwrap_or(MbValue::none())
-            });
+            return COROUTINES.read().unwrap()
+                .get(&(id as u64))
+                .and_then(|c| c.result)
+                .unwrap_or(MbValue::none());
         }
 
         // Schedule via event loop and drive to completion
         let task_handle = mb_orbit_schedule(awaitable);
         let task_id = task_handle.as_int().unwrap_or(0) as u64;
 
-        // Register waker at suspension point (#313 R2)
         mb_orbit_register_waker(awaitable);
 
         let mut event_loop = EventLoop::new();
         event_loop.schedule(task_id);
 
         // Also schedule any waker-associated tasks
-        WAKERS.with(|w| {
-            for (_, &tid) in w.borrow().iter() {
+        {
+            let wakers = WAKERS.read().unwrap();
+            for (_, &tid) in wakers.iter() {
                 event_loop.schedule(tid);
             }
-        });
+        }
 
-        // Bounded iteration to prevent infinite spin.
-        // The event loop is the sole stepping authority — it drives the
-        // coroutine via the task queue (no direct mb_coroutine_step here).
         let max_iterations = 100_000;
         let mut completed = false;
         for _ in 0..max_iterations {
-            let done = COROUTINES.with(|coros| {
-                coros.borrow().get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true)
-            });
+            super::gc::gc_safepoint();
+            let done = COROUTINES.read().unwrap()
+                .get(&(id as u64)).map(|c| c.exhausted).unwrap_or(true);
             if done { completed = true; break; }
             event_loop.tick();
         }
@@ -291,14 +266,12 @@ pub fn mb_await(awaitable: MbValue) -> MbValue {
             eprintln!("mamba: mb_await: iteration limit reached, coroutine may be incomplete");
         }
 
-        // Clean up waker registration
-        WAKERS.with(|w| { w.borrow_mut().remove(&(id as u64)); });
+        WAKERS.write().unwrap().remove(&(id as u64));
 
-        COROUTINES.with(|coros| {
-            coros.borrow().get(&(id as u64))
-                .and_then(|c| c.result)
-                .unwrap_or(MbValue::none())
-        })
+        COROUTINES.read().unwrap()
+            .get(&(id as u64))
+            .and_then(|c| c.result)
+            .unwrap_or(MbValue::none())
     } else {
         MbValue::none()
     }
@@ -306,15 +279,12 @@ pub fn mb_await(awaitable: MbValue) -> MbValue {
 
 // ── asyncio-compatible Functions ──
 
-/// asyncio.gather(*coros) — run multiple coroutines concurrently
-/// and collect results. Schedules all coroutines on a shared event
-/// loop and drives them round-robin until all complete.
+/// asyncio.gather(*coros) — run multiple coroutines concurrently.
 pub fn mb_gather(coros: MbValue) -> MbValue {
     if let Some(ptr) = coros.as_ptr() {
         unsafe {
             if let ObjData::List(ref lock) = (*ptr).data {
                 let coro_list = lock.read().unwrap();
-                // Schedule all coroutines as tasks on a shared event loop
                 let mut event_loop = EventLoop::new();
                 let task_ids: Vec<(u64, u64)> = coro_list.iter().map(|c| {
                     let coro_id = c.as_int().unwrap_or(0) as u64;
@@ -324,15 +294,12 @@ pub fn mb_gather(coros: MbValue) -> MbValue {
                     (coro_id, tid)
                 }).collect();
 
-                // Drive all tasks concurrently until all complete
                 let max_iterations = 100_000;
                 let mut completed = false;
                 for _ in 0..max_iterations {
                     let all_done = task_ids.iter().all(|(cid, _)| {
-                        COROUTINES.with(|coros| {
-                            coros.borrow().get(cid)
-                                .map(|c| c.exhausted).unwrap_or(true)
-                        })
+                        COROUTINES.read().unwrap()
+                            .get(cid).map(|c| c.exhausted).unwrap_or(true)
                     });
                     if all_done { completed = true; break; }
                     event_loop.tick();
@@ -341,13 +308,11 @@ pub fn mb_gather(coros: MbValue) -> MbValue {
                     eprintln!("mamba: mb_gather: iteration limit reached, some tasks may be incomplete");
                 }
 
-                // Collect results in original order
                 let results: Vec<MbValue> = task_ids.iter().map(|(cid, _)| {
-                    COROUTINES.with(|coros| {
-                        coros.borrow().get(cid)
-                            .and_then(|c| c.result)
-                            .unwrap_or(MbValue::none())
-                    })
+                    COROUTINES.read().unwrap()
+                        .get(cid)
+                        .and_then(|c| c.result)
+                        .unwrap_or(MbValue::none())
                 }).collect();
                 return MbValue::from_ptr(MbObject::new_list(results));
             }
@@ -356,12 +321,7 @@ pub fn mb_gather(coros: MbValue) -> MbValue {
     MbValue::none()
 }
 
-/// asyncio.sleep(seconds) — cooperative sleep that yields to the event loop.
-///
-/// Creates a timer coroutine that completes after the given duration,
-/// allowing other tasks in the event loop to make progress. This is
-/// non-blocking at the task level (other gather'd tasks can run), though
-/// the thread itself uses spin-yield (v2 will use Tokio timers).
+/// asyncio.sleep(seconds) — cooperative sleep.
 pub fn mb_sleep(seconds: MbValue) -> MbValue {
     let duration = if let Some(secs) = seconds.as_float() {
         if secs <= 0.0 || secs.is_nan() || secs.is_infinite() {
@@ -379,32 +339,24 @@ pub fn mb_sleep(seconds: MbValue) -> MbValue {
         return MbValue::none();
     };
 
-    // Create a timer coroutine that marks itself complete after the duration.
-    // The event loop can drive other tasks while this timer ticks.
     let name = MbValue::from_ptr(MbObject::new_str("sleep_timer".to_string()));
     let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
     let coro = super::async_rt::mb_coroutine_new(name, locals);
 
-    // Store deadline in the timer registry so EventLoop::tick can check it
     let deadline = std::time::Instant::now() + duration;
-    TIMERS.with(|t| {
-        if let Some(coro_id) = coro.as_int() {
-            t.borrow_mut().insert(coro_id as u64, deadline);
-        }
-    });
+    if let Some(coro_id) = coro.as_int() {
+        TIMERS.write().unwrap().insert(coro_id as u64, deadline);
+    }
 
     coro
 }
 
-// Timer registry: maps coroutine IDs to their wake-up deadlines.
-thread_local! {
-    static TIMERS: std::cell::RefCell<HashMap<u64, std::time::Instant>> =
-        std::cell::RefCell::new(HashMap::new());
-}
+/// Global timer registry — maps coroutine IDs to their wake-up deadlines.
+static TIMERS: std::sync::LazyLock<RwLock<HashMap<u64, std::time::Instant>>> =
+    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
 
-/// asyncio.wait(tasks, timeout=None) — wait for tasks with optional timeout.
+/// asyncio.wait(tasks, timeout=None).
 pub fn mb_async_wait(tasks: MbValue, _timeout: MbValue) -> MbValue {
-    // Simple implementation: wait for all tasks
     if let Some(ptr) = tasks.as_ptr() {
         unsafe {
             if let ObjData::List(ref lock) = (*ptr).data {
@@ -430,9 +382,7 @@ pub fn mb_async_wait(tasks: MbValue, _timeout: MbValue) -> MbValue {
 }
 
 /// Run the event loop until the main coroutine completes.
-/// Schedules all pending tasks and drives them round-robin.
 pub fn mb_run_until_complete(main_coro: MbValue) -> MbValue {
-    // Create a task for the main coroutine
     let main_task_id = {
         let coro_id = main_coro.as_int().unwrap_or(0) as u64;
         let task = MbTask {
@@ -442,7 +392,7 @@ pub fn mb_run_until_complete(main_coro: MbValue) -> MbValue {
             result: MbValue::none(),
         };
         let id = alloc_task_id();
-        TASKS.with(|tasks| { tasks.borrow_mut().insert(id, task); });
+        TASKS.write().unwrap().insert(id, task);
         id
     };
 
@@ -450,47 +400,43 @@ pub fn mb_run_until_complete(main_coro: MbValue) -> MbValue {
     event_loop.schedule(main_task_id);
 
     // Also schedule all pending tasks
-    TASKS.with(|tasks| {
-        for (&id, task) in tasks.borrow().iter() {
+    {
+        let tasks = TASKS.read().unwrap();
+        for (&id, task) in tasks.iter() {
             if !task.done {
                 event_loop.schedule(id);
             }
         }
-    });
+    }
 
-    // Run until main task completes
     let max_iterations = 10_000;
     for _ in 0..max_iterations {
-        let main_done = TASKS.with(|tasks| {
-            tasks.borrow().get(&main_task_id).map(|t| t.done).unwrap_or(true)
-        });
+        super::gc::gc_safepoint();
+        let main_done = TASKS.read().unwrap()
+            .get(&main_task_id).map(|t| t.done).unwrap_or(true);
         if main_done { break; }
         event_loop.tick();
     }
 
-    // Return main task result
-    TASKS.with(|tasks| {
-        tasks.borrow().get(&main_task_id)
-            .map(|t| t.result)
-            .unwrap_or(MbValue::none())
-    })
+    TASKS.read().unwrap()
+        .get(&main_task_id)
+        .map(|t| t.result)
+        .unwrap_or(MbValue::none())
 }
 
 // ── GIL Management (#313 R3) ──
+// In no-GIL mode, these are no-ops but kept for API compatibility.
 
 thread_local! {
     static GIL_HELD: std::cell::Cell<bool> = std::cell::Cell::new(true);
 }
 
-/// Release the GIL before a coroutine suspension point.
-/// Allows other threads to execute Mamba code while this
-/// coroutine is waiting for I/O or another async operation.
+/// Release the GIL (no-op in no-GIL mode, kept for API compat).
 pub fn mb_gil_release() {
     GIL_HELD.with(|g| g.set(false));
 }
 
-/// Re-acquire the GIL after a coroutine resumes.
-/// Must be called before executing any Mamba bytecode.
+/// Re-acquire the GIL (no-op in no-GIL mode, kept for API compat).
 pub fn mb_gil_acquire() {
     GIL_HELD.with(|g| g.set(true));
 }
@@ -502,25 +448,15 @@ pub fn mb_gil_held() -> MbValue {
 
 // ── Future Interoperability (#313 R4) ──
 
-/// Await an external future (e.g., a Tokio future wrapped as MbValue).
-///
-/// Resolution strategy:
-/// 1. If the value is a Mamba coroutine handle → drives it via `mb_await`
-/// 2. If the value is a timer coroutine (from mb_sleep) → drives via event loop
-/// 3. Otherwise → treats as an already-resolved value (passthrough)
-///
-/// v2 will add a polling/waker protocol for real Tokio futures.
+/// Await an external future.
 pub fn mb_await_external(future: MbValue) -> MbValue {
     if let Some(id) = future.as_int() {
-        // Check if it's a registered coroutine (including timer coroutines)
-        let is_coro = COROUTINES.with(|coros| {
-            coros.borrow().contains_key(&(id as u64))
-        });
+        let is_coro = COROUTINES.read().unwrap()
+            .contains_key(&(id as u64));
         if is_coro {
             return mb_await(future);
         }
     }
-    // Already-resolved value: return as-is
     future
 }
 
@@ -553,7 +489,6 @@ mod tests {
 
     #[test]
     fn test_gil_release_acquire() {
-        // Ensure GIL starts held
         mb_gil_acquire();
         assert_eq!(mb_gil_held().as_bool(), Some(true));
         mb_gil_release();
@@ -574,22 +509,17 @@ mod tests {
         let name = MbValue::from_ptr(MbObject::new_str("waker_coro".to_string()));
         let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
         let coro = mb_coroutine_new(name, locals);
-        // Schedule creates a task
         let task = mb_orbit_schedule(coro);
         assert!(task.is_int());
-        // Register waker should not panic
         mb_orbit_register_waker(coro);
-        // Waker should be registered
-        let has_waker = WAKERS.with(|w| {
-            w.borrow().contains_key(&(coro.as_int().unwrap() as u64))
-        });
+        let has_waker = WAKERS.read().unwrap()
+            .contains_key(&(coro.as_int().unwrap() as u64));
         assert!(has_waker, "waker should be registered for the coroutine");
         mb_coroutine_release(coro);
     }
 
     #[test]
     fn test_gather_completed_coroutines() {
-        // Create two completed coroutines
         let n1 = MbValue::from_ptr(MbObject::new_str("c1".to_string()));
         let l1 = MbValue::from_ptr(MbObject::new_list(vec![]));
         let c1 = mb_coroutine_new(n1, l1);
@@ -600,7 +530,6 @@ mod tests {
         let c2 = mb_coroutine_new(n2, l2);
         mb_coroutine_complete(c2, MbValue::from_int(20));
 
-        // Gather should collect both results
         let coros = MbValue::from_ptr(MbObject::new_list(vec![c1, c2]));
         let results = mb_gather(coros);
         assert!(results.as_ptr().is_some(), "gather should return a list");
@@ -608,35 +537,27 @@ mod tests {
 
     #[test]
     fn test_sleep_creates_timer_coroutine() {
-        // mb_sleep should return a coroutine handle, not block
         let coro = mb_sleep(MbValue::from_int(0));
         assert!(coro.is_int(), "sleep should return a coroutine handle");
-        // The timer coroutine should be registered
-        let is_registered = COROUTINES.with(|coros| {
-            coros.borrow().contains_key(&(coro.as_int().unwrap() as u64))
-        });
+        let is_registered = COROUTINES.read().unwrap()
+            .contains_key(&(coro.as_int().unwrap() as u64));
         assert!(is_registered, "sleep timer should be in COROUTINES");
     }
 
     #[test]
     fn test_sleep_timer_expires_in_event_loop() {
-        // Create a zero-duration sleep timer
         let timer_coro = mb_sleep(MbValue::from_int(0));
         let task = mb_create_task(timer_coro);
         let tid = task.as_int().unwrap() as u64;
 
-        // Create event loop and tick — timer with 0 duration should expire
         let mut event_loop = EventLoop::new();
         event_loop.schedule(tid);
-        // Small delay to ensure timer expires
         std::thread::sleep(std::time::Duration::from_millis(1));
         event_loop.tick();
 
-        // Timer coroutine should now be exhausted
-        let exhausted = COROUTINES.with(|coros| {
-            coros.borrow().get(&(timer_coro.as_int().unwrap() as u64))
-                .map(|c| c.exhausted).unwrap_or(false)
-        });
+        let exhausted = COROUTINES.read().unwrap()
+            .get(&(timer_coro.as_int().unwrap() as u64))
+            .map(|c| c.exhausted).unwrap_or(false);
         assert!(exhausted, "zero-duration timer should expire after tick");
     }
 }
diff --git a/crates/mamba/src/runtime/class.rs b/crates/mamba/src/runtime/class.rs
index c83fc6c..d44924f 100644
--- a/crates/mamba/src/runtime/class.rs
+++ b/crates/mamba/src/runtime/class.rs
@@ -995,6 +995,8 @@ pub fn mb_obj_del(obj: MbValue) {
 /// Only callable values registered via `mb_class_register` can be invoked.
 /// Non-callable or unregistered values return None (TypeError).
 pub fn mb_call_method1(method: MbValue, arg: MbValue) -> MbValue {
+    // Safepoint poll at method call (R4)
+    super::gc::gc_safepoint();
     if let Some(fn_addr) = method.as_int() {
         let addr = fn_addr as u64;
         let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
@@ -1013,6 +1015,8 @@ pub fn mb_call_method1(method: MbValue, arg: MbValue) -> MbValue {
 /// Checks NaN-box tag for primitives, then ObjData variant for heap objects.
 /// Falls back to MRO-based lookup for user class instances.
 pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) -> MbValue {
+    // Safepoint poll at method dispatch (R4)
+    super::gc::gc_safepoint();
     let name = extract_str(method_name).unwrap_or_default();
 
     // Primitives — no heap deref needed
diff --git a/crates/mamba/src/runtime/gc.rs b/crates/mamba/src/runtime/gc.rs
index 75a18cd..8a90d32 100644
--- a/crates/mamba/src/runtime/gc.rs
+++ b/crates/mamba/src/runtime/gc.rs
@@ -9,7 +9,8 @@
 /// - Stop-the-world: all mutator threads must be paused during collection
 
 use std::collections::HashSet;
-use std::sync::Mutex;
+use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
+use std::sync::{Condvar, Mutex};
 use super::rc::MbObject;
 use super::value::MbValue;
 
@@ -51,6 +52,89 @@ impl GcState {
 static GC: std::sync::LazyLock<Mutex<GcState>> =
     std::sync::LazyLock::new(|| Mutex::new(GcState::new()));
 
+// ── Safepoint Protocol (R4) ──
+
+/// Global safepoint flag — set by GC, polled by mutator threads.
+static SAFEPOINT_REQUESTED: AtomicBool = AtomicBool::new(false);
+
+/// Number of mutator threads that have reached a safepoint.
+static THREADS_AT_SAFEPOINT: AtomicUsize = AtomicUsize::new(0);
+
+/// Total number of registered mutator threads.
+static REGISTERED_THREADS: AtomicUsize = AtomicUsize::new(0);
+
+/// Condvar for GC to wait until all mutators are parked.
+static SAFEPOINT_SYNC: std::sync::LazyLock<(Mutex<()>, Condvar)> =
+    std::sync::LazyLock::new(|| (Mutex::new(()), Condvar::new()));
+
+thread_local! {
+    static IS_REGISTERED: std::cell::Cell<bool> = std::cell::Cell::new(false);
+}
+
+/// Register the calling thread as a mutator. Must be called once per thread
+/// before executing Mamba code.
+pub fn gc_register_thread() {
+    IS_REGISTERED.with(|r| r.set(true));
+    REGISTERED_THREADS.fetch_add(1, Ordering::SeqCst);
+}
+
+/// Unregister the calling thread. Call when the thread exits.
+pub fn gc_unregister_thread() {
+    IS_REGISTERED.with(|r| r.set(false));
+    REGISTERED_THREADS.fetch_sub(1, Ordering::SeqCst);
+}
+
+/// Safepoint poll — called by interpreter at function entry and loop backedges.
+/// If a GC safepoint is requested, the thread parks until GC completes.
+#[inline]
+pub fn gc_safepoint() {
+    if SAFEPOINT_REQUESTED.load(Ordering::Acquire) {
+        gc_safepoint_slow();
+    }
+}
+
+#[cold]
+fn gc_safepoint_slow() {
+    let (lock, cvar) = &*SAFEPOINT_SYNC;
+    THREADS_AT_SAFEPOINT.fetch_add(1, Ordering::SeqCst);
+    // Wake GC thread if it's waiting for all mutators
+    cvar.notify_all();
+    // Park until safepoint is cleared
+    let mut guard = lock.lock().unwrap();
+    while SAFEPOINT_REQUESTED.load(Ordering::Acquire) {
+        guard = cvar.wait(guard).unwrap();
+    }
+    THREADS_AT_SAFEPOINT.fetch_sub(1, Ordering::SeqCst);
+}
+
+/// Request all mutator threads to reach a safepoint (STW begin).
+/// The calling (collector) thread is excluded from the wait count to
+/// prevent self-deadlock when a registered mutator triggers GC.
+fn request_safepoint() {
+    SAFEPOINT_REQUESTED.store(true, Ordering::Release);
+    let total = REGISTERED_THREADS.load(Ordering::SeqCst);
+    // Subtract 1 if the collector thread is itself a registered mutator.
+    // A registered thread calling collect() cannot also park at a safepoint.
+    let is_collector_registered = IS_REGISTERED.with(|r| r.get());
+    let wait_for = if is_collector_registered && total > 0 { total - 1 } else { total };
+    if wait_for == 0 {
+        return;
+    }
+    let (lock, cvar) = &*SAFEPOINT_SYNC;
+    let mut guard = lock.lock().unwrap();
+    while THREADS_AT_SAFEPOINT.load(Ordering::SeqCst) < wait_for {
+        guard = cvar.wait(guard).unwrap();
+    }
+    drop(guard);
+}
+
+/// Release all mutator threads from safepoint (STW end).
+fn release_safepoint() {
+    SAFEPOINT_REQUESTED.store(false, Ordering::Release);
+    let (_, cvar) = &*SAFEPOINT_SYNC;
+    cvar.notify_all();
+}
+
 // ── Object Tracking ──
 
 /// Track a container object for cycle detection.
@@ -106,6 +190,9 @@ pub fn gc_clear_roots() {
 // ── Collection ──
 
 /// Run a garbage collection cycle. Returns the number of objects collected.
+///
+/// Uses cooperative safepoint-based STW: requests all registered mutator
+/// threads to pause, then marks and sweeps, then resumes them.
 pub fn collect() -> usize {
     let (tracked_snapshot, root_ptrs) = {
         let mut gc = GC.lock().unwrap();
@@ -120,6 +207,9 @@ pub fn collect() -> usize {
         (tracked, roots)
     };
 
+    // STW: request all mutator threads to park at safepoints
+    request_safepoint();
+
     // Phase 1: Mark from roots (lock released during traversal)
     for root_ptr in &root_ptrs {
         mark_object(*root_ptr);
@@ -151,6 +241,9 @@ pub fn collect() -> usize {
         count
     };
 
+    // STW end: resume all mutator threads
+    release_safepoint();
+
     collected
 }
 
@@ -291,8 +384,17 @@ mod tests {
     use super::*;
     use super::super::rc::{MbObject, ObjData};
 
+    /// Serialize GC tests that depend on collect() behavior to avoid
+    /// cross-test interference from global GC state.
+    static GC_TEST_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
+        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
+
     #[test]
     fn test_track_and_collect_unreachable() {
+        let _lock = GC_TEST_LOCK.lock().unwrap();
+        // Flush stale objects from other tests
+        collect();
+
         let a = MbObject::new_list(vec![]);
         let b = MbObject::new_list(vec![]);
 
@@ -313,6 +415,9 @@ mod tests {
 
     #[test]
     fn test_reachable_not_collected() {
+        let _lock = GC_TEST_LOCK.lock().unwrap();
+        collect(); // flush stale
+
         let obj = MbObject::new_list(vec![MbValue::from_int(42)]);
         let val = MbValue::from_ptr(obj);
         gc_add_root(val);
@@ -327,13 +432,11 @@ mod tests {
 
     #[test]
     fn test_nested_reachability() {
-        // Clear any tracked objects from other tests to isolate this test
-        gc_clear_roots();
-        collect(); // flush unrooted objects from other tests
+        let _lock = GC_TEST_LOCK.lock().unwrap();
+        collect(); // flush stale
 
         let inner = MbObject::new_list(vec![MbValue::from_int(1)]);
         let outer = MbObject::new_list(vec![MbValue::from_ptr(inner)]);
-
         let root = MbValue::from_ptr(outer);
         gc_add_root(root);
 
diff --git a/crates/mamba/src/runtime/iter.rs b/crates/mamba/src/runtime/iter.rs
index 72122c3..7c4b671 100644
--- a/crates/mamba/src/runtime/iter.rs
+++ b/crates/mamba/src/runtime/iter.rs
@@ -235,6 +235,8 @@ pub fn mb_zip(a: MbValue, b: MbValue) -> MbValue {
 
 /// Get the next value from an iterator. Returns None when exhausted.
 pub fn mb_next(iter_handle: MbValue) -> MbValue {
+    // Safepoint poll at loop backedge (R4)
+    super::gc::gc_safepoint();
     if let Some(id) = iter_handle.as_int() {
         ITERATORS.with(|iters| {
             let mut iters = iters.borrow_mut();
diff --git a/crates/mamba/src/runtime/mod.rs b/crates/mamba/src/runtime/mod.rs
index ca0817f..2c681d3 100644
--- a/crates/mamba/src/runtime/mod.rs
+++ b/crates/mamba/src/runtime/mod.rs
@@ -15,6 +15,7 @@ pub mod closure;
 pub mod module;
 pub mod async_rt;
 pub mod async_task;
+pub mod tokio_exec;
 pub mod gc;
 pub mod stdlib;
 pub mod file_io;
diff --git a/crates/mamba/src/runtime/symbols.rs b/crates/mamba/src/runtime/symbols.rs
index 9820b45..07ac730 100644
--- a/crates/mamba/src/runtime/symbols.rs
+++ b/crates/mamba/src/runtime/symbols.rs
@@ -46,6 +46,7 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
     use super::async_rt;
     use super::set_ops;
     use super::file_io;
+    use super::tokio_exec;
 
     vec![
         // ── Builtins ──
@@ -242,6 +243,9 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
         rt_sym!("mb_gil_held", async_rt::mb_gil_held as fn() -> super::MbValue, [], I64),
         // ── Async: Future interop (#313 R4) ──
         rt_sym!("mb_await_external", async_rt::mb_await_external as fn(super::MbValue) -> super::MbValue, [I64], I64),
+        // ── Async: Tokio multi-threaded executor (R6) ──
+        rt_sym!("mb_tokio_spawn", tokio_exec::mb_tokio_spawn as fn(super::MbValue) -> super::MbValue, [I64], I64),
+        rt_sym!("mb_tokio_gather", tokio_exec::mb_tokio_gather as fn(super::MbValue) -> super::MbValue, [I64], I64),
         // ── Property / classmethod / staticmethod ──
         rt_sym!("mb_property_new", class::mb_property_new as fn(super::MbValue) -> super::MbValue, [I64], I64),
         rt_sym!("mb_property_setter", class::mb_property_setter as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
diff --git a/crates/mamba/src/runtime/tokio_exec.rs b/crates/mamba/src/runtime/tokio_exec.rs
new file mode 100644
index 0000000..ccbccca
--- /dev/null
+++ b/crates/mamba/src/runtime/tokio_exec.rs
@@ -0,0 +1,212 @@
+/// Tokio multi-threaded executor for Mamba async tasks (R6).
+///
+/// Provides a shared Tokio runtime for true parallel async I/O.
+/// Mamba tasks are spawned on Tokio's thread pool and driven to
+/// completion via cooperative scheduling with GC safepoints.
+
+use std::sync::OnceLock;
+use super::value::MbValue;
+use super::rc::{MbObject, ObjData};
+use super::async_rt::{
+    COROUTINES, TASKS, alloc_task_id,
+    mb_coroutine_step, MbTask,
+};
+
+/// Global shared Tokio runtime (multi-threaded).
+static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
+
+/// Get or initialize the shared Tokio runtime.
+fn runtime() -> &'static tokio::runtime::Runtime {
+    TOKIO_RT.get_or_init(|| {
+        tokio::runtime::Builder::new_multi_thread()
+            .worker_threads(2)
+            .enable_all()
+            .thread_name("mamba-tokio")
+            .on_thread_start(|| {
+                super::gc::gc_register_thread();
+            })
+            .on_thread_stop(|| {
+                super::gc::gc_unregister_thread();
+            })
+            .build()
+            .expect("failed to create Mamba Tokio runtime")
+    })
+}
+
+/// Spawn a coroutine as a Tokio task for parallel execution.
+/// Returns a task handle (same as mb_create_task).
+pub fn mb_tokio_spawn(coro: MbValue) -> MbValue {
+    let coro_id = coro.as_int().unwrap_or(0) as u64;
+    let task = MbTask {
+        name: format!("tokio-task-{coro_id}"),
+        coroutine_id: coro_id,
+        done: false,
+        result: MbValue::none(),
+    };
+    let task_id = alloc_task_id();
+    TASKS.write().unwrap().insert(task_id, task);
+
+    let rt = runtime();
+    rt.spawn(async move {
+        // Drive the coroutine to completion
+        let coro_handle = MbValue::from_int(coro_id as i64);
+        let max_steps = 100_000;
+        for _ in 0..max_steps {
+            super::gc::gc_safepoint();
+            let exhausted = COROUTINES.read().unwrap()
+                .get(&coro_id).map(|c| c.exhausted).unwrap_or(true);
+            if exhausted { break; }
+            mb_coroutine_step(coro_handle);
+            // Yield to Tokio scheduler between steps
+            tokio::task::yield_now().await;
+        }
+
+        // Propagate result to task
+        let result = COROUTINES.read().unwrap()
+            .get(&coro_id)
+            .and_then(|c| c.result)
+            .unwrap_or(MbValue::none());
+        if let Some(task) = TASKS.write().unwrap().get_mut(&task_id) {
+            task.done = true;
+            task.result = result;
+        }
+    });
+
+    MbValue::from_int(task_id as i64)
+}
+
+/// Run multiple coroutines in parallel on the Tokio thread pool.
+/// Blocks until all complete and returns a list of results.
+pub fn mb_tokio_gather(coros: MbValue) -> MbValue {
+    let coro_ids: Vec<u64> = if let Some(ptr) = coros.as_ptr() {
+        unsafe {
+            if let ObjData::List(ref lock) = (*ptr).data {
+                lock.read().unwrap().iter()
+                    .filter_map(|c| c.as_int().map(|i| i as u64))
+                    .collect()
+            } else {
+                return MbValue::none();
+            }
+        }
+    } else {
+        return MbValue::none();
+    };
+
+    if coro_ids.is_empty() {
+        return MbValue::from_ptr(MbObject::new_list(vec![]));
+    }
+
+    let rt = runtime();
+    let mut handles = Vec::with_capacity(coro_ids.len());
+    let mut task_ids = Vec::with_capacity(coro_ids.len());
+
+    for &coro_id in &coro_ids {
+        let task = MbTask {
+            name: format!("tokio-gather-{coro_id}"),
+            coroutine_id: coro_id,
+            done: false,
+            result: MbValue::none(),
+        };
+        let task_id = alloc_task_id();
+        TASKS.write().unwrap().insert(task_id, task);
+        task_ids.push(task_id);
+
+        let handle = rt.spawn(async move {
+            let coro_handle = MbValue::from_int(coro_id as i64);
+            let max_steps = 100_000;
+            for _ in 0..max_steps {
+                super::gc::gc_safepoint();
+                let exhausted = COROUTINES.read().unwrap()
+                    .get(&coro_id).map(|c| c.exhausted).unwrap_or(true);
+                if exhausted { break; }
+                mb_coroutine_step(coro_handle);
+                tokio::task::yield_now().await;
+            }
+
+            COROUTINES.read().unwrap()
+                .get(&coro_id)
+                .and_then(|c| c.result)
+                .unwrap_or(MbValue::none())
+        });
+        handles.push(handle);
+    }
+
+    // Block current thread until all tasks complete
+    let results: Vec<MbValue> = rt.block_on(async {
+        let mut results = Vec::with_capacity(handles.len());
+        for (i, handle) in handles.into_iter().enumerate() {
+            let result = handle.await.unwrap_or(MbValue::none());
+            // Update task state
+            if let Some(task) = TASKS.write().unwrap().get_mut(&task_ids[i]) {
+                task.done = true;
+                task.result = result;
+            }
+            results.push(result);
+        }
+        results
+    });
+
+    MbValue::from_ptr(MbObject::new_list(results))
+}
+
+/// Shutdown the Tokio runtime (call at interpreter exit).
+pub fn mb_tokio_shutdown() {
+    // OnceLock doesn't support take(), so runtime lives for process lifetime.
+    // Tokio handles cleanup on process exit.
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use super::super::async_rt::{mb_coroutine_new, mb_coroutine_complete};
+
+    #[test]
+    fn test_runtime_initializes() {
+        let rt = runtime();
+        // Just verify it doesn't panic
+        rt.block_on(async { 42 });
+    }
+
+    #[test]
+    fn test_tokio_spawn_completed_coro() {
+        let name = MbValue::from_ptr(MbObject::new_str("tokio_test".to_string()));
+        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let coro = mb_coroutine_new(name, locals);
+        mb_coroutine_complete(coro, MbValue::from_int(42));
+
+        let task = mb_tokio_spawn(coro);
+        assert!(task.is_int());
+
+        // Give Tokio time to complete
+        std::thread::sleep(std::time::Duration::from_millis(50));
+
+        let task_id = task.as_int().unwrap() as u64;
+        let done = TASKS.read().unwrap()
+            .get(&task_id).map(|t| t.done).unwrap_or(false);
+        assert!(done, "Tokio task should complete for exhausted coroutine");
+    }
+
+    #[test]
+    fn test_tokio_gather_empty() {
+        let coros = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let result = mb_tokio_gather(coros);
+        assert!(result.as_ptr().is_some(), "gather of empty list should return list");
+    }
+
+    #[test]
+    fn test_tokio_gather_completed_coros() {
+        let n1 = MbValue::from_ptr(MbObject::new_str("g1".to_string()));
+        let l1 = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let c1 = mb_coroutine_new(n1, l1);
+        mb_coroutine_complete(c1, MbValue::from_int(10));
+
+        let n2 = MbValue::from_ptr(MbObject::new_str("g2".to_string()));
+        let l2 = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let c2 = mb_coroutine_new(n2, l2);
+        mb_coroutine_complete(c2, MbValue::from_int(20));
+
+        let coros = MbValue::from_ptr(MbObject::new_list(vec![c1, c2]));
+        let result = mb_tokio_gather(coros);
+        assert!(result.as_ptr().is_some(), "gather should return a list");
+    }
+}
diff --git a/crates/mamba/tests/thread_safety_tests.rs b/crates/mamba/tests/thread_safety_tests.rs
new file mode 100644
index 0000000..655b2bb
--- /dev/null
+++ b/crates/mamba/tests/thread_safety_tests.rs
@@ -0,0 +1,140 @@
+/// Integration tests for thread-safe Mamba runtime.
+///
+/// Tests safepoint-based STW GC, multi-thread async scheduling,
+/// and concurrent access to global async state.
+
+use cclab_mamba::runtime::value::MbValue;
+use cclab_mamba::runtime::rc::MbObject;
+use cclab_mamba::runtime::gc;
+use cclab_mamba::runtime::async_rt;
+use cclab_mamba::runtime::tokio_exec;
+
+#[test]
+fn test_safepoint_register_unregister() {
+    // Register and unregister a thread — no panic, no deadlock
+    gc::gc_register_thread();
+    gc::gc_safepoint();
+    gc::gc_unregister_thread();
+}
+
+#[test]
+fn test_safepoint_gc_collect_from_registered_thread() {
+    // A registered mutator that triggers collect() must not self-deadlock
+    gc::gc_register_thread();
+
+    let obj = MbObject::new_list(vec![]);
+    gc::gc_add_root(MbValue::from_ptr(obj));
+    let _freed = gc::collect();
+    // collect returned successfully = no deadlock
+    gc::gc_remove_root(MbValue::from_ptr(obj));
+    gc::gc_untrack(obj);
+    unsafe { drop(Box::from_raw(obj)); }
+
+    gc::gc_unregister_thread();
+}
+
+#[test]
+fn test_multi_thread_coroutine_access() {
+    // Multiple threads reading/writing coroutine state concurrently
+    let name = MbValue::from_ptr(MbObject::new_str("shared".to_string()));
+    let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
+    let coro = async_rt::mb_coroutine_new(name, locals);
+
+    let coro_bits = coro.to_bits();
+    let mut handles = Vec::new();
+    for i in 0..4 {
+        handles.push(std::thread::spawn(move || {
+            let handle = MbValue::from_bits(coro_bits);
+            // Read state from different threads
+            let state = async_rt::mb_coroutine_get_state(handle);
+            assert!(state < u32::MAX, "should read valid state from thread {i}");
+        }));
+    }
+    for h in handles {
+        h.join().unwrap();
+    }
+    async_rt::mb_coroutine_release(coro);
+}
+
+#[test]
+fn test_concurrent_coroutine_creation() {
+    // Multiple threads creating coroutines concurrently — IDs must be unique
+    let mut handles = Vec::new();
+    for _ in 0..4 {
+        handles.push(std::thread::spawn(|| {
+            let mut ids = Vec::new();
+            for _ in 0..100 {
+                let name = MbValue::from_ptr(MbObject::new_str("cc".to_string()));
+                let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
+                let coro = async_rt::mb_coroutine_new(name, locals);
+                ids.push(coro.as_int().unwrap());
+            }
+            ids
+        }));
+    }
+
+    let mut all_ids: Vec<i64> = Vec::new();
+    for h in handles {
+        all_ids.extend(h.join().unwrap());
+    }
+    let total = all_ids.len();
+    all_ids.sort();
+    all_ids.dedup();
+    assert_eq!(all_ids.len(), total, "all coroutine IDs must be unique across threads");
+}
+
+#[test]
+fn test_tokio_spawn_from_multiple_threads() {
+    // Spawn Tokio tasks from multiple threads
+    let mut handles = Vec::new();
+    for i in 0..4 {
+        handles.push(std::thread::spawn(move || {
+            let name = MbValue::from_ptr(
+                MbObject::new_str(format!("mt_coro_{i}"))
+            );
+            let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
+            let coro = async_rt::mb_coroutine_new(name, locals);
+            async_rt::mb_coroutine_complete(coro, MbValue::from_int(i as i64));
+            let task = tokio_exec::mb_tokio_spawn(coro);
+            task.as_int().unwrap()
+        }));
+    }
+
+    let task_ids: Vec<i64> = handles.into_iter()
+        .map(|h| h.join().unwrap())
+        .collect();
+
+    // Give Tokio time to complete
+    std::thread::sleep(std::time::Duration::from_millis(100));
+
+    for tid in &task_ids {
+        let done = async_rt::TASKS.read().unwrap()
+            .get(&(*tid as u64)).map(|t| t.done).unwrap_or(false);
+        assert!(done, "Tokio task {tid} should be done");
+    }
+}
+
+#[test]
+fn test_tokio_gather_parallel_execution() {
+    let mut coros = Vec::new();
+    for i in 0..4 {
+        let name = MbValue::from_ptr(
+            MbObject::new_str(format!("gather_coro_{i}"))
+        );
+        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let coro = async_rt::mb_coroutine_new(name, locals);
+        async_rt::mb_coroutine_complete(coro, MbValue::from_int((i + 1) * 10));
+        coros.push(coro);
+    }
+
+    let coro_list = MbValue::from_ptr(MbObject::new_list(coros));
+    let results = tokio_exec::mb_tokio_gather(coro_list);
+
+    assert!(results.as_ptr().is_some(), "gather should return a list");
+    unsafe {
+        if let cclab_mamba::runtime::rc::ObjData::List(ref lock) = (*results.as_ptr().unwrap()).data {
+            let items = lock.read().unwrap();
+            assert_eq!(items.len(), 4, "should have 4 results");
+        }
+    }
+}
diff --git a/crates/cclab-sdd/src/mcp/tools/agent.rs b/crates/cclab-sdd/src/mcp/tools/agent.rs
index 782ea36..05e3841 100644
--- a/crates/cclab-sdd/src/mcp/tools/agent.rs
+++ b/crates/cclab-sdd/src/mcp/tools/agent.rs
@@ -467,10 +467,20 @@ pub async fn execute_streaming(
     });
 
     if agent_result.status == "error" && !agent_result.state_changed {
+        let expected_phase_hint = get_verification(&action)
+            .and_then(|v| v.expected_phases.first().cloned())
+            .map(|p| crate::mcp::tools::workflow_common::phase_to_string(&p))
+            .unwrap_or_default();
         response["error"] = json!({
             "type": "agent_failure",
             "message": "Agent failed verification or exited with error",
             "retried": true,
+            "fallback_hint": format!(
+                "Agent '{}' failed for action '{}'. \
+                 Mainthread should perform the action manually, then call \
+                 sdd_run_change(advance_to=\"{}\") to continue.",
+                agent, action, expected_phase_hint
+            ),
         });
     }
 
diff --git a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
index 30bdd05..37cd963 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
@@ -155,11 +155,21 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {

... truncated (232 more lines)
```
