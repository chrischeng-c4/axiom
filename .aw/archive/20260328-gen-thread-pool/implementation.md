---
id: implementation
type: change_implementation
change_id: gen-thread-pool
---

# Implementation

## Summary

Replaced per-generator thread::spawn with GenPool (4 long-lived worker threads). Global GENERATOR_REGISTRY (DashMap) replaces thread-local GENERATORS. AtomicU64 for globally unique generator IDs. Barrier-based cleanup keeps pool alive across tests. Eliminates SIGBUS/SIGSEGV from pthread churn on macOS aarch64.

## Diff

```diff
diff --git a/crates/mamba/Cargo.toml b/crates/mamba/Cargo.toml
index 666a356e..cd13af93 100644
--- a/crates/mamba/Cargo.toml
+++ b/crates/mamba/Cargo.toml
@@ -43,6 +43,10 @@ chrono.workspace = true
 base64.workspace = true
 rand = "0.8"
 
+# Thread pool for generator workers (#1114 — GenPool)
+crossbeam-channel = "0.5"
+dashmap = { workspace = true }
+
 # Ordered dict (Python 3.7+ insertion order)
 indexmap = "2"
 
diff --git a/crates/mamba/src/runtime/generator.rs b/crates/mamba/src/runtime/generator.rs
index 481ea29c..36c8bcb1 100644
--- a/crates/mamba/src/runtime/generator.rs
+++ b/crates/mamba/src/runtime/generator.rs
@@ -1,93 +1,162 @@
-/// Generator functions and yield for the Mamba runtime (#290).
+/// Generator functions and yield for the Mamba runtime (#290, #1114).
 ///
-/// Generators are implemented using OS threads with channel-based yield/resume
-/// communication. Each generator body runs in a spawned thread. `yield value`
-/// sends the value through a channel and blocks until the caller resumes.
-/// `next()` / `send()` resume the generator and wait for the next yielded value.
+/// Generators are implemented using a thread pool (GenPool) with channel-based
+/// yield/resume communication. A fixed pool of long-lived worker threads
+/// executes generator bodies, eliminating per-generator `thread::spawn` which
+/// causes EXC_BAD_ACCESS on macOS aarch64 after ~130 spawn/join cycles due to
+/// cumulative pthread lifecycle corruption.
 ///
 /// Architecture:
 /// - Generator functions are compiled into a body function + constructor wrapper
 /// - The constructor creates a generator object and stores the body fn address + args
-/// - On first `next()`, the body thread is spawned
-/// - yield/resume use synchronous channels for bidirectional communication
+/// - On first `next()`, a GenJob is dispatched to the pool (not thread::spawn)
+/// - yield/resume use synchronous crossbeam channels for bidirectional communication
 /// - Output capture is shared across threads via Arc<Mutex<Vec<u8>>>
+/// - All generator state lives in a global `DashMap` registry (not thread-local)
+/// - `alloc_gen_id` uses `AtomicU64` for global uniqueness across pool workers
+/// - `cleanup_all_generators()` drains the registry, shuts down the pool, and
+///   joins all workers — guaranteeing no worker executes JIT code when
+///   `CraneliftJitBackend` drops
+
+use std::sync::atomic::{AtomicU64, Ordering};
+use std::sync::{Arc, Mutex};
+use std::thread::{self, JoinHandle};
+
+use crossbeam_channel::{self as cc};
+use dashmap::DashMap;
 
-use std::collections::HashMap;
-use std::sync::{mpsc, Arc, Mutex};
-use std::thread;
-use super::value::MbValue;
 use super::rc::{MbObject, ObjData};
+use super::value::MbValue;
 
-/// Messages from the generator thread to the caller.
+// ── Pool constants ───────────────────────────────────────────────────────────
+
+/// Number of worker threads in the generator pool.
+const POOL_SIZE: usize = 4;
+
+// ── Messages ─────────────────────────────────────────────────────────────────
+
+/// Messages from the generator worker to the caller.
 #[derive(Debug)]
 enum ToCallerMsg {
-    /// Generator yielded a value
+    /// Generator yielded a value.
     Yielded(MbValue),
-    /// Generator body returned (StopIteration with optional return value)
+    /// Generator body returned (StopIteration with optional return value).
     Returned(MbValue),
 }
 
-/// Messages from the caller to the generator thread.
+/// Messages from the caller to the generator worker.
 #[derive(Debug)]
 enum ToGenMsg {
-    /// Resume generator (next() or send(value))
+    /// Resume generator (next() or send(value)).
     Resume(MbValue),
-    /// Throw an exception into the generator
+    /// Throw an exception into the generator.
     Throw(String, String),
-    /// Close the generator (raise GeneratorExit)
+    /// Close the generator (raise GeneratorExit).
     Close,
 }
 
-/// State for a thread-based generator.
-struct ThreadedGen {
-    /// Channel: generator thread → caller
-    from_gen: mpsc::Receiver<ToCallerMsg>,
-    /// Channel: caller → generator thread
-    to_gen: mpsc::SyncSender<ToGenMsg>,
-    /// Whether the generator has been exhausted
+// ── Pool structures ──────────────────────────────────────────────────────────
+
+/// Messages dispatched through the pool job channel.
+enum PoolMsg {
+    /// A generator job to execute.
+    Job(GenJob),
+    /// Barrier: worker acknowledges by sending `()` on the oneshot sender.
+    /// Used by `cleanup_all_generators` to wait for all workers to be idle
+    /// (not executing JIT code) without destroying the pool.
+    Barrier(cc::Sender<()>),
+    /// Shutdown signal — worker should exit its loop.
+    Shutdown,
+}
+
+/// A job dispatched to a pool worker thread.
+struct GenJob {
+    /// Generator ID (for registry update after body returns).
+    gen_id: u64,
+    /// Body function address (NaN-boxed pointer bits).
+    body_fn_addr: u64,
+    /// Cloned arguments for the body function.
+    args: Vec<MbValue>,
+    /// Worker→caller channel sender (yields/returns go here).
+    to_caller: cc::Sender<ToCallerMsg>,
+    /// Caller→worker channel receiver (resume/throw/close come from here).
+    from_caller: cc::Receiver<ToGenMsg>,
+    /// Shared capture buffer for output.
+    shared_capture: Option<Arc<Mutex<Vec<u8>>>>,
+}
+
+/// Inner pool state holding worker thread handles and the job sender.
+struct GenPoolInner {
+    /// Worker thread join handles.
+    workers: Vec<JoinHandle<()>>,
+    /// Job channel sender — clone and send `PoolMsg::Job(...)` to dispatch.
+    sender: cc::Sender<PoolMsg>,
+}
+
+/// Caller-side channel endpoints wrapped in Arc so they can be shared
+/// without cloning the underlying crossbeam Receiver (which would create
+/// a second MPMC consumer and cause message-stealing races).
+struct GenChannels {
+    /// Sender: caller → worker (resume/throw/close).
+    to_gen: cc::Sender<ToGenMsg>,
+    /// Receiver: worker → caller (yielded/returned values).
+    from_gen: cc::Receiver<ToCallerMsg>,
+}
+
+/// Entry in the global generator registry.
+struct GenEntry {
+    /// Arc-wrapped caller-side channels (shared without cloning Receiver).
+    channels: Arc<GenChannels>,
+
+    // ── Worker-side endpoints (moved to GenJob on start) ──
+    /// Worker→caller sender (taken by `ensure_started`).
+    pending_to_caller: Option<cc::Sender<ToCallerMsg>>,
+    /// Caller→worker receiver (taken by `ensure_started`).
+    pending_from_caller: Option<cc::Receiver<ToGenMsg>>,
+
+    // ── Generator state ──
+    /// Whether the generator has been exhausted (body returned or closed).
     exhausted: bool,
-    /// Whether the generator has been started (first next() called)
+    /// Whether the generator has been started (first next() dispatched job).
     started: bool,
-    /// Return value from the generator body (StopIteration.value)
+    /// Return value from the generator body (StopIteration.value).
     return_value: MbValue,
-    /// Thread handle
-    _thread: Option<thread::JoinHandle<()>>,
-    /// Body function address (for deferred thread spawn)
+    /// Body function address (NaN-boxed pointer bits).
     body_fn_addr: u64,
-    /// Captured arguments for the body function
+    /// Captured arguments for the body function.
     args: Vec<MbValue>,
-    /// Generator name
+    /// Generator name (for debugging).
     #[allow(dead_code)]
     name: String,
 }
 
-// Thread-local generator storage.
-thread_local! {
-    static GENERATORS: std::cell::RefCell<HashMap<u64, ThreadedGen>> =
-        std::cell::RefCell::new(HashMap::new());
-    static NEXT_GEN_ID: std::cell::Cell<u64> = std::cell::Cell::new(1);
-}
+// ── Global state ─────────────────────────────────────────────────────────────
 
-fn alloc_gen_id() -> u64 {
-    NEXT_GEN_ID.with(|cell| {
-        let id = cell.get();
-        cell.set(id + 1);
-        id
-    })
-}
+/// Pool singleton. `Mutex<Option<...>>` allows re-initialization after
+/// `cleanup_all_generators()` shuts the pool down.
+static GEN_POOL: Mutex<Option<GenPoolInner>> = Mutex::new(None);
+
+/// Global generator registry — replaces thread-local `GENERATORS` HashMap.
+/// `DashMap` provides shard-level concurrent read/write.
+static GENERATOR_REGISTRY: std::sync::LazyLock<DashMap<u64, GenEntry>> =
+    std::sync::LazyLock::new(DashMap::new);
+
+/// Atomic generator ID counter — replaces thread-local `Cell<u64>`.
+/// `fetch_add(1, Relaxed)` guarantees unique IDs across all pool workers.
+static NEXT_GEN_ID: AtomicU64 = AtomicU64::new(1);
 
-// ── Thread-local channels for yield from within the generator body thread ──
+// ── Thread-local channels (set per-job by worker threads) ────────────────────
 
 thread_local! {
-    /// Sender to the caller (used by mb_generator_yield_value in the gen thread)
-    static GEN_TX: std::cell::RefCell<Option<mpsc::SyncSender<ToCallerMsg>>> =
+    /// Sender to the caller (used by `mb_generator_yield_value` in the worker).
+    static GEN_TX: std::cell::RefCell<Option<cc::Sender<ToCallerMsg>>> =
         std::cell::RefCell::new(None);
-    /// Receiver from the caller (used by mb_generator_yield_value in the gen thread)
-    static GEN_RX: std::cell::RefCell<Option<mpsc::Receiver<ToGenMsg>>> =
+    /// Receiver from the caller (used by `mb_generator_yield_value` in the worker).
+    static GEN_RX: std::cell::RefCell<Option<cc::Receiver<ToGenMsg>>> =
         std::cell::RefCell::new(None);
 }
 
-// ── Shared output capture for generator threads ──
+// ── Shared output capture for generator threads ──────────────────────────────
 
 thread_local! {
     /// Shared capture buffer for output from generator threads.
@@ -112,8 +181,8 @@ fn set_shared_capture(buf: Option<Arc<Mutex<Vec<u8>>>>) {
 }
 
 /// Activate shared capture mode: creates a shared buffer and redirects
-/// the current thread's capture to use it. Called before spawning a
-/// generator thread.
+/// the current thread's capture to use it. Called before dispatching a
+/// generator job to the pool.
 pub fn activate_shared_capture() -> Option<Arc<Mutex<Vec<u8>>>> {
     if super::output::is_capturing() {
         let shared = SHARED_CAPTURE.with(|sc| sc.borrow().clone());
@@ -159,141 +228,242 @@ pub fn flush_shared_capture() {
     });
 }
 
-// ── Generator Creation ──
+// ── Pool initialization & worker loop ────────────────────────────────────────
 
-/// Create a new thread-based generator. Called from compiled constructor wrapper.
-/// Arguments:
-/// - body_fn_addr: NaN-boxed pointer to the compiled body function
-/// - arg_count: number of arguments packed after this
-/// The actual args are passed via mb_generator_create_with_args.
+/// Get (or lazily create) the pool sender for dispatching jobs.
+fn get_pool_sender() -> cc::Sender<PoolMsg> {
+    let mut pool = GEN_POOL.lock().unwrap();
+    if pool.is_none() {
+        *pool = Some(init_pool());
+    }
+    pool.as_ref().unwrap().sender.clone()
+}
+
+/// Spawn the pool worker threads and return the pool state.
+fn init_pool() -> GenPoolInner {
+    let (sender, receiver) = cc::unbounded::<PoolMsg>();
+    let mut workers = Vec::with_capacity(POOL_SIZE);
+
+    for i in 0..POOL_SIZE {
+        let rx = receiver.clone();
+        let handle = thread::Builder::new()
+            .name(format!("mamba-gen-worker-{i}"))
+            .spawn(move || worker_loop(rx))
+            .expect("failed to spawn generator pool worker");
+        workers.push(handle);
+    }
+
+    GenPoolInner { workers, sender }
+}
+
+/// Main loop for a pool worker thread. Receives jobs and executes them;
+/// exits on `Shutdown` sentinel or channel disconnect.
+fn worker_loop(receiver: cc::Receiver<PoolMsg>) {
+    loop {
+        match receiver.recv() {
+            Ok(PoolMsg::Job(job)) => execute_gen_job(job),
+            Ok(PoolMsg::Barrier(ack)) => {
+                // Acknowledge — proves this worker is idle (not in JIT code).
+                let _ = ack.send(());
+            }
+            Ok(PoolMsg::Shutdown) | Err(_) => break,
+        }
+    }
+}
+
+/// Execute a single generator job on the current worker thread.
+fn execute_gen_job(job: GenJob) {
+    let GenJob {
+        gen_id,
+        body_fn_addr,
+        args,
+        to_caller,
+        from_caller,
+        shared_capture,
+    } = job;
+
+    // Reset stale thread-local state from the previous job on this worker.
+    // Pool workers are long-lived, so runtime thread-locals (StopIteration
+    // flags, exceptions, iterators) may persist from an earlier generator.
+    super::iter::cleanup_all_iterators();
+    super::exception::cleanup_all_exceptions();
+    super::iter::check_and_clear_stop();
+
+    // Set up thread-local channels for this job
+    GEN_TX.with(|tx| *tx.borrow_mut() = Some(to_caller.clone()));
+    GEN_RX.with(|rx| *rx.borrow_mut() = Some(from_caller));
+
+    // Set up shared output capture
+    if let Some(ref cap) = shared_capture {
+        set_shared_capture(Some(cap.clone()));
+    }
+
+    // Wait for the first Resume signal before starting execution
+    let first_msg = GEN_RX.with(|rx| {
+        rx.borrow().as_ref().and_then(|r| r.recv().ok())
+    });
+    match first_msg {
+        Some(ToGenMsg::Close) => {
+            let _ = to_caller.send(ToCallerMsg::Returned(MbValue::none()));
+            cleanup_worker_thread_locals();
+            return;
+        }
+        Some(ToGenMsg::Throw(_exc_type, _msg)) => {
+            // Throw before first yield — just return as StopIteration
+            let _ = to_caller.send(ToCallerMsg::Returned(MbValue::none()));
+            cleanup_worker_thread_locals();
+            return;
+        }
+        Some(ToGenMsg::Resume(_)) => {
+            // Good, start executing the body
+        }
+        None => {
+            // Channel closed (generator dropped before start)
+            cleanup_worker_thread_locals();
+            return;
+        }
+    }
+
+    // Call the compiled body function
+    let return_value = call_body_fn(body_fn_addr, &args);
+
+    // Body returned — send final value to caller
+    let _ = to_caller.send(ToCallerMsg::Returned(return_value));
+
+    // Update registry entry state to Completed
+    if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&gen_id) {
+        entry.exhausted = true;
+    }
+
+    // Clear thread-locals so the worker is clean for the next job
+    cleanup_worker_thread_locals();
+}
+
+/// Clear per-job thread-local state after a generator job completes.
+///
+/// Pool workers are long-lived — unlike per-generator threads, their
+/// thread-locals persist across jobs.  We must reset ALL runtime
+/// thread-locals that a generator body might have touched, including
+/// StopIteration flags, exception state, and iterator tables.
+fn cleanup_worker_thread_locals() {
+    GEN_TX.with(|tx| *tx.borrow_mut() = None);
+    GEN_RX.with(|rx| *rx.borrow_mut() = None);
+    set_shared_capture(None);
+    // Reset runtime thread-locals that the generator body may have modified.
+    super::iter::cleanup_all_iterators();
+    super::exception::cleanup_all_exceptions();
+    super::iter::check_and_clear_stop();
+}
+
+// ── Generator ID allocation ──────────────────────────────────────────────────
+
+fn alloc_gen_id() -> u64 {
+    NEXT_GEN_ID.fetch_add(1, Ordering::Relaxed)
+}
+
+// ── Generator Creation ───────────────────────────────────────────────────────
+
+/// Create a new generator. Called from compiled constructor wrapper.
+/// Lazily initializes the pool on first call.
 pub fn mb_generator_create(name: MbValue, body_fn_addr: MbValue) -> MbValue {
     let gen_name = extract_str(name).unwrap_or_else(|| "<generator>".to_string());
     let fn_addr = body_fn_addr.to_bits();
 
     let id = alloc_gen_id();
-    // Create placeholder channels (will be created when thread spawns)
-    let (to_caller_tx, to_caller_rx) = mpsc::sync_channel::<ToCallerMsg>(0);
-    let (to_gen_tx, to_gen_rx) = mpsc::sync_channel::<ToGenMsg>(0);
 
-    let gen = ThreadedGen {
-        from_gen: to_caller_rx,
+    // Lazily initialize the pool (no-op if already active)
+    let _ = get_pool_sender();
+
+    // Create bidirectional channels:
+    //   to_gen_tx → to_gen_rx : caller sends Resume/Throw/Close, worker receives
+    //   to_caller_tx → to_caller_rx : worker sends Yielded/Returned, caller receives
+    let (to_caller_tx, to_caller_rx) = cc::bounded::<ToCallerMsg>(0);
+    let (to_gen_tx, to_gen_rx) = cc::bounded::<ToGenMsg>(0);
+
+    let channels = Arc::new(GenChannels {
         to_gen: to_gen_tx,
+        from_gen: to_caller_rx,
+    });
+
+    let entry = GenEntry {
+        channels,
+        pending_to_caller: Some(to_caller_tx),
+        pending_from_caller: Some(to_gen_rx),
         exhausted: false,
         started: false,
         return_value: MbValue::none(),
-        _thread: None,
         body_fn_addr: fn_addr,
         args: Vec::new(),
         name: gen_name,
     };
-    // Store sender/receiver that will be moved to the thread later
-    // We need to keep them around temporarily
-    GENERATORS.with(|gens| { gens.borrow_mut().insert(id, gen); });
-    // Store the channel endpoints for thread spawn
-    PENDING_CHANNELS.with(|pc| {
-        pc.borrow_mut().insert(id, (to_caller_tx, to_gen_rx));
-    });
+
+    GENERATOR_REGISTRY.insert(id, entry);
     MbValue::from_int(id as i64)
 }
 
-thread_local! {
-    static PENDING_CHANNELS: std::cell::RefCell<HashMap<u64, (
-        mpsc::SyncSender<ToCallerMsg>,
-        mpsc::Receiver<ToGenMsg>,
-    )>> = std::cell::RefCell::new(HashMap::new());
-}
-
 /// Store an argument for the generator (called after mb_generator_create).
 pub fn mb_generator_store_arg(gen_handle: MbValue, arg: MbValue) {
     if let Some(id) = gen_handle.as_int() {
-        GENERATORS.with(|gens| {
-            if let Some(gen) = gens.borrow_mut().get_mut(&(id as u64)) {
-                gen.args.push(arg);
-            }
-        });
+        if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&(id as u64)) {
+            entry.args.push(arg);
+        }
     }
 }
 
-/// Spawn the generator thread if not already started.
+/// Dispatch the generator body to a pool worker if not already started.
 fn ensure_started(id: u64) {
-    let should_start = GENERATORS.with(|gens| {
-        gens.borrow().get(&id).map(|g| !g.started).unwrap_or(false)
-    });
-    if !should_start { return; }
-
-    // Get the pending channels
-    let channels = PENDING_CHANNELS.with(|pc| pc.borrow_mut().remove(&id));
-    let (to_caller_tx, to_gen_rx) = match channels {
-        Some(c) => c,
+    // Quick read check — avoid write lock if already started
+    let should_start = GENERATOR_REGISTRY
+        .get(&id)
+        .map(|e| !e.started)
+        .unwrap_or(false);
+    if !should_start {
+        return;
+    }
+
+    // Take pending channel endpoints and args under exclusive access
+    let job_data = {
+        let mut entry = match GENERATOR_REGISTRY.get_mut(&id) {
+            Some(e) => e,
+            None => return,
+        };
+        if entry.started {
+            return; // Double-check under exclusive access
+        }
+        entry.started = true;
+
+        let to_caller = entry.pending_to_caller.take();
+        let from_caller = entry.pending_from_caller.take();
+
+        match (to_caller, from_caller) {
+            (Some(tc), Some(fc)) => {
+                Some((entry.body_fn_addr, entry.args.clone(), tc, fc))
+            }
+            _ => None,
+        }
+        // DashMap RefMut dropped here
+    };
+
+    let (body_fn_addr, args, to_caller, from_caller) = match job_data {
+        Some(data) => data,
         None => return,
     };
 
-    // Get body function address and args
-    let (body_fn_addr, args) = GENERATORS.with(|gens| {
-        let gens = gens.borrow();
-        let gen = gens.get(&id).unwrap();
-        (gen.body_fn_addr, gen.args.clone())
-    });
-
-    // Get shared capture buffer for the generator thread
+    // Get shared capture buffer for the generator
     let shared_capture = activate_shared_capture();
 
-    // Mark as started
-    GENERATORS.with(|gens| {
-        if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-            gen.started = true;
-        }
-    });
-
-    // Spawn the generator thread
-    let thread = thread::spawn(move || {
-        // Set up thread-local channels for this generator thread
-        GEN_TX.with(|tx| { *tx.borrow_mut() = Some(to_caller_tx.clone()); });
-        GEN_RX.with(|rx| { *rx.borrow_mut() = Some(to_gen_rx); });
-
-        // Set up shared output capture
-        if let Some(ref cap) = shared_capture {
-            set_shared_capture(Some(cap.clone()));
-        }
-
-        // Wait for the first Resume signal before starting execution
-        let first_msg = GEN_RX.with(|rx| {
-            rx.borrow().as_ref().and_then(|r| r.recv().ok())
-        });
-        match first_msg {
-            Some(ToGenMsg::Close) => {
-                let _ = to_caller_tx.send(ToCallerMsg::Returned(MbValue::none()));
-                return;
-            }
-            Some(ToGenMsg::Throw(_exc_type, _msg)) => {
-                // Throw before first yield - just return as StopIteration
-                let _ = to_caller_tx.send(ToCallerMsg::Returned(MbValue::none()));
-                return;
-            }
-            Some(ToGenMsg::Resume(_)) => {
-                // Good, start executing
-            }
-            None => return, // Channel closed
-        }
-
-        // Call the compiled body function
-        // The body function signature: fn(gen_handle: i64, arg0: i64, arg1: i64, ...) -> i64
-        // We pack the generator handle as first arg so yield can reference it
-        // Actually, the body just uses thread-local GEN_TX/GEN_RX for yield
-        // So the body takes just the original args: fn(arg0, arg1, ...) -> i64
-
-        let fn_ptr = body_fn_addr;
-        let return_value = call_body_fn(fn_ptr, &args);
-
-        // Body returned — send final value
-        let _ = to_caller_tx.send(ToCallerMsg::Returned(return_value));
-    });
-
-    GENERATORS.with(|gens| {
-        if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-            gen._thread = Some(thread);
-        }
-    });
+    let job = GenJob {
+        gen_id: id,
+        body_fn_addr,
+        args,
+        to_caller,
+        from_caller,
+        shared_capture,
+    };
+
+    // Dispatch job to pool
+    let sender = get_pool_sender();
+    let _ = sender.send(PoolMsg::Job(job));
 }
 
 /// Call the compiled body function with the given arguments.
@@ -301,7 +471,9 @@ fn ensure_started(id: u64) {
 fn call_body_fn(fn_addr: u64, args: &[MbValue]) -> MbValue {
     // Extract raw pointer from NaN-boxed pointer value
     let raw_addr = fn_addr & 0x0000_FFFF_FFFF_FFFF; // strip NaN prefix
-    if raw_addr == 0 { return MbValue::none(); }
+    if raw_addr == 0 {
+        return MbValue::none();
+    }
 
     // Call based on number of arguments
     unsafe {
@@ -311,15 +483,20 @@ fn call_body_fn(fn_addr: u64, args: &[MbValue]) -> MbValue {
                 MbValue::from_bits(f() as u64)
             }
             1 => {
-                let f: extern "C" fn(i64) -> i64 = std::mem::transmute(raw_addr as usize);
+                let f: extern "C" fn(i64) -> i64 =
+                    std::mem::transmute(raw_addr as usize);
                 MbValue::from_bits(f(args[0].to_bits() as i64) as u64)
             }
             2 => {
-                let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(raw_addr as usize);
-                MbValue::from_bits(f(args[0].to_bits() as i64, args[1].to_bits() as i64) as u64)
+                let f: extern "C" fn(i64, i64) -> i64 =
+                    std::mem::transmute(raw_addr as usize);
+                MbValue::from_bits(
+                    f(args[0].to_bits() as i64, args[1].to_bits() as i64) as u64,
+                )
             }
             3 => {
-                let f: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(raw_addr as usize);
+                let f: extern "C" fn(i64, i64, i64) -> i64 =
+                    std::mem::transmute(raw_addr as usize);
                 MbValue::from_bits(f(
                     args[0].to_bits() as i64,
                     args[1].to_bits() as i64,
@@ -327,7 +504,8 @@ fn call_body_fn(fn_addr: u64, args: &[MbValue]) -> MbValue {
                 ) as u64)
             }
             4 => {
-                let f: extern "C" fn(i64, i64, i64, i64) -> i64 = std::mem::transmute(raw_addr as usize);
+                let f: extern "C" fn(i64, i64, i64, i64) -> i64 =
+                    std::mem::transmute(raw_addr as usize);
                 MbValue::from_bits(f(
                     args[0].to_bits() as i64,
                     args[1].to_bits() as i64,
@@ -340,7 +518,7 @@ fn call_body_fn(fn_addr: u64, args: &[MbValue]) -> MbValue {
     }
 }
 
-// ── Generator Protocol ──
+// ── Generator Protocol ───────────────────────────────────────────────────────
 
 /// Advance the generator (next()). Returns the yielded value.
 /// On exhaustion, signals StopIteration via the iter flag.
@@ -354,34 +532,36 @@ pub fn mb_generator_send(gen_handle: MbValue, value: MbValue) -> MbValue {
         let id = id as u64;
 
         // Check if exhausted
-        let exhausted = GENERATORS.with(|gens| {
-            gens.borrow().get(&id).map(|g| g.exhausted).unwrap_or(true)
-        });
+        let exhausted = GENERATOR_REGISTRY
+            .get(&id)
+            .map(|e| e.exhausted)
+            .unwrap_or(true);
         if exhausted {
-            // Signal StopIteration (both flag and exception)
             raise_stop_iteration(MbValue::none());
             return MbValue::none();
         }
 
-        // Ensure thread is started
+        // Ensure worker is started
         ensure_started(id);
 
-        // Send resume signal
-        let send_ok = GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&id) {
-                gen.to_gen.send(ToGenMsg::Resume(value)).is_ok()
-            } else {
-                false
+        // Clone the Arc<GenChannels> so we don't hold DashMap lock during
+        // blocking channel I/O.  The Arc is a pointer bump, not a Receiver clone.
+        let ch = match GENERATOR_REGISTRY.get(&id) {
+            Some(entry) => entry.channels.clone(),
+            None => {
+                raise_stop_iteration(MbValue::none());
+                return MbValue::none();
             }
-        });
+        };
+        // DashMap Ref dropped here
+
+        // Send resume signal (blocks until worker is at a yield/recv point)
+        let send_ok = ch.to_gen.send(ToGenMsg::Resume(value)).is_ok();
 
         if !send_ok {
-            GENERATORS.with(|gens| {
-                if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                    gen.exhausted = true;
-                }
-            });
+            if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                entry.exhausted = true;
+            }
             raise_stop_iteration(MbValue::none());
             return MbValue::none();
         }
@@ -389,15 +569,8 @@ pub fn mb_generator_send(gen_handle: MbValue, value: MbValue) -> MbValue {
         // Flush any shared capture output from previous yield
         flush_shared_capture();
 
-        // Wait for yielded value
-        let msg = GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&id) {
-                gen.from_gen.recv().ok()
-            } else {
-                None
-            }
-        });
+        // Wait for yielded value (blocks until worker yields or returns)
+        let msg = ch.from_gen.recv().ok();
 
         // Flush capture output after receiving
         flush_shared_capture();
@@ -405,21 +578,17 @@ pub fn mb_generator_send(gen_handle: MbValue, value: MbValue) -> MbValue {
         match msg {
             Some(ToCallerMsg::Yielded(val)) => val,
             Some(ToCallerMsg::Returned(ret_val)) => {
-                GENERATORS.with(|gens| {
-                    if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                        gen.exhausted = true;
-                        gen.return_value = ret_val;
-                    }
-                });
+                if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                    entry.exhausted = true;
+                    entry.return_value = ret_val;
+                }
                 raise_stop_iteration(ret_val);
                 MbValue::none()
             }
             None => {
-                GENERATORS.with(|gens| {
-                    if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                        gen.exhausted = true;
-                    }
-                });
+                if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                    entry.exhausted = true;
+                }
                 raise_stop_iteration(MbValue::none());
                 MbValue::none()
             }
@@ -440,38 +609,43 @@ pub fn mb_generator_stop_value() -> MbValue {
 }
 
 /// Throw an exception into the generator.
-pub fn mb_generator_throw(gen_handle: MbValue, exc_type: MbValue, exc_msg: MbValue) -> MbValue {
+pub fn mb_generator_throw(
+    gen_handle: MbValue,
+    exc_type: MbValue,
+    exc_msg: MbValue,
+) -> MbValue {
     if let Some(id) = gen_handle.as_int() {
         let id = id as u64;
-        let exhausted = GENERATORS.with(|gens| {
-            gens.borrow().get(&id).map(|g| g.exhausted).unwrap_or(true)
-        });
+        let exhausted = GENERATOR_REGISTRY
+            .get(&id)
+            .map(|e| e.exhausted)
+            .unwrap_or(true);
         if exhausted {
             super::iter::signal_stop_iteration();
             return MbValue::none();
         }
 
-        // Ensure thread is started
+        // Ensure worker is started
         ensure_started(id);
 
         let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
         let msg = extract_str(exc_msg).unwrap_or_default();
 
-        let send_ok = GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&id) {
-                gen.to_gen.send(ToGenMsg::Throw(type_name, msg)).is_ok()
-            } else {
-                false
+        // Clone the Arc so we don't hold DashMap lock during blocking I/O
+        let ch = match GENERATOR_REGISTRY.get(&id) {
+            Some(entry) => entry.channels.clone(),
+            None => {
+                super::iter::signal_stop_iteration();
+                return MbValue::none();
             }
-        });
+        };
+
+        let send_ok = ch.to_gen.send(ToGenMsg::Throw(type_name, msg)).is_ok();
 
         if !send_ok {
-            GENERATORS.with(|gens| {
-                if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                    gen.exhausted = true;
-                }
-            });
+            if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                entry.exhausted = true;
+            }
             super::iter::signal_stop_iteration();
             return MbValue::none();
         }
@@ -479,35 +653,24 @@ pub fn mb_generator_throw(gen_handle: MbValue, exc_type: MbValue, exc_msg: MbVal
         flush_shared_capture();
 
         // Wait for response (yield or return)
-        let msg = GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&id) {
-                gen.from_gen.recv().ok()
-            } else {
-                None
-            }
-        });
+        let msg = ch.from_gen.recv().ok();
 
         flush_shared_capture();
 
         match msg {
             Some(ToCallerMsg::Yielded(val)) => val,
             Some(ToCallerMsg::Returned(ret_val)) => {
-                GENERATORS.with(|gens| {
-                    if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                        gen.exhausted = true;
-                        gen.return_value = ret_val;
-                    }
-                });
+                if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                    entry.exhausted = true;
+                    entry.return_value = ret_val;
+                }
                 super::iter::signal_stop_iteration();
                 MbValue::none()
             }
             None => {
-                GENERATORS.with(|gens| {
-                    if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                        gen.exhausted = true;
-                    }
-                });
+                if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+                    entry.exhausted = true;
+                }
                 super::iter::signal_stop_iteration();
                 MbValue::none()
             }
@@ -521,64 +684,46 @@ pub fn mb_generator_throw(gen_handle: MbValue, exc_type: MbValue, exc_msg: MbVal
 pub fn mb_generator_close(gen_handle: MbValue) {
     if let Some(id) = gen_handle.as_int() {
         let id = id as u64;
-        let exhausted = GENERATORS.with(|gens| {
-            gens.borrow().get(&id).map(|g| g.exhausted).unwrap_or(true)
-        });
-        if exhausted { return; }
+        let exhausted = GENERATOR_REGISTRY
+            .get(&id)
+            .map(|e| e.exhausted)
+            .unwrap_or(true);
+        if exhausted {
+            return;
+        }
 
         ensure_started(id);
 
-        let send_ok = GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&id) {
-                gen.to_gen.send(ToGenMsg::Close).is_ok()
-            } else {
-                false
-            }
-        });
+        // Clone Arc to avoid holding DashMap lock during blocking I/O
+        let ch = match GENERATOR_REGISTRY.get(&id) {
+            Some(entry) => entry.channels.clone(),
+            None => return,
+        };
+
+        let send_ok = ch.to_gen.send(ToGenMsg::Close).is_ok();
 
         if send_ok {
             flush_shared_capture();
-            // Wait for the Returned message
-            let _msg = GENERATORS.with(|gens| {
-                let gens = gens.borrow();
-                if let Some(gen) = gens.get(&id) {
-                    gen.from_gen.recv().ok()
-                } else {
-                    None
-                }
-            });
+            // Wait for the Returned message (worker finishes body + cleanup)
+            let _ = ch.from_gen.recv();
             flush_shared_capture();
         }
 
-        // Join the thread to ensure it has fully terminated before we
-        // return. This prevents use-after-free of JIT code memory.
-        let thread_handle = GENERATORS.with(|gens| {
-            gens.borrow_mut().get_mut(&id).and_then(|gen| gen._thread.take())
-        });
-        if let Some(handle) = thread_handle {
-            let _ = handle.join();
-        }
+        // No per-generator thread to join — worker returns to pool
 
-        GENERATORS.with(|gens| {
-            if let Some(gen) = gens.borrow_mut().get_mut(&id) {
-                gen.exhausted = true;
-            }
-        });
+        if let Some(mut entry) = GENERATOR_REGISTRY.get_mut(&id) {
+            entry.exhausted = true;
+        }
     }
 }
 
 /// Check if a generator is exhausted.
 pub fn mb_generator_is_exhausted(gen_handle: MbValue) -> MbValue {
     if let Some(id) = gen_handle.as_int() {
-        GENERATORS.with(|gens| {
-            let gens = gens.borrow();
-            if let Some(gen) = gens.get(&(id as u64)) {
-                MbValue::from_bool(gen.exhausted)
-            } else {
-                MbValue::from_bool(true)
-            }
-        })
+        GENERATOR_REGISTRY
+            .get(&(id as u64))
+            .map(|e| MbValue::from_bool(e.exhausted))
+            .unwrap_or_else(|| MbValue::from_bool(true))
     } else {
         MbValue::from_bool(true)
     }
@@ -587,8 +732,7 @@ pub fn mb_generator_is_exhausted(gen_handle: MbValue) -> MbValue {
 /// Check if a value is a known generator handle.
 pub fn is_known_generator(gen_handle: MbValue) -> bool {
     if let Some(id) = gen_handle.as_int() {
-        GENERATORS.with(|gens| gens.borrow().contains_key(&(id as u64)))
-            || PENDING_CHANNELS.with(|pc| pc.borrow().contains_key(&(id as u64)))
+        GENERATOR_REGISTRY.contains_key(&(id as u64))
     } else {
         false
     }
@@ -597,71 +741,80 @@ pub fn is_known_generator(gen_handle: MbValue) -> bool {
 /// Release a generator's resources.
 pub fn mb_generator_release(gen_handle: MbValue) {
     if let Some(id) = gen_handle.as_int() {
-        GENERATORS.with(|gens| { gens.borrow_mut().remove(&(id as u64)); });
-        PENDING_CHANNELS.with(|pc| { pc.borrow_mut().remove(&(id as u64)); });
+        GENERATOR_REGISTRY.remove(&(id as u64));
     }
 }
 
-/// Close all active generators and join their threads.
+/// Close all active generators, drain the global registry, and ensure no
+/// pool worker is executing JIT code.
 ///
-/// Must be called before JIT code memory is freed to prevent generator
-/// threads from executing deallocated code.
+/// The pool is **kept alive** across calls to avoid pthread churn — only a
+/// barrier synchronization is used to prove all workers are idle.  This is
+/// critical on macOS aarch64 where ~130 thread spawn/join cycles corrupt
+/// process state.
+///
+/// Must be called before JIT code memory is freed to prevent workers from
+/// executing deallocated code.
 pub fn cleanup_all_generators() {
-    // Collect all generator IDs
-    let ids: Vec<u64> = GENERATORS.with(|gens| {
-        gens.borrow().keys().copied().collect()
-    });
+    // 1. Close all active (started, not exhausted) generators so workers
+    //    finish their current JIT body execution and return to the pool loop.
+    let ids: Vec<u64> = GENERATOR_REGISTRY.iter().map(|e| *e.key()).collect();
 
     for id in ids {
-        let exhausted = GENERATORS.with(|gens| {
-            gens.borrow().get(&id).map(|g| g.exhausted).unwrap_or(true)
+        let info = GENERATOR_REGISTRY.get(&id).map(|e| {
+            (e.exhausted, e.started, e.channels.clone())
         });
 
-        if !exhausted {
-            // Try to close: send Close and wait for response
-            let started = GENERATORS.with(|gens| {
-                gens.borrow().get(&id).map(|g| g.started).unwrap_or(false)
-            });
-
-            if started {
-                let send_ok = GENERATORS.with(|gens| {
-                    let gens = gens.borrow();
-                    if let Some(gen) = gens.get(&id) {
-                        gen.to_gen.send(ToGenMsg::Close).is_ok()
-                    } else {
-                        false
-                    }
-                });
-
-                if send_ok {
-                    // Wait for the response (drains the thread)
-                    let _msg = GENERATORS.with(|gens| {
-                        let gens = gens.borrow();
-                        if let Some(gen) = gens.get(&id) {
-                            gen.from_gen.recv().ok()
-                        } else {
-                            None
-                        }
-                    });
+        if let Some((exhausted, started, ch)) = info {
+            if !exhausted && started {
+                // Send Close and wait for acknowledgement. This ensures the
+                // worker has left the JIT body function before we proceed.
+                if ch.to_gen.send(ToGenMsg::Close).is_ok() {
+                    let _ = ch.from_gen.recv();
                 }
             }
+        }
+    }
 
-            // Join the thread
-            let handle = GENERATORS.with(|gens| {
-                gens.borrow_mut().get_mut(&id).and_then(|g| g._thread.take())
-            });
-            if let Some(h) = handle {
-                let _ = h.join();
-            }
+    // 2. Drain the global registry (drops all channel endpoints).
+    GENERATOR_REGISTRY.clear();
+
+    // 3. Barrier: send a Barrier message to each pool worker and wait for
+    //    all of them to acknowledge. Once every worker has responded, we
+    //    know no worker is executing JIT code — it is safe to drop the
+    //    CraneliftJitBackend. The pool stays alive for the next test.
+    let pool = GEN_POOL.lock().unwrap();
+    if let Some(ref pool) = *pool {
+        let (ack_tx, ack_rx) = cc::bounded::<()>(0);
+        for _ in &pool.workers {
+            let _ = pool.sender.send(PoolMsg::Barrier(ack_tx.clone()));
+        }
+        drop(ack_tx); // Drop our copy so ack_rx closes when all workers ack
+        // Wait for every worker to acknowledge
+        for _ in &pool.workers {
+            let _ = ack_rx.recv();
         }
     }
+}
 
-    // Clear everything
-    GENERATORS.with(|gens| gens.borrow_mut().clear());
-    PENDING_CHANNELS.with(|pc| pc.borrow_mut().clear());
+/// Shut down the pool permanently: send Shutdown sentinels and join all
+/// worker threads.  Called only during process exit or when the pool must
+/// be fully destroyed.
+#[allow(dead_code)]
+pub fn shutdown_pool() {
+    let pool = GEN_POOL.lock().unwrap().take();
+    if let Some(pool) = pool {
+        for _ in &pool.workers {
+            let _ = pool.sender.send(PoolMsg::Shutdown);
+        }
+        drop(pool.sender);
+        for handle in pool.workers {
+            let _ = handle.join();
+        }
+    }
 }
 
-// ── Called from compiled generator body code (runs in generator thread) ──
+// ── Called from compiled generator body code (runs in pool worker) ────────────
 
 /// Yield a value from the generator body. Called from compiled code.
 /// Sends the value to the caller and blocks until resume.
@@ -676,7 +829,9 @@ pub fn mb_generator_yield_value(value: MbValue) -> MbValue {
             false
         }
     });
-    if !sent { return MbValue::none(); }
+    if !sent {
+        return MbValue::none();
+    }
 
     // Wait for resume signal from caller
     let msg = GEN_RX.with(|rx| {
@@ -710,10 +865,8 @@ pub fn mb_generator_yield_value(value: MbValue) -> MbValue {
 pub fn mb_generator_yield_from(sub_iter: MbValue) -> MbValue {
     // If sub_iter is a generator handle (int), delegate yield
     if sub_iter.is_int() {
-        // Check if it's a generator
-        let is_gen = GENERATORS.with(|gens| {
-            gens.borrow().contains_key(&(sub_iter.as_int().unwrap() as u64))
-        });
+        let is_gen = GENERATOR_REGISTRY
+            .contains_key(&(sub_iter.as_int().unwrap() as u64));
         if is_gen {
             return yield_from_generator(sub_iter);
         }
@@ -721,15 +874,21 @@ pub fn mb_generator_yield_from(sub_iter: MbValue) -> MbValue {
 
     // Otherwise, iterate over the iterable and yield each value
     let iter_handle = super::iter::mb_iter(sub_iter);
-    if iter_handle.is_none() { return MbValue::none(); }
+    if iter_handle.is_none() {
+        return MbValue::none();
+    }
 
     loop {
         let has = super::iter::mb_has_next(iter_handle);
-        if has.as_bool() == Some(false) { break; }
+        if has.as_bool() == Some(false) {
+            break;
+        }
         let val = super::iter::mb_next(iter_handle);
         if val.is_none() {
             let exhausted = super::iter::mb_has_next(iter_handle);
-            if exhausted.as_bool() == Some(false) { break; }
+            if exhausted.as_bool() == Some(false) {
+                break;
+            }
         }
         // Yield this value to our caller
         let _sent = mb_generator_yield_value(val);
@@ -748,9 +907,10 @@ fn yield_from_generator(sub_gen: MbValue) -> MbValue {
     loop {
         // Check if sub-generator is exhausted
         let exhausted = if let Some(id) = sub_gen.as_int() {
-            GENERATORS.with(|gens| {
-                gens.borrow().get(&(id as u64)).map(|g| g.exhausted).unwrap_or(true)
-            })
+            GENERATOR_REGISTRY
+                .get(&(id as u64))
+                .map(|e| e.exhausted)
+                .unwrap_or(true)
         } else {
             true
         };
@@ -758,9 +918,10 @@ fn yield_from_generator(sub_gen: MbValue) -> MbValue {
         if exhausted {
             // Sub-generator returned; get its return value
             let ret_val = if let Some(id) = sub_gen.as_int() {
-                GENERATORS.with(|gens| {
-                    gens.borrow().get(&(id as u64)).map(|g| g.return_value).unwrap_or(MbValue::none())
-                })
+                GENERATOR_REGISTRY
+                    .get(&(id as u64))
+                    .map(|e| e.return_value)
+                    .unwrap_or(MbValue::none())
             } else {
                 MbValue::none()
             };
@@ -782,7 +943,7 @@ fn yield_from_generator(sub_gen: MbValue) -> MbValue {
     }
 }
 
-// ── Helpers ──
+// ── Helpers ──────────────────────────────────────────────────────────────────
 
 /// Raise StopIteration with an optional return value.
 /// Sets both the iterator flag and the exception state so try/except works.
@@ -790,8 +951,6 @@ fn raise_stop_iteration(return_value: MbValue) {
     super::iter::signal_stop_iteration();
     LAST_STOP_VALUE.with(|v| v.set(return_value.to_bits()));
     // Raise as exception for try/except handling
-    // The StopIteration exception carries the value in its message field
-    // For StopIteration.value, we store it as an instance attribute
     let exc_type = MbValue::from_ptr(MbObject::new_str("StopIteration".to_string()));
     let exc_msg = MbValue::from_ptr(MbObject::new_str(String::new()));
     super::exception::mb_raise(exc_type, exc_msg);
@@ -840,4 +999,167 @@ mod tests {
         mb_generator_close(bad); // should not panic
         mb_generator_release(bad); // should not panic
     }
+
+    // ── S7/R4: Unique generator IDs across concurrent workers ───────────────
+
+    /// Spawn 10 threads each calling `alloc_gen_id()` 100 times; collect all
+    /// IDs and verify no duplicates.  Validates AtomicU64 counter correctness
+    /// under contention.
+    #[test]
+    fn test_unique_gen_ids_concurrent() {
+        let mut handles = Vec::new();
+        for _ in 0..10 {
+            handles.push(std::thread::spawn(|| {
+                let mut ids = Vec::with_capacity(100);
+                for _ in 0..100 {
+                    ids.push(alloc_gen_id());
+                }
+                ids
+            }));
+        }
+
+        let mut all_ids: Vec<u64> = Vec::new();
+        for h in handles {
+            all_ids.extend(h.join().expect("thread should not panic"));
+        }
+
+        let total = all_ids.len();
+        assert_eq!(total, 1000, "expected 1000 IDs from 10×100 threads");
+
+        // Check uniqueness
+        all_ids.sort();
+        all_ids.dedup();
+        assert_eq!(
+            all_ids.len(),
+            total,
+            "all generator IDs must be unique across concurrent threads"
+        );
+    }
+
+    // ── S5/R6: cleanup_all_generators() drains registry ─────────────────────
+
+    /// Create multiple generators, call `cleanup_all_generators()`, and verify
+    /// the global `GENERATOR_REGISTRY` is empty afterward.
+    #[test]
+    fn test_cleanup_drains_registry() {
+        // Create several generators (they won't have real body functions)
+        let mut gen_ids = Vec::new();
+        for i in 0..5 {
+            let name = MbValue::from_ptr(MbObject::new_str(format!("cleanup_gen_{i}")));
+            let body_fn = MbValue::none();
+            let gen = mb_generator_create(name, body_fn);
+            gen_ids.push(gen);
+        }
+
+        // Verify they exist in the registry
+        for gen in &gen_ids {
+            assert!(
+                is_known_generator(*gen),
+                "generator should be registered before cleanup"
+            );
+        }
+
+        // Cleanup should drain the registry
+        cleanup_all_generators();
+
+        // Verify registry is empty
+        assert!(
+            GENERATOR_REGISTRY.is_empty(),
+            "GENERATOR_REGISTRY should be empty after cleanup_all_generators()"
+        );
+
+        // All generators should appear exhausted (not found = exhausted)
+        for gen in &gen_ids {
+            assert_eq!(
+                mb_generator_is_exhausted(*gen).as_bool(),
+                Some(true),
+                "generator should report exhausted after cleanup"
+            );
+        }
+    }
+
+    // ── S6/R1: Lazy pool initialization ─────────────────────────────────────
+
+    /// Verify that creating generators initializes the pool (GEN_POOL becomes
+    /// Some).  This indirectly tests lazy init — the pool is created on demand.
+    #[test]
+    fn test_pool_initialized_after_generator_create() {
+        // Create a generator (triggers lazy pool init via get_pool_sender())
+        let name = MbValue::from_ptr(MbObject::new_str("pool_test".to_string()));
+        let body_fn = MbValue::none();
+        let gen = mb_generator_create(name, body_fn);
+
+        // Pool should be initialized now
+        let pool = GEN_POOL.lock().unwrap();
+        assert!(
+            pool.is_some(),
+            "GEN_POOL should be initialized after mb_generator_create()"
+        );
+        drop(pool);
+
+        // Cleanup
+        mb_generator_release(gen);
+        cleanup_all_generators();
+    }
+
+    /// Verify that `cleanup_all_generators()` does NOT destroy the pool —
+    /// it only barrier-syncs.  The pool remains alive for reuse.
+    #[test]
+    fn test_cleanup_preserves_pool() {
+        // Ensure pool is initialized
+        let name = MbValue::from_ptr(MbObject::new_str("pool_preserve".to_string()));
+        let body_fn = MbValue::none();
+        let gen = mb_generator_create(name, body_fn);
+        mb_generator_release(gen);
+
+        cleanup_all_generators();
+
+        // Pool should still be alive (Some)
+        let pool = GEN_POOL.lock().unwrap();
+        assert!(
+            pool.is_some(),
+            "GEN_POOL should remain alive after cleanup (barrier-only, no shutdown)"
+        );
+        drop(pool);
+    }
+
+    // ── R3: Global registry lookups from any thread ─────────────────────────
+
+    /// Create a generator on one thread and verify it's visible from another
+    /// thread via the global registry.
+    #[test]
+    fn test_global_registry_cross_thread_visibility() {
+        let name = MbValue::from_ptr(MbObject::new_str("cross_thread".to_string()));
+        let body_fn = MbValue::none();
+        let gen = mb_generator_create(name, body_fn);
+        let gen_bits = gen.to_bits();
+
+        let handle = std::thread::spawn(move || {
+            let gen_handle = MbValue::from_bits(gen_bits);
+            // Should be visible from another thread via global DashMap
+            is_known_generator(gen_handle)
+        });
+
+        let visible = handle.join().expect("thread should not panic");
+        assert!(
+            visible,
+            "generator created on main thread should be visible from worker thread"
+        );
+
+        mb_generator_release(gen);
+        cleanup_all_generators();
+    }
+
+    // ── R4: Atomic ID monotonicity ──────────────────────────────────────────
+
+    /// Verify that sequential `alloc_gen_id()` calls produce strictly
+    /// monotonically increasing IDs.
+    #[test]
+    fn test_gen_id_monotonically_increasing() {
+        let id1 = alloc_gen_id();
+        let id2 = alloc_gen_id();
+        let id3 = alloc_gen_id();
+        assert!(id1 < id2, "IDs should be strictly increasing: {id1} < {id2}");
+        assert!(id2 < id3, "IDs should be strictly increasing: {id2} < {id3}");
+    }
 }

```
