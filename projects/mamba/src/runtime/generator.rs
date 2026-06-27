/// Generator functions and yield for the Mamba runtime (#290, #1114, #1187).
///
/// Generators are implemented using same-thread stackful coroutines.  Each
/// generator gets a private stack allocated via `mmap`.  `next()`/`send()`
/// switches from the caller's stack to the generator's stack, and `yield`
/// switches back.  This eliminates **all** cross-thread synchronization,
/// reducing per-yield overhead from ~2 channel round-trips to a single
/// register-save + stack-pointer swap (~10 ns).
///
/// Architecture:
/// - Generator functions are compiled into a body function + constructor wrapper
/// - The constructor creates a generator object with a private coroutine stack
/// - On first `next()`, we switch to the coroutine stack and call the body fn
/// - `mb_generator_yield_value` saves registers and swaps back to the caller
/// - Subsequent `next()` calls restore the generator's registers and resume
/// - Output capture uses the caller thread's buffer directly (no Arc<Mutex>)
/// - Generator state lives in a thread-local HashMap (no DashMap needed)
/// - `cleanup_all_generators()` deallocates all coroutine stacks
use std::sync::atomic::{AtomicU64, Ordering};

use super::rc::{MbObject, ObjData};
use super::value::MbValue;

// ── Coroutine stack constants ───────────────────────────────────────────────

/// Default coroutine stack size: 64 KiB.  Sufficient for most generator
/// bodies.  The guard page adds one extra page of protection.
const CORO_STACK_SIZE: usize = 64 * 1024;

/// Page size for guard page allocation.
const PAGE_SIZE: usize = 16384; // 16 KiB on macOS aarch64

// ── Coroutine context ───────────────────────────────────────────────────────

/// Number of u64 register slots in the save area.
#[cfg(target_arch = "aarch64")]
const CORO_CTX_REGS: usize = 21; // x19-x28(10), x29, x30, SP, d8-d15(8)
#[cfg(target_arch = "x86_64")]
const CORO_CTX_REGS: usize = 8; // rbx, rbp, r12-r15, rsp, rip

/// Saved CPU context for a coroutine (callee-saved registers + SP + LR/RIP).
#[repr(C)]
struct CoroContext {
    regs: [u64; CORO_CTX_REGS],
}

impl CoroContext {
    fn new() -> Self {
        CoroContext {
            regs: [0u64; CORO_CTX_REGS],
        }
    }
}

/// A coroutine stack: mmap'd memory with a guard page at the bottom.
struct CoroStack {
    /// Base address of the mmap'd region (guard page is at base).
    base: *mut u8,
    /// Total mmap size (guard + usable stack).
    total_size: usize,
}

impl CoroStack {
    /// Allocate a new coroutine stack with a guard page.
    fn new(stack_size: usize) -> Self {
        let total = stack_size + PAGE_SIZE; // guard + stack
        unsafe {
            let base = libc::mmap(
                std::ptr::null_mut(),
                total,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            ) as *mut u8;
            assert!(!base.is_null(), "mmap failed for coroutine stack");

            // Guard page: no access at the bottom of the stack
            libc::mprotect(base as *mut libc::c_void, PAGE_SIZE, libc::PROT_NONE);

            CoroStack {
                base,
                total_size: total,
            }
        }
    }

    /// Top of the usable stack (stacks grow downward on both aarch64 and x86_64).
    /// Must be 16-byte aligned.
    fn top(&self) -> *mut u8 {
        unsafe {
            let top = self.base.add(self.total_size);
            // Ensure 16-byte alignment
            let aligned = (top as usize) & !0xF;
            aligned as *mut u8
        }
    }
}

impl Drop for CoroStack {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base as *mut libc::c_void, self.total_size);
        }
    }
}

// ── Platform-specific stack switching ───────────────────────────────────────

// swap_context is defined in pure assembly via global_asm! below.
// It has no compiler-generated prologue/epilogue, which is essential
// because it swaps the stack pointer — a normal function's epilogue
// would try to restore from the wrong stack.
//
// Export both C symbol spellings: Mach-O targets use a leading underscore for
// external symbols, while ELF targets do not.
//
// Signature: extern "C" fn(from: *mut CoroContext, to: *const CoroContext)
// - Saves all callee-saved registers into `from`
// - Restores all callee-saved registers from `to`
// - Returns to the restored LR/RIP (effectively "jumping" to where `to` was saved)

extern "C" {
    fn swap_context(from: *mut CoroContext, to: *const CoroContext);
}

#[cfg(target_arch = "aarch64")]
std::arch::global_asm!(
    ".global swap_context",
    ".global _swap_context",
    "swap_context:",
    "_swap_context:",
    // x0 = from, x1 = to
    // Save callee-saved GPRs into from
    "stp x19, x20, [x0]",
    "stp x21, x22, [x0, #16]",
    "stp x23, x24, [x0, #32]",
    "stp x25, x26, [x0, #48]",
    "stp x27, x28, [x0, #64]",
    "stp x29, x30, [x0, #80]",
    "mov x9, sp",
    "str x9,        [x0, #96]",
    // Save callee-saved SIMD regs
    "stp d8,  d9,  [x0, #104]",
    "stp d10, d11, [x0, #120]",
    "stp d12, d13, [x0, #136]",
    "stp d14, d15, [x0, #152]",
    // Restore from to
    "ldp x19, x20, [x1]",
    "ldp x21, x22, [x1, #16]",
    "ldp x23, x24, [x1, #32]",
    "ldp x25, x26, [x1, #48]",
    "ldp x27, x28, [x1, #64]",
    "ldp x29, x30, [x1, #80]",
    "ldr x9,        [x1, #96]",
    "mov sp, x9",
    "ldp d8,  d9,  [x1, #104]",
    "ldp d10, d11, [x1, #120]",
    "ldp d12, d13, [x1, #136]",
    "ldp d14, d15, [x1, #152]",
    // Return to restored LR (x30)
    "ret",
);

#[cfg(target_arch = "x86_64")]
std::arch::global_asm!(
    ".global swap_context",
    ".global _swap_context",
    "swap_context:",
    "_swap_context:",
    // rdi = from, rsi = to
    // Save callee-saved registers into from
    "mov [rdi],      rbx",
    "mov [rdi + 8],  rbp",
    "mov [rdi + 16], r12",
    "mov [rdi + 24], r13",
    "mov [rdi + 32], r14",
    "mov [rdi + 40], r15",
    "mov [rdi + 48], rsp",
    // Save return address
    "lea rax, [rip + .Lswap_resume]",
    "mov [rdi + 56], rax",
    // Restore from to
    "mov rbx, [rsi]",
    "mov rbp, [rsi + 8]",
    "mov r12, [rsi + 16]",
    "mov r13, [rsi + 24]",
    "mov r14, [rsi + 32]",
    "mov r15, [rsi + 40]",
    "mov rsp, [rsi + 48]",
    // Jump to restored rip
    "jmp [rsi + 56]",
    ".Lswap_resume:",
    "ret",
);

// ── Generator entry ─────────────────────────────────────────────────────────

/// Generator state.
enum GenState {
    /// Created but not yet started (first `next()` not called).
    Created,
    /// Suspended at a yield point.
    Suspended,
    /// Body has returned (exhausted).
    Completed,
}

/// A single generator instance.
struct GenEntry {
    /// Coroutine context (saved registers when suspended).
    /// Box-allocated so its address is stable even when the HashMap resizes.
    /// This is critical because we pass raw pointers to swap_context.
    coro_ctx: Box<CoroContext>,
    /// Coroutine stack.
    coro_stack: CoroStack,
    /// Current state.
    state: GenState,
    /// Body function address (NaN-boxed pointer bits).
    body_fn_addr: u64,
    /// Captured arguments.
    args: Vec<MbValue>,
    /// Argument names in declaration order for inspect.getgeneratorlocals.
    arg_names: Vec<String>,
    /// Name of a yielded local value when lowering can identify `yield <var>`.
    yield_local_name: Option<String>,
    /// Snapshot of generator locals visible to inspect.getgeneratorlocals.
    locals: HashMap<String, MbValue>,
    /// Name for debugging.
    #[allow(dead_code)]
    name: String,
    /// Value passed from yield to caller (set by yield, read by next).
    #[allow(dead_code)]
    yielded_value: MbValue,
    /// Value passed from caller to generator (set by send, read after yield).
    #[allow(dead_code)]
    sent_value: MbValue,
    /// Return value (set when body returns).
    return_value: MbValue,
    /// Throw request: exception type + message.
    throw_request: Option<(String, String)>,
    /// Close request flag.
    close_request: bool,
}

// ── Caller context stack ────────────────────────────────────────────────────

/// Maximum nesting depth for generators (yield from chains).
const MAX_GEN_NESTING: usize = 16;

/// Pre-allocated stack of caller CoroContexts.  Avoids heap allocation
/// on every `resume_generator` call.  Each slot is its own allocation so
/// the pointer remains stable even when the depth counter changes.
struct CallerCtxStack {
    slots: [std::cell::UnsafeCell<CoroContext>; MAX_GEN_NESTING],
    depth: std::cell::Cell<usize>,
}

// SAFETY: CallerCtxStack is only used from thread-locals (single-threaded access).
unsafe impl Sync for CallerCtxStack {}

impl CallerCtxStack {
    const fn new() -> Self {
        CallerCtxStack {
            slots: [const {
                std::cell::UnsafeCell::new(CoroContext {
                    regs: [0u64; CORO_CTX_REGS],
                })
            }; MAX_GEN_NESTING],
            depth: std::cell::Cell::new(0),
        }
    }

    /// Push a new context and return a raw pointer to it.
    fn push(&self) -> *mut CoroContext {
        let d = self.depth.get();
        assert!(
            d < MAX_GEN_NESTING,
            "generator nesting too deep (max {MAX_GEN_NESTING})"
        );
        self.depth.set(d + 1);
        self.slots[d].get()
    }

    /// Pop the top context.
    fn pop(&self) {
        let d = self.depth.get();
        debug_assert!(d > 0, "CallerCtxStack underflow");
        self.depth.set(d - 1);
    }

    /// Reset the caller stack after discarding all suspended generators.
    fn reset(&self) {
        self.depth.set(0);
    }

    /// Get the top context pointer (must have at least one entry).
    fn top(&self) -> *mut CoroContext {
        let d = self.depth.get();
        debug_assert!(d > 0, "CallerCtxStack::top on empty stack");
        self.slots[d - 1].get()
    }
}

// ── Thread-local state ──────────────────────────────────────────────────────

use std::collections::HashMap;

/// Bundled hot-path value-transfer cells. See `GEN_XFER` doc.
struct GenXfer {
    yield_v: std::cell::Cell<u64>,
    completion: std::cell::Cell<u64>,
    send: std::cell::Cell<u64>,
    throw: std::cell::Cell<u64>,
}

/// Bundled "active generator" state. Co-locating these four Cells in one
/// thread_local lets the resume / yield hot paths fold ~4 TLV-stub macOS
/// arm64 lookups per round-trip into a single `GEN_ACTIVE.with(...)` —
/// each .with on Apple silicon dispatches via dyld's TLV thunk.
struct GenActive {
    /// Currently executing generator ID (set before swap_into-generator).
    active_id: std::cell::Cell<Option<u64>>,
    /// Cached coro_ctx pointer for the active generator (avoids HashMap
    /// lookup in the yield hot path).
    active_ctx: std::cell::Cell<*mut CoroContext>,
    /// Resume-cache id: bench hot path resumes the same generator
    /// repeatedly, so this cache lets the second-and-subsequent calls
    /// skip the GENERATORS HashMap lookup entirely. `u64::MAX` is the
    /// empty sentinel (NEXT_GEN_ID starts at 1, never reaches MAX).
    /// Invalidated on completion, throw, close, or runtime reset.
    last_resumed_id: std::cell::Cell<u64>,
    /// Resume-cache coro_ctx ptr — paired with last_resumed_id.
    last_resumed_ctx: std::cell::Cell<*mut CoroContext>,
}

thread_local! {
    /// Generator registry (thread-local — generators run on their creator's thread).
    static GENERATORS: std::cell::RefCell<HashMap<u64, GenEntry>> =
        std::cell::RefCell::new(HashMap::new());

    /// Bundled active-generator + resume-cache cells (see GenActive).
    static GEN_ACTIVE: GenActive = GenActive {
        active_id: std::cell::Cell::new(None),
        active_ctx: std::cell::Cell::new(std::ptr::null_mut()),
        last_resumed_id: std::cell::Cell::new(u64::MAX),
        last_resumed_ctx: std::cell::Cell::new(std::ptr::null_mut()),
    };

    /// Fast value-transfer cells between yield and resume. Bundled into a
    /// single TLS to amortize the per-`with()` TLV stub cost on macOS arm64.
    /// The bench resumes 10000× per iter so each saved TLS lookup matters.
    /// - `yield_v`: set by yield_value, read by resume_generator post-swap.
    /// - `completion`: 0 = yielded, non-zero = trampoline-returned bits.
    ///   Valid MbValues always carry NAN_PREFIX so 0 is a reliable sentinel.
    /// - `send`: set by resume_generator, read by yield_value post-swap.
    /// - `throw`: 0 = none, 1 = close, 2+ = ptr to Box<(String,String)>.
    static GEN_XFER: GenXfer = GenXfer {
        yield_v: std::cell::Cell::new(0),
        completion: std::cell::Cell::new(0),
        send: std::cell::Cell::new(0),
        throw: std::cell::Cell::new(0),
    };

    /// Stack of caller contexts — supports nested generator resumption.
    /// Pre-allocated fixed-capacity array to avoid heap allocation per resume.
    /// Max nesting depth: 16 (yield from chains deeper than this will panic).
    static CALLER_CTX_STACK: CallerCtxStack = CallerCtxStack::new();

    /// StopIteration return value.
    static LAST_STOP_VALUE: std::cell::Cell<u64> = std::cell::Cell::new(MbValue::none().to_bits());
}

/// Atomic generator ID counter (global, for uniqueness across threads).
static NEXT_GEN_ID: AtomicU64 = AtomicU64::new(1);

fn alloc_gen_id() -> u64 {
    NEXT_GEN_ID.fetch_add(1, Ordering::Relaxed)
}

// ── Shared output capture (kept for compatibility with test infrastructure) ──

use std::sync::{Arc, Mutex};

thread_local! {
    static SHARED_CAPTURE: std::cell::RefCell<Option<Arc<Mutex<Vec<u8>>>>> =
        std::cell::RefCell::new(None);
}

pub fn activate_shared_capture() -> Option<Arc<Mutex<Vec<u8>>>> {
    let existing = SHARED_CAPTURE.with(|sc| sc.borrow().clone());
    if let Some(buf) = existing {
        return Some(buf);
    }
    if super::output::is_capturing() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        SHARED_CAPTURE.with(|sc| *sc.borrow_mut() = Some(buf.clone()));
        Some(buf)
    } else {
        None
    }
}

pub fn write_shared_capture(s: &str) -> bool {
    SHARED_CAPTURE.with(|sc| {
        if let Some(ref buf) = *sc.borrow() {
            use std::io::Write;
            let mut guard = buf.lock().unwrap();
            let _ = guard.write_all(s.as_bytes());
            true
        } else {
            false
        }
    })
}

pub fn flush_shared_capture() {
    SHARED_CAPTURE.with(|sc| {
        if let Some(ref buf) = *sc.borrow() {
            let data = {
                let mut guard = buf.lock().unwrap();
                std::mem::take(&mut *guard)
            };
            if !data.is_empty() {
                let s = String::from_utf8_lossy(&data);
                super::output::write_captured(&s);
            }
        }
    });
}

// ── Generator Creation ──────────────────────────────────────────────────────

/// Create a new generator. Called from compiled constructor wrapper.
pub fn mb_generator_create(name: MbValue, body_fn_addr: MbValue) -> MbValue {
    let gen_name = extract_str(name).unwrap_or_else(|| "<generator>".to_string());
    let fn_addr = body_fn_addr.to_bits();
    let id = alloc_gen_id();

    let entry = GenEntry {
        coro_ctx: Box::new(CoroContext::new()),
        coro_stack: CoroStack::new(CORO_STACK_SIZE),
        state: GenState::Created,
        body_fn_addr: fn_addr,
        args: Vec::new(),
        arg_names: Vec::new(),
        yield_local_name: None,
        locals: HashMap::new(),
        name: gen_name,
        yielded_value: MbValue::none(),
        sent_value: MbValue::none(),
        return_value: MbValue::none(),
        throw_request: None,
        close_request: false,
    };

    GENERATORS.with(|gens| gens.borrow_mut().insert(id, entry));
    let handle = MbValue::from_int(id as i64);
    // Register handle as its own iterator so `iter(g) is g` (CPython
    // contract). The ITERATORS-keyed entry is what every dispatcher
    // already understands — peek-ahead, exhaustion latching, etc.
    super::iter::register_generator_iter(handle);
    handle
}

/// Store an argument for the generator.
pub fn mb_generator_store_arg(gen_handle: MbValue, arg: MbValue) {
    if let Some(id) = gen_handle.as_int() {
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&(id as u64)) {
                let idx = entry.args.len();
                entry.args.push(arg);
                if let Some(name) = entry.arg_names.get(idx).cloned() {
                    entry.locals.insert(name, arg);
                }
            }
        });
    }
}

/// Register a generator's parameter names and an optional yielded-local name.
///
/// The metadata list is `[arg_name..., yielded_local_or_None]`.
pub fn mb_generator_set_local_names(gen_handle: MbValue, names: MbValue) {
    let Some(id) = gen_handle.as_int() else {
        return;
    };
    let mut items: Vec<MbValue> = Vec::new();
    if let Some(ptr) = names.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                items = lock.read().unwrap().to_vec();
            }
        }
    }
    let mut arg_names = Vec::new();
    let mut yield_local_name = None;
    for (idx, item) in items.iter().copied().enumerate() {
        if idx + 1 == items.len() {
            yield_local_name = extract_str(item);
        } else if let Some(name) = extract_str(item) {
            arg_names.push(name);
        }
    }
    GENERATORS.with(|gens| {
        if let Some(entry) = gens.borrow_mut().get_mut(&(id as u64)) {
            entry.arg_names = arg_names;
            entry.yield_local_name = yield_local_name;
            let pairs: Vec<(String, MbValue)> = entry
                .arg_names
                .iter()
                .cloned()
                .zip(entry.args.iter().copied())
                .collect();
            for (name, value) in pairs {
                entry.locals.insert(name, value);
            }
        }
    });
}

/// Snapshot live locals for inspect.getgeneratorlocals.
pub fn mb_generator_locals(gen_handle: MbValue) -> Option<Vec<(String, MbValue)>> {
    let id = gen_handle.as_int()? as u64;
    GENERATORS.with(|gens| {
        let gens = gens.borrow();
        let entry = gens.get(&id)?;
        if matches!(entry.state, GenState::Completed) {
            return Some(Vec::new());
        }
        let mut out = Vec::new();
        for name in &entry.arg_names {
            if let Some(value) = entry.locals.get(name).copied() {
                out.push((name.clone(), value));
            }
        }
        if let Some(name) = &entry.yield_local_name {
            if !entry.arg_names.iter().any(|arg| arg == name) {
                if let Some(value) = entry.locals.get(name).copied() {
                    out.push((name.clone(), value));
                }
            }
        }
        Some(out)
    })
}

// ── Coroutine entry trampoline ──────────────────────────────────────────────

/// Trampoline function that runs on the generator's coroutine stack.
/// Reads the generator ID from ACTIVE_GEN_ID (set before the swap into this
/// coroutine), calls the compiled body, then swaps back to the caller.
extern "C" fn gen_trampoline() {
    let gen_id = GEN_ACTIVE
        .with(|a| a.active_id.get())
        .expect("gen_trampoline: ACTIVE_GEN_ID must be set");

    // Extract body fn addr and args from the registry.
    // Borrow is released before calling the body.
    let (body_fn_addr, args) = GENERATORS.with(|gens| {
        let gens = gens.borrow();
        let entry = gens.get(&gen_id).expect("generator must exist");
        (entry.body_fn_addr, entry.args.clone())
    });

    // Call the compiled body function (may yield many times via
    // mb_generator_yield_value, each of which does a swap_context)
    let return_value = call_body_fn(body_fn_addr, &args);

    // Body returned — mark as completed (state is read by other APIs:
    // mb_generator_close, mb_iter_next on a generator handle, etc.).
    GENERATORS.with(|gens| {
        if let Some(entry) = gens.borrow_mut().get_mut(&gen_id) {
            entry.state = GenState::Completed;
            entry.return_value = return_value;
        }
    });
    // Hot-path signal to resume_generator: yielded vs completed. 0 is
    // reserved (NaN-boxed values always set NAN_PREFIX); set bits
    // unconditionally — caller will treat any non-zero as "completed".
    GEN_XFER.with(|x| x.completion.set(return_value.to_bits()));

    // Switch back to caller. ACTIVE_GEN_CTX was cached by resume_generator
    // before the initial swap, so we don't need a HashMap lookup here.
    unsafe {
        let gen_ctx_ptr = GEN_ACTIVE.with(|a| a.active_ctx.get());
        let caller_ctx = CALLER_CTX_STACK.with(|stack| stack.top());
        swap_context(gen_ctx_ptr, caller_ctx);
    }
    // swap_context never returns here because the generator is completed
    // and won't be resumed — the caller reads Completed state.
    unreachable!("generator trampoline should not be reached after completion");
}

/// Initialize the coroutine context for first execution.
///
/// Sets up the saved registers so that the first `swap_context` into this
/// coroutine effectively "returns into" `gen_trampoline()`.
///
/// On aarch64: set LR (x30) = trampoline, SP = coroutine stack top.
/// On x86_64:  set RIP = trampoline, RSP = coroutine stack top.
fn init_coro_context(ctx: &mut CoroContext, stack_top: *mut u8) {
    #[cfg(target_arch = "aarch64")]
    {
        // regs layout: [x19..x28 (10), x29, x30, sp, d8..d15 (8)]
        //               idx 0..9       10    11  12   13..20
        ctx.regs[10] = 0; // x29 (FP) = 0 (base frame)
        ctx.regs[11] = gen_trampoline as *const () as u64; // x30 (LR) — swap_context does `ret` which jumps here
                                                           // SP: 16-byte aligned, leave space for a minimal frame
        ctx.regs[12] = (stack_top as u64) & !0xF;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // regs layout: [rbx, rbp, r12, r13, r14, r15, rsp, rip]
        //               0     1    2     3    4    5    6    7
        ctx.regs[1] = 0; // rbp = 0 (base frame)
        ctx.regs[6] = (stack_top as u64) & !0xF; // rsp (16-byte aligned)
        ctx.regs[7] = gen_trampoline as *const () as u64; // rip — swap_context jumps here
    }
}

// ── Resume / Yield ──────────────────────────────────────────────────────────

/// Resume a generator: switch from caller to generator coroutine.
/// Returns the yielded value (or None + StopIteration on exhaustion).
///
/// Optimized hot path: single GENERATORS borrow for check+prepare+get-ptr,
/// then swap, then single borrow for read-result.
fn resume_generator(id: u64, send_value: MbValue) -> MbValue {
    // Fast path: cache hit means this id was previously prepped and hasn't
    // completed (completion / throw / close all bust the cache). State is
    // therefore Suspended and throw_request is None — skip the HashMap
    // lookup entirely. Single TLS hit reads both id + ctx.
    let cached_ctx = GEN_ACTIVE.with(|a| {
        if a.last_resumed_id.get() == id {
            Some(a.last_resumed_ctx.get())
        } else {
            None
        }
    });

    let (gen_ctx_ptr, has_throw) = if let Some(ctx_ptr) = cached_ctx {
        (ctx_ptr, false)
    } else {
        // Slow path: full borrow, check state, init if Created.
        let prep = GENERATORS.with(|gens| {
            let mut gens = gens.borrow_mut();
            let entry = match gens.get_mut(&id) {
                Some(e) => e,
                None => return None,
            };
            match entry.state {
                GenState::Completed => return None,
                GenState::Created => {
                    let stack_top = entry.coro_stack.top();
                    init_coro_context(&mut *entry.coro_ctx, stack_top);
                    entry.state = GenState::Suspended;
                }
                GenState::Suspended => {}
            }
            let has_throw = entry.throw_request.is_some();
            let ctx_ptr = &*entry.coro_ctx as *const CoroContext as *mut CoroContext;
            Some((ctx_ptr, has_throw))
        });

        match prep {
            Some(p) => {
                // Populate cache only when no throw is pending — throw paths
                // need to re-read entry.throw_request on the next resume.
                if !p.1 {
                    GEN_ACTIVE.with(|a| {
                        a.last_resumed_id.set(id);
                        a.last_resumed_ctx.set(p.0);
                    });
                }
                p
            }
            None => {
                raise_stop_iteration(MbValue::none());
                return MbValue::none();
            }
        }
    };

    // Clear stale exceptions (unless a throw is pending)
    if !has_throw {
        super::exception::clear_current_exception();
    }

    // Set up fast-path value transfer + clear completion (one TLS hit).
    GEN_XFER.with(|x| {
        x.send.set(send_value.to_bits());
        x.completion.set(0);
    });

    // Set active generator ID + coro_ctx ptr in one TLS hit; capture
    // previous values for nested-resume restore on the way back.
    let (prev_active, prev_ctx) = GEN_ACTIVE.with(|a| {
        let pa = a.active_id.replace(Some(id));
        let pc = a.active_ctx.replace(gen_ctx_ptr);
        (pa, pc)
    });

    // Push caller context and swap
    let caller_ctx_ptr = CALLER_CTX_STACK.with(|stack| stack.push());

    // === SWAP: caller → generator ===
    unsafe {
        swap_context(caller_ctx_ptr, gen_ctx_ptr);
    }
    // === SWAP BACK: generator yielded or completed ===

    CALLER_CTX_STACK.with(|stack| stack.pop());
    GEN_ACTIVE.with(|a| {
        a.active_id.set(prev_active);
        a.active_ctx.set(prev_ctx);
    });

    // Hot path: yield vs completion is signaled via x.completion.
    // No HashMap lookup needed — the trampoline already wrote the return
    // value bits there before its final swap. 0 means "yielded".
    // Read both completion + yield in a single TLS hit so the common-case
    // yield branch doesn't pay for two `with()` calls.
    let (completion_bits, yield_bits) = GEN_XFER.with(|x| {
        let c = x.completion.get();
        x.completion.set(0);
        (c, x.yield_v.get())
    });
    if completion_bits != 0 {
        // Generator completed — bust the resume cache so a future resume
        // on this id (e.g. user calling next() on exhausted gen) goes
        // through the slow path and observes GenState::Completed.
        GEN_ACTIVE.with(|a| {
            if a.last_resumed_id.get() == id {
                a.last_resumed_id.set(u64::MAX);
                a.last_resumed_ctx.set(std::ptr::null_mut());
            }
        });
        // Preserve any exception the generator body unwound with — only
        // synthesize StopIteration when the body returned cleanly without
        // a pending exception. Otherwise an uncaught body exception (e.g.
        // a thrown exception that the body did not catch) would be
        // overwritten by the natural-completion StopIteration, and
        // mb_generator_throw / yield-from would lose the original.
        if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            // PEP 479: StopIteration escaping from inside a generator body
            // becomes RuntimeError with the original exception as __cause__.
            let cause = super::exception::mb_catch_exception();
            super::iter::check_and_clear_stop();
            super::exception::mb_raise_from(
                MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "generator raised StopIteration".to_string(),
                )),
                cause,
            );
        } else if !super::exception::mb_has_exception()
            .as_bool()
            .unwrap_or(false)
        {
            raise_stop_iteration(MbValue::from_bits(completion_bits));
        }
        MbValue::none()
    } else {
        MbValue::from_bits(yield_bits)
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Advance the generator (next()). Returns the yielded value.
pub fn mb_generator_next(gen_handle: MbValue) -> MbValue {
    mb_generator_send(gen_handle, MbValue::none())
}

/// Send a value into the generator and advance to the next yield.
pub fn mb_generator_send(gen_handle: MbValue, value: MbValue) -> MbValue {
    if let Some(id) = gen_handle.as_int() {
        let id = id as u64;

        // Hot path: next() / send(None) skip the pre-resume state check entirely.
        // resume_generator's prep block already handles Completed/missing by
        // raising StopIteration, so the extra GENERATORS borrow is only needed
        // when we have to detect the CPython-specific TypeError for
        // send(non-None) on a just-started generator.
        if !value.is_none() {
            let state = GENERATORS.with(|gens| {
                let gens = gens.borrow();
                gens.get(&id).map(|e| match e.state {
                    GenState::Created => 0u8,
                    GenState::Suspended => 1,
                    GenState::Completed => 2,
                })
            });

            match state {
                Some(2) | None => {
                    raise_stop_iteration(MbValue::none());
                    return MbValue::none();
                }
                Some(0) => {
                    // CPython: send(non-None) to just-started generator raises TypeError
                    let exc_type = MbValue::from_ptr(MbObject::new_str("TypeError".to_string()));
                    let exc_msg = MbValue::from_ptr(MbObject::new_str(
                        "can't send non-None value to a just-started generator".to_string(),
                    ));
                    super::exception::mb_raise(exc_type, exc_msg);
                    return MbValue::none();
                }
                _ => {}
            }
        }

        resume_generator(id, value)
    } else {
        MbValue::none()
    }
}

/// Get the StopIteration return value from the last exhausted generator.
pub fn mb_generator_stop_value() -> MbValue {
    MbValue::from_bits(LAST_STOP_VALUE.with(|v| v.get()))
}

/// Throw an exception into the generator.
pub fn mb_generator_throw(gen_handle: MbValue, exc_type: MbValue, exc_msg: MbValue) -> MbValue {
    if let Some(id) = gen_handle.as_int() {
        let id = id as u64;

        let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
        let msg = extract_str(exc_msg).unwrap_or_default();

        let state = GENERATORS.with(|gens| {
            let gens = gens.borrow();
            gens.get(&id).map(|e| match e.state {
                GenState::Created => 0u8,
                GenState::Suspended => 1,
                GenState::Completed => 2,
            })
        });

        match state {
            Some(2) | None => {
                // CPython: throw on exhausted generator raises the thrown exception
                super::exception::set_current_exception(super::exception::MbException::new(
                    &type_name, &msg,
                ));
                return MbValue::none();
            }
            Some(0) => {
                // Throw before first yield — mark completed and raise
                GENERATORS.with(|gens| {
                    if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                        entry.state = GenState::Completed;
                    }
                });
                super::exception::set_current_exception(super::exception::MbException::new(
                    &type_name, &msg,
                ));
                return MbValue::none();
            }
            _ => {}
        }

        // Set throw request via fast-path transfer AND in the entry (for has_throw check)
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                entry.throw_request = Some((type_name.clone(), msg.clone()));
            }
        });
        // Also set throw xfer so yield_value picks it up.
        let throw_data = Box::new((type_name.clone(), msg.clone()));
        GEN_XFER.with(|x| x.throw.set(Box::into_raw(throw_data) as u64));

        // Bust resume cache so resume_generator re-reads has_throw.
        GEN_ACTIVE.with(|a| {
            if a.last_resumed_id.get() == id {
                a.last_resumed_id.set(u64::MAX);
                a.last_resumed_ctx.set(std::ptr::null_mut());
            }
        });

        // Resume generator — yield_value will see THROW_XFER
        let result = resume_generator(id, MbValue::none());

        // Clear the throw_request from entry (may have been consumed by yield_value
        // or may still be there if generator completed without yielding)
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                entry.throw_request = None;
            }
        });

        // Check if generator completed (exception not caught)
        let completed = GENERATORS.with(|gens| {
            gens.borrow()
                .get(&id)
                .map(|e| matches!(e.state, GenState::Completed))
                .unwrap_or(true)
        });

        if completed {
            // resume_generator preserved any exception the body unwound
            // with: if the body caught the throw and ran to completion,
            // it set StopIteration; if the body didn't catch, the original
            // exception is still pending. Either is the correct value to
            // surface to the caller of throw(). Defensive fallback only
            // when no exception is pending (shouldn't happen post-fix).
            if !super::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false)
            {
                super::exception::set_current_exception(super::exception::MbException::new(
                    &type_name, &msg,
                ));
            }
            MbValue::none()
        } else {
            result
        }
    } else {
        MbValue::none()
    }
}

/// Close the generator.
pub fn mb_generator_close(gen_handle: MbValue) {
    if let Some(id) = gen_handle.as_int() {
        let id = id as u64;

        let state = GENERATORS.with(|gens| {
            let gens = gens.borrow();
            gens.get(&id).map(|e| match e.state {
                GenState::Created => 0u8,
                GenState::Suspended => 1,
                GenState::Completed => 2,
            })
        });

        match state {
            Some(2) | None => return, // Already exhausted
            Some(0) => {
                // Not started — just mark completed
                GENERATORS.with(|gens| {
                    if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                        entry.state = GenState::Completed;
                    }
                });
                return;
            }
            _ => {}
        }

        // Set close request via fast-path AND in entry
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                entry.close_request = true;
            }
        });
        GEN_XFER.with(|x| x.throw.set(1)); // 1 = close

        // Bust resume cache — gen will complete during this resume, and
        // close() bypasses the normal Suspended-resume path.
        GEN_ACTIVE.with(|a| {
            if a.last_resumed_id.get() == id {
                a.last_resumed_id.set(u64::MAX);
                a.last_resumed_ctx.set(std::ptr::null_mut());
            }
        });

        // Resume — yield_value will see THROW_XFER=1 (close) and raise GeneratorExit
        let result = resume_generator(id, MbValue::none());

        // Check if generator yielded despite GeneratorExit (illegal)
        let completed = GENERATORS.with(|gens| {
            gens.borrow()
                .get(&id)
                .map(|e| matches!(e.state, GenState::Completed))
                .unwrap_or(true)
        });

        if !completed && !result.is_none() {
            // Generator ignored GeneratorExit and yielded again
            GENERATORS.with(|gens| {
                if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                    entry.state = GenState::Completed;
                }
            });
            super::exception::set_current_exception(super::exception::MbException::new(
                "RuntimeError",
                "generator ignored GeneratorExit",
            ));
            return;
        }

        // Mark completed
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                entry.state = GenState::Completed;
            }
        });

        // Per CPython, generator.close() never propagates StopIteration
        // (synthesized by resume_generator on clean completion) nor the
        // GeneratorExit that close() injected (preserved by
        // resume_generator if the body let it unwind). Both must surface
        // as `None` — drop either if pending so the caller does not see
        // a phantom uncaught exception.
        let pending = super::exception::mb_get_exception();
        if !pending.is_none() {
            if let Some(ptr) = pending.as_ptr() {
                let is_termination = unsafe {
                    matches!(
                        &(*ptr).data,
                        super::rc::ObjData::Instance { class_name, .. }
                            if class_name == "StopIteration"
                                || class_name == "GeneratorExit"
                    )
                };
                if is_termination {
                    super::exception::mb_clear_exception();
                }
            }
        }
    }
}

/// Check if a generator is exhausted.
pub fn mb_generator_is_exhausted(gen_handle: MbValue) -> MbValue {
    if let Some(id) = gen_handle.as_int() {
        GENERATORS.with(|gens| {
            gens.borrow()
                .get(&(id as u64))
                .map(|e| MbValue::from_bool(matches!(e.state, GenState::Completed)))
                .unwrap_or_else(|| MbValue::from_bool(true))
        })
    } else {
        MbValue::from_bool(true)
    }
}

/// Check if a value is a known generator handle.
pub fn is_known_generator(gen_handle: MbValue) -> bool {
    if let Some(id) = gen_handle.as_int() {
        GENERATORS.with(|gens| gens.borrow().contains_key(&(id as u64)))
    } else {
        false
    }
}

/// Delete a variable: close if it's a generator, then release heap memory.
///
/// Called by the JIT'd `del var` lowering.  Generator handles are stored as
/// plain integers (NaN-boxed), so `release_if_ptr` would be a no-op for them;
/// we must explicitly call `mb_generator_close` before releasing.
pub fn mb_del_var(val: MbValue) {
    if is_known_generator(val) {
        mb_generator_close(val);
    }
    // Release heap-allocated objects (no-op for integers / generator handles).
    unsafe { super::rc::release_if_ptr(val) };
}

/// Release a generator's resources.
pub fn mb_generator_release(gen_handle: MbValue) {
    if let Some(id) = gen_handle.as_int() {
        let id = id as u64;
        // Bust resume cache before dropping the entry — the cached
        // ctx_ptr would otherwise dangle.
        GEN_ACTIVE.with(|a| {
            if a.last_resumed_id.get() == id {
                a.last_resumed_id.set(u64::MAX);
                a.last_resumed_ctx.set(std::ptr::null_mut());
            }
        });
        GENERATORS.with(|gens| gens.borrow_mut().remove(&id));
        super::iter::unregister_generator_iter(gen_handle);
    }
}

/// Clean up all generators: close active ones (triggering finally blocks),
/// then deallocate all coroutine stacks.
/// Must be called before JIT code memory is freed.
pub fn cleanup_all_generators() {
    // Collect IDs of generators that need closing (started but not completed).
    let ids: Vec<u64> = GENERATORS.with(|gens| {
        gens.borrow()
            .iter()
            .filter(|(_, e)| matches!(e.state, GenState::Suspended))
            .map(|(id, _)| *id)
            .collect()
    });

    // Close each active generator (triggers finally blocks).
    for id in ids {
        mb_generator_close(MbValue::from_int(id as i64));
    }

    // Collect all remaining gen IDs to clean their ITERATORS entries before
    // dropping the registry.
    let remaining_ids: Vec<u64> = GENERATORS.with(|gens| gens.borrow().keys().copied().collect());
    for id in remaining_ids {
        super::iter::unregister_generator_iter(MbValue::from_int(id as i64));
    }
    // Now drain the registry (drops all coroutine stacks).
    GENERATORS.with(|gens| gens.borrow_mut().clear());
}

/// Discard generator runtime state during process/test teardown without
/// resuming suspended coroutine bodies.
///
/// `cleanup_all_generators()` preserves Python close/finally semantics by
/// resuming suspended generators. The centralized shutdown path cannot safely
/// do that after partially clearing runtime state, so it drops the registry and
/// coroutine stacks directly. Values captured on those stacks are intentionally
/// leaked at the MbValue layer; the mapped stack memory is retired here.
pub(crate) fn cleanup_generator_state_for_runtime_reset() {
    let stale_ids: Vec<u64> = GENERATORS.with(|gens| {
        gens.try_borrow()
            .map(|g| g.keys().copied().collect())
            .unwrap_or_default()
    });
    for id in stale_ids {
        super::iter::unregister_generator_iter(MbValue::from_int(id as i64));
    }
    let _ = GENERATORS.with(|gens| gens.try_borrow_mut().map(|mut gens| gens.clear()));
    GEN_ACTIVE.with(|a| {
        a.active_id.set(None);
        a.active_ctx.set(std::ptr::null_mut());
        a.last_resumed_id.set(u64::MAX);
        a.last_resumed_ctx.set(std::ptr::null_mut());
    });
    GEN_XFER.with(|x| {
        x.yield_v.set(0);
        x.send.set(0);
        x.throw.set(0);
        x.completion.set(0);
    });
    CALLER_CTX_STACK.with(|stack| stack.reset());
    LAST_STOP_VALUE.with(|cell| cell.set(MbValue::none().to_bits()));
}

/// Shut down the pool (no-op in coroutine mode — kept for API compatibility).
#[allow(dead_code)]
pub fn shutdown_pool() {
    // No thread pool in coroutine mode
}

// ── Called from compiled generator body code ────────────────────────────────

/// Yield a value from the generator body. Called from compiled code.
/// Saves generator context and switches back to caller.
/// Returns the value passed by send() (or None for plain next()).
pub fn mb_generator_yield_value(value: MbValue) -> MbValue {
    if let Some(id) = GEN_ACTIVE.with(|a| a.active_id.get()) {
        GENERATORS.with(|gens| {
            if let Some(entry) = gens.borrow_mut().get_mut(&id) {
                if let Some(name) = entry.yield_local_name.clone() {
                    entry.locals.insert(name, value);
                }
            }
        });
    }

    // Hot path: transfer value via thread-local cell + use cached ctx pointer.
    // No HashMap lookup, no RefCell borrow.
    GEN_XFER.with(|x| x.yield_v.set(value.to_bits()));

    let gen_ctx_ptr = GEN_ACTIVE.with(|a| a.active_ctx.get());

    // Switch back to caller.
    unsafe {
        let caller_ctx = CALLER_CTX_STACK.with(|stack| stack.top());
        swap_context(gen_ctx_ptr, caller_ctx);
    }

    // We're back. Read throw + sent in one TLS hit (common path: no throw).
    let (throw_code, sent_bits) = GEN_XFER.with(|x| (x.throw.get(), x.send.get()));

    let (throw_req, close_req, sent_value) = if throw_code == 0 {
        // Common case: no throw, no close.
        (None, false, MbValue::from_bits(sent_bits))
    } else if throw_code == 1 {
        // Close request
        GEN_XFER.with(|x| x.throw.set(0));
        (None, true, MbValue::none())
    } else {
        // Throw request: throw_code is a pointer to Box<(String, String)>
        GEN_XFER.with(|x| x.throw.set(0));
        let boxed = unsafe { Box::from_raw(throw_code as *mut (String, String)) };
        (Some(*boxed), false, MbValue::none())
    };

    if let Some((exc_type, exc_msg)) = throw_req {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str(exc_type)),
            MbValue::from_ptr(MbObject::new_str(exc_msg)),
        );
        return MbValue::none();
    }

    if close_req {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("GeneratorExit".to_string())),
            MbValue::from_ptr(MbObject::new_str(String::new())),
        );
        return MbValue::none();
    }

    unsafe {
        super::rc::retain_if_ptr(sent_value);
    }
    sent_value
}

/// Yield from a sub-iterator/generator. Called from compiled code.
pub fn mb_generator_yield_from(sub_iter: MbValue) -> MbValue {
    // If sub_iter is a generator handle, delegate yield
    if sub_iter.is_int() && is_known_generator(sub_iter) {
        return yield_from_generator(sub_iter);
    }

    // Otherwise, iterate and yield each value
    let iter_handle = super::iter::mb_iter(sub_iter);
    if iter_handle.is_none() {
        return MbValue::none();
    }

    loop {
        let has = super::iter::mb_has_next(iter_handle);
        if has.as_bool() == Some(false) {
            break;
        }
        let val = super::iter::mb_next(iter_handle);
        if val.is_none() {
            let exhausted = super::iter::mb_has_next(iter_handle);
            if exhausted.as_bool() == Some(false) {
                break;
            }
        }
        let _sent = mb_generator_yield_value(val);
    }
    super::iter::mb_iter_release(iter_handle);
    MbValue::none()
}

/// Yield from a sub-generator, properly forwarding send/throw/close.
fn yield_from_generator(sub_gen: MbValue) -> MbValue {
    let mut val = mb_generator_next(sub_gen);

    loop {
        let exhausted = if let Some(id) = sub_gen.as_int() {
            GENERATORS.with(|gens| {
                gens.borrow()
                    .get(&(id as u64))
                    .map(|e| matches!(e.state, GenState::Completed))
                    .unwrap_or(true)
            })
        } else {
            true
        };

        if exhausted {
            let ret_val = if let Some(id) = sub_gen.as_int() {
                GENERATORS.with(|gens| {
                    gens.borrow()
                        .get(&(id as u64))
                        .map(|e| e.return_value)
                        .unwrap_or(MbValue::none())
                })
            } else {
                MbValue::none()
            };
            super::iter::check_and_clear_stop();
            super::exception::clear_current_exception();
            return ret_val;
        }

        let sent = mb_generator_yield_value(val);

        if super::exception::mb_has_exception().as_bool() == Some(true) {
            let exc_val = super::exception::mb_catch_exception();
            let exc_type_str = super::exception::get_exception_type_pub(exc_val)
                .unwrap_or_else(|| "Exception".to_string());
            let exc_msg_str =
                super::exception::get_exception_message_pub(exc_val).unwrap_or_default();
            let type_vreg = MbValue::from_ptr(MbObject::new_str(exc_type_str));
            let msg_vreg = MbValue::from_ptr(MbObject::new_str(exc_msg_str));
            val = mb_generator_throw(sub_gen, type_vreg, msg_vreg);
            if super::exception::mb_has_exception().as_bool() == Some(true) {
                return MbValue::none();
            }
            continue;
        }

        if sent.is_none() {
            val = mb_generator_next(sub_gen);
        } else {
            val = mb_generator_send(sub_gen, sent);
        }
    }
}

// ── Body function calling ───────────────────────────────────────────────────

/// Call the compiled body function with the given arguments.
fn call_body_fn(fn_addr: u64, args: &[MbValue]) -> MbValue {
    let raw_addr = fn_addr & 0x0000_FFFF_FFFF_FFFF;
    if raw_addr == 0 {
        return MbValue::none();
    }

    unsafe {
        match args.len() {
            0 => {
                let f: extern "C" fn() -> i64 = std::mem::transmute(raw_addr as usize);
                MbValue::from_bits(f() as u64)
            }
            1 => {
                let f: extern "C" fn(i64) -> i64 = std::mem::transmute(raw_addr as usize);
                MbValue::from_bits(f(args[0].to_bits() as i64) as u64)
            }
            2 => {
                let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(raw_addr as usize);
                MbValue::from_bits(f(args[0].to_bits() as i64, args[1].to_bits() as i64) as u64)
            }
            3 => {
                let f: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(raw_addr as usize);
                MbValue::from_bits(f(
                    args[0].to_bits() as i64,
                    args[1].to_bits() as i64,
                    args[2].to_bits() as i64,
                ) as u64)
            }
            4 => {
                let f: extern "C" fn(i64, i64, i64, i64) -> i64 =
                    std::mem::transmute(raw_addr as usize);
                MbValue::from_bits(f(
                    args[0].to_bits() as i64,
                    args[1].to_bits() as i64,
                    args[2].to_bits() as i64,
                    args[3].to_bits() as i64,
                ) as u64)
            }
            _ => MbValue::none(),
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn raise_stop_iteration(return_value: MbValue) {
    super::iter::signal_stop_iteration();
    LAST_STOP_VALUE.with(|v| v.set(return_value.to_bits()));
    let exc_type = MbValue::from_ptr(MbObject::new_str("StopIteration".to_string()));
    let exc_msg = MbValue::from_ptr(MbObject::new_str(String::new()));
    super::exception::mb_raise(exc_type, exc_msg);
    LAST_STOP_VALUE.with(|v| v.set(return_value.to_bits()));
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::super::rc::MbObject;
    use super::*;

    #[test]
    fn test_generator_create_and_is_exhausted() {
        let name = MbValue::from_ptr(MbObject::new_str("test_gen".to_string()));
        let body_fn = MbValue::none();
        let gen = mb_generator_create(name, body_fn);
        assert_eq!(mb_generator_is_exhausted(gen).as_bool(), Some(false));
        mb_generator_release(gen);
    }

    #[test]
    fn test_generator_release_cleans_up() {
        let name = MbValue::from_ptr(MbObject::new_str("release".into()));
        let body_fn = MbValue::none();
        let gen = mb_generator_create(name, body_fn);
        mb_generator_release(gen);
        assert_eq!(mb_generator_is_exhausted(gen).as_bool(), Some(true));
    }

    #[test]
    fn test_generator_non_int_handle() {
        let bad = MbValue::from_bool(true);
        assert_eq!(mb_generator_is_exhausted(bad).as_bool(), Some(true));
        assert!(mb_generator_next(bad).is_none());
        mb_generator_close(bad);
        mb_generator_release(bad);
    }

    #[test]
    fn test_unique_gen_ids_concurrent() {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(std::thread::spawn(|| {
                let mut ids = Vec::with_capacity(100);
                for _ in 0..100 {
                    ids.push(alloc_gen_id());
                }
                ids
            }));
        }

        let mut all_ids: Vec<u64> = Vec::new();
        for h in handles {
            all_ids.extend(h.join().expect("thread should not panic"));
        }

        let total = all_ids.len();
        assert_eq!(total, 1000, "expected 1000 IDs from 10x100 threads");

        all_ids.sort();
        all_ids.dedup();
        assert_eq!(all_ids.len(), total, "all generator IDs must be unique");
    }

    #[test]
    fn test_cleanup_drains_registry() {
        let mut gen_ids = Vec::new();
        for i in 0..5 {
            let name = MbValue::from_ptr(MbObject::new_str(format!("cleanup_gen_{i}")));
            let body_fn = MbValue::none();
            let gen = mb_generator_create(name, body_fn);
            gen_ids.push(gen);
        }

        for gen in &gen_ids {
            assert!(is_known_generator(*gen), "generator should be registered");
        }

        cleanup_all_generators();

        for gen in &gen_ids {
            assert!(
                !is_known_generator(*gen),
                "generator should be removed after cleanup"
            );
            assert_eq!(
                mb_generator_is_exhausted(*gen).as_bool(),
                Some(true),
                "generator should report exhausted after cleanup"
            );
        }
    }

    #[test]
    fn test_gen_id_monotonically_increasing() {
        let id1 = alloc_gen_id();
        let id2 = alloc_gen_id();
        let id3 = alloc_gen_id();
        assert!(
            id1 < id2,
            "IDs should be strictly increasing: {id1} < {id2}"
        );
        assert!(
            id2 < id3,
            "IDs should be strictly increasing: {id2} < {id3}"
        );
    }

    #[test]
    fn test_coro_stack_allocation() {
        let stack = CoroStack::new(CORO_STACK_SIZE);
        let top = stack.top();
        assert!(!top.is_null());
        assert_eq!(top as usize % 16, 0, "stack top must be 16-byte aligned");
        // Stack should be above base + guard page
        let usable_base = unsafe { stack.base.add(PAGE_SIZE) };
        assert!(top as usize > usable_base as usize);
    }

    /// 8. mb_generator_store_arg appends arguments to the generator's
    ///    args vec, read later by the coroutine trampoline.
    #[test]
    fn test_store_arg_appends_to_generator_args() {
        let name = MbValue::from_ptr(MbObject::new_str("arg_test".into()));
        let gen = mb_generator_create(name, MbValue::none());
        let gen_id = gen.as_int().unwrap() as u64;
        mb_generator_store_arg(gen, MbValue::from_int(42));
        mb_generator_store_arg(gen, MbValue::from_int(99));
        GENERATORS.with(|gens| {
            let gens = gens.borrow();
            let entry = gens.get(&gen_id).unwrap();
            assert_eq!(entry.args.len(), 2);
            assert_eq!(entry.args[0].as_int(), Some(42));
            assert_eq!(entry.args[1].as_int(), Some(99));
        });
        mb_generator_release(gen);
    }
}
