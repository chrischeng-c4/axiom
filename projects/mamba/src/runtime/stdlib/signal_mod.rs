use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
#[cfg(unix)]
use core::ffi::c_void;
use std::cell::RefCell;
/// signal module for Mamba (#1470, #1265 Goal 2 / 3-gate).
///
/// Provides the CPython 3.12 `signal` 61-entry public surface
/// (per `projects/mamba/data/cpython312_surface.json`):
///   - 30 `SIG*` integer constants (one per host platform's signal-number
///     vocabulary; values mirror macOS/Linux POSIX assignments).
///   - 4 `SIG_BLOCK` / `SIG_UNBLOCK` / `SIG_SETMASK` mask-op constants.
///   - 2 disposition sentinels: `SIG_DFL = 0`, `SIG_IGN = 1`.
///   - 3 `ITIMER_*` itimer-mode constants.
///   - `NSIG` (32 on darwin; CPython's reported max signal number + 1).
///   - 3 type / enum class shells: `Signals`, `Handlers`, `Sigmasks`.
///   - 1 exception type shell: `ItimerError`.
///   - 18 callables: `signal`, `getsignal`, `raise_signal`, `set_wakeup_fd`,
///     `siginterrupt`, `strsignal`, `valid_signals`, `default_int_handler`,
///     `pthread_kill`, `pthread_sigmask`, `alarm`, `getitimer`, `setitimer`,
///     `pause`, `sigpending`, `sigwait`.
///
/// Behavior summary (surface, not full semantics):
///   - **`getsignal(signum)`** is the perf-gate hot path (#1470 Gate 2).
///     CPython resolves through a Python-level handler dict, threads through
///     `Signals(signum)` IntEnum construction, and returns either an
///     IntEnum sentinel (SIG_DFL / SIG_IGN) or a Python callable. Mamba
///     returns the integer sentinel `0` (SIG_DFL) directly — no dict lookup,
///     no IntEnum coercion — which beats CPython's hot loop comfortably.
///   - **`signal(signum, handler)`** returns the previous handler;
///     Mamba returns the SIG_DFL sentinel (`0`).
///   - **`raise_signal`** synchronously invokes the installed Python-level
///     handler when one was registered through `signal.signal`.
///   - **`pthread_kill`, `pause`, `sigwait`, `sigpending`, `setitimer`,
///     `getitimer`, `pthread_sigmask`, `siginterrupt`** — return `None` or
///     an empty/zero value as CPython would on a no-op fast path. These are
///     surface-presence stubs; process-wide Unix signal plumbing remains out
///     of scope.
///   - **`set_wakeup_fd`** records the fd and synchronous `raise_signal`
///     writes one signal-number byte to it, matching event-loop wakeup usage.
///   - **`strsignal(signum)`** returns `None` (CPython returns a localized
///     description; surface-presence callers only check callable).
///   - **`valid_signals()`** returns an empty set (CPython returns the set
///     of usable signals).
///   - **`default_int_handler(signum, frame)`** is a no-op returning `None`
///     (CPython raises `KeyboardInterrupt`; emulating that requires real
///     exception propagation which mamba's stdlib surface deliberately
///     avoids — see traceback_mod.rs carve-out).
///   - **`Signals`, `Handlers`, `Sigmasks`** are lightweight IntEnum-style
///     singleton members for the module constants; constructors perform
///     value-to-member lookup so `Signals(2) is SIGINT` holds.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - No Unix `sigaction` plumbing — Mamba only supports synchronous
///     Python-level delivery for handlers recorded in this module.
///     Process-wide signal handling can be wired later behind a separate
///     issue once exception propagation across the JIT boundary lands.
///   - Full EnumMeta behavior is not modeled; only the value/name/identity
///     surface required by the signal module constants is provided.
///   - SIG* integer values reflect the host (macOS/darwin) signal table.
///     Linux-only signal numbers (e.g. `SIGRTMIN`, `SIGPWR`) are not
///     exposed; surface-presence does not test them.
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

#[cfg(unix)]
unsafe extern "C" {
    fn write(fd: i32, buf: *const c_void, count: usize) -> isize;
}

// ── Exception helpers ──
// Raise catchable Python exceptions through the thread-local exception
// machinery (same pattern as bisect_mod / codecs_mod). The returned
// `MbValue::none()` is the dispatcher's return value; the interpreter checks
// the pending-exception flag after the native call returns.

fn raise_exc(exc_type: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}
fn raise_type_error(msg: &str) -> MbValue {
    raise_exc("TypeError", msg)
}
fn raise_value_error(msg: &str) -> MbValue {
    raise_exc("ValueError", msg)
}
fn raise_os_error(msg: &str) -> MbValue {
    raise_exc("OSError", msg)
}

// ── Per-process signal-disposition table ──
// CPython tracks each signal's installed handler so `signal.signal()` can
// return the *previous* disposition. We do not deliver real signals, but we
// still record the last handler installed per signum so the documented
// "returns previous handler" contract holds for pure book-keeping callers.
thread_local! {
    static HANDLERS: RefCell<HashMap<i64, MbValue>> = RefCell::new(HashMap::new());
    static WAKEUP_FD: RefCell<i64> = const { RefCell::new(-1) };
}
static BLOCKED_SIGNALS: LazyLock<Mutex<HashSet<i64>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PENDING_SIGNALS: LazyLock<Mutex<HashSet<i64>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// The distinct, valid signal numbers exposed by this build (mirrors the
/// SIG* table registered below). Used by `valid_signals()` and range checks.
fn valid_signal_numbers() -> Vec<i64> {
    let mut nums: Vec<i64> = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31,
    ];
    nums.dedup();
    nums
}

fn signal_member_defs() -> &'static [(&'static str, i64)] {
    &[
        ("SIGABRT", 6),
        ("SIGALRM", 14),
        ("SIGBUS", 10),
        ("SIGCHLD", 20),
        ("SIGCONT", 19),
        ("SIGEMT", 7),
        ("SIGFPE", 8),
        ("SIGHUP", 1),
        ("SIGILL", 4),
        ("SIGINFO", 29),
        ("SIGINT", 2),
        ("SIGIO", 23),
        ("SIGIOT", 6),
        ("SIGKILL", 9),
        ("SIGPIPE", 13),
        ("SIGPROF", 27),
        ("SIGQUIT", 3),
        ("SIGSEGV", 11),
        ("SIGSTOP", 17),
        ("SIGSYS", 12),
        ("SIGTERM", 15),
        ("SIGTRAP", 5),
        ("SIGTSTP", 18),
        ("SIGTTIN", 21),
        ("SIGTTOU", 22),
        ("SIGURG", 16),
        ("SIGUSR1", 30),
        ("SIGUSR2", 31),
        ("SIGVTALRM", 26),
        ("SIGWINCH", 28),
        ("SIGXCPU", 24),
        ("SIGXFSZ", 25),
    ]
}

fn handler_member_defs() -> &'static [(&'static str, i64)] {
    &[("SIG_DFL", 0), ("SIG_IGN", 1)]
}

fn sigmask_member_defs() -> &'static [(&'static str, i64)] {
    &[("SIG_BLOCK", 1), ("SIG_UNBLOCK", 2), ("SIG_SETMASK", 3)]
}

fn enum_defs(class_name: &str) -> &'static [(&'static str, i64)] {
    match class_name {
        "Signals" => signal_member_defs(),
        "Handlers" => handler_member_defs(),
        "Sigmasks" => sigmask_member_defs(),
        _ => &[],
    }
}

fn insert_int_enum_members(
    attrs: &mut HashMap<String, MbValue>,
    class_name: &str,
    defs: &[(&'static str, i64)],
) {
    let members = super::enum_class::register_int_enum(class_name, defs);
    for ((name, _), member) in defs.iter().zip(members) {
        attrs.insert((*name).to_string(), member);
    }
}

fn lookup_registered_member(class_name: &str, value: i64) -> Option<MbValue> {
    enum_defs(class_name)
        .iter()
        .any(|(_, member_value)| *member_value == value)
        .then_some(())?;
    let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(value)]));
    super::enum_class::enum_class_call(class_name, args).filter(|member| !member.is_none())
}

fn member_value(v: MbValue) -> Option<i64> {
    super::enum_class::int_member_value(v).and_then(|raw| raw.as_int_pyint())
}

fn signal_int_value(v: MbValue) -> Option<i64> {
    v.as_int_pyint().or_else(|| member_value(v))
}

pub(crate) fn signal_enum_int_value(v: MbValue) -> Option<MbValue> {
    member_value(v).map(MbValue::from_int)
}

fn handler_sentinel(value: i64) -> MbValue {
    lookup_registered_member("Handlers", value).unwrap_or_else(|| MbValue::from_int(value))
}

fn signal_member_value(signum: i64) -> MbValue {
    lookup_registered_member("Signals", signum).unwrap_or_else(|| MbValue::from_int(signum))
}

fn signal_set<I>(items: I) -> MbValue
where
    I: IntoIterator<Item = i64>,
{
    MbValue::from_ptr(MbObject::new_set(
        items.into_iter().map(signal_member_value).collect(),
    ))
}

fn mask_numbers(mask: MbValue) -> Option<Vec<i64>> {
    let mut nums = Vec::new();
    for item in seq_items(mask)? {
        nums.push(signal_int_value(item)?);
    }
    Some(nums)
}

fn blocked_snapshot() -> Vec<i64> {
    BLOCKED_SIGNALS
        .lock()
        .unwrap()
        .iter()
        .copied()
        .collect()
}

fn is_signal_blocked(signum: i64) -> bool {
    BLOCKED_SIGNALS.lock().unwrap().contains(&signum)
}

fn queue_pending_signal(signum: i64) {
    PENDING_SIGNALS.lock().unwrap().insert(signum);
}

fn take_unblocked_pending() -> Vec<i64> {
    let blocked = BLOCKED_SIGNALS.lock().unwrap().clone();
    let mut pending = PENDING_SIGNALS.lock().unwrap();
    let ready: Vec<i64> = pending
        .iter()
        .copied()
        .filter(|signum| !blocked.contains(signum))
        .collect();
    for signum in &ready {
        pending.remove(signum);
    }
    ready
}

fn take_pending_from(mask: &[i64]) -> Option<i64> {
    let mut pending = PENDING_SIGNALS.lock().unwrap();
    let signum = mask.iter().copied().find(|signum| pending.contains(signum))?;
    pending.remove(&signum);
    Some(signum)
}

/// Coerce a value to a Python signal number. Accepts plain ints, bools, and
/// the module's IntEnum-style `Signals` members.
fn as_signum(v: MbValue) -> Option<i64> {
    signal_int_value(v)
}

#[cfg(unix)]
fn write_wakeup_byte(signum: i64) {
    let fd = WAKEUP_FD.with(|w| *w.borrow());
    if fd < 0 {
        return;
    }
    let byte = [(signum & 0xff) as u8];
    unsafe {
        let _ = write(fd as i32, byte.as_ptr().cast::<c_void>(), byte.len());
    }
}

#[cfg(not(unix))]
fn write_wakeup_byte(_signum: i64) {}

/// CPython-style short description for a signal number, mirroring the
/// `strsignal(3)` keywords callers assert on. Returns `None` for numbers
/// outside the known table (CPython returns `None` for unknown signals).
fn signal_description(signum: i64) -> Option<&'static str> {
    Some(match signum {
        1 => "Hangup: 1",
        2 => "Interrupt: 2",
        3 => "Quit: 3",
        4 => "Illegal instruction: 4",
        5 => "Trace/BPT trap: 5",
        6 => "Abort trap: 6",
        7 => "EMT trap: 7",
        8 => "Floating point exception: 8",
        9 => "Killed: 9",
        10 => "Bus error: 10",
        11 => "Segmentation fault: 11",
        12 => "Bad system call: 12",
        13 => "Broken pipe: 13",
        14 => "Alarm clock: 14",
        15 => "Terminated: 15",
        16 => "Urgent I/O condition: 16",
        17 => "Suspended (signal): 17",
        18 => "Suspended: 18",
        19 => "Continued: 19",
        20 => "Child exited: 20",
        21 => "Stopped (tty input): 21",
        22 => "Stopped (tty output): 22",
        23 => "I/O possible: 23",
        24 => "Cputime limit exceeded: 24",
        25 => "Filesize limit exceeded: 25",
        26 => "Virtual timer expired: 26",
        27 => "Profiling timer expired: 27",
        28 => "Window size changes: 28",
        29 => "Information request: 29",
        30 => "User defined signal 1: 30",
        31 => "User defined signal 2: 31",
        _ => return None,
    })
}

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

// Callables (18 surface entries)
disp_variadic!(d_signal, mb_signal_signal);
disp_unary!(d_getsignal, mb_signal_getsignal);
disp_unary!(d_raise_signal, mb_signal_raise_signal);
disp_variadic!(d_set_wakeup_fd, mb_signal_set_wakeup_fd);
disp_binary!(d_siginterrupt, mb_signal_siginterrupt);
disp_unary!(d_strsignal, mb_signal_strsignal);
disp_nullary!(d_valid_signals, mb_signal_valid_signals);
disp_binary!(d_default_int_handler, mb_signal_default_int_handler);
disp_binary!(d_pthread_kill, mb_signal_pthread_kill);
disp_variadic!(d_pthread_sigmask, mb_signal_pthread_sigmask);
disp_unary!(d_alarm, mb_signal_alarm);
disp_unary!(d_getitimer, mb_signal_getitimer);
disp_variadic!(d_setitimer, mb_signal_setitimer);
disp_nullary!(d_pause, mb_signal_pause);
disp_nullary!(d_sigpending, mb_signal_sigpending);
disp_unary!(d_sigwait, mb_signal_sigwait);

// Class / type shells (4 surface entries)
disp_variadic!(d_signals, mb_signal_signals_new);
disp_variadic!(d_handlers, mb_signal_handlers_new);
disp_variadic!(d_sigmasks, mb_signal_sigmasks_new);
disp_variadic!(d_itimer_error, mb_signal_itimer_error_new);

/// Register the signal module.
pub fn register() {
    let mut attrs = HashMap::new();

    // ── Callables ──
    let dispatchers: Vec<(&str, usize)> = vec![
        ("signal", d_signal as *const () as usize),
        ("getsignal", d_getsignal as *const () as usize),
        ("raise_signal", d_raise_signal as *const () as usize),
        ("set_wakeup_fd", d_set_wakeup_fd as *const () as usize),
        ("siginterrupt", d_siginterrupt as *const () as usize),
        ("strsignal", d_strsignal as *const () as usize),
        ("valid_signals", d_valid_signals as *const () as usize),
        (
            "default_int_handler",
            d_default_int_handler as *const () as usize,
        ),
        ("pthread_kill", d_pthread_kill as *const () as usize),
        ("pthread_sigmask", d_pthread_sigmask as *const () as usize),
        ("alarm", d_alarm as *const () as usize),
        ("getitimer", d_getitimer as *const () as usize),
        ("setitimer", d_setitimer as *const () as usize),
        ("pause", d_pause as *const () as usize),
        ("sigpending", d_sigpending as *const () as usize),
        ("sigwait", d_sigwait as *const () as usize),
        // Class / type shells
        ("Signals", d_signals as *const () as usize),
        ("Handlers", d_handlers as *const () as usize),
        ("Sigmasks", d_sigmasks as *const () as usize),
        ("ItimerError", d_itimer_error as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        if matches!(name, "Signals" | "Handlers" | "Sigmasks" | "ItimerError") {
            super::super::module::NATIVE_TYPE_NAMES.with(|m| {
                m.borrow_mut().insert(addr as u64, name.to_string());
            });
        }
    }

    // ── IntEnum-style constants. Aliases share the first canonical member for
    // a value (e.g. SIGIOT aliases SIGABRT), which preserves identity lookup.
    insert_int_enum_members(&mut attrs, "Signals", signal_member_defs());
    insert_int_enum_members(&mut attrs, "Handlers", handler_member_defs());
    insert_int_enum_members(&mut attrs, "Sigmasks", sigmask_member_defs());

    // Itimer modes.
    attrs.insert("ITIMER_REAL".to_string(), MbValue::from_int(0));
    attrs.insert("ITIMER_VIRTUAL".to_string(), MbValue::from_int(1));
    attrs.insert("ITIMER_PROF".to_string(), MbValue::from_int(2));

    // NSIG — max signal number + 1 (32 on darwin per CPython).
    attrs.insert("NSIG".to_string(), MbValue::from_int(32));

    super::register_module("signal", attrs);
}

// ── Callables ──

/// signal.signal(signum, handler) -> previous handler.
///
/// Validates the call boundary the way CPython does and records the new
/// disposition so the *previous* one can be returned:
///   - exactly two positional arguments (else `TypeError`);
///   - `signum` must be an integer (else `TypeError`);
///   - `signum` must be a valid signal number (else `ValueError`);
///   - `SIGKILL` / `SIGSTOP` cannot be caught or ignored (`OSError`);
///   - `handler` must be `SIG_DFL` (0), `SIG_IGN` (1), or a callable
///     (else `TypeError`).
pub fn mb_signal_signal(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error(&format!(
            "signal() takes exactly 2 arguments ({} given)",
            args.len()
        ));
    }
    let signum = match as_signum(args[0]) {
        Some(n) => n,
        None => {
            return raise_type_error(
                "signal handler must be signal.SIG_IGN, signal.SIG_DFL, or a callable object",
            );
        }
    };
    if !valid_signal_numbers().contains(&signum) {
        // CPython raises ValueError("signal number out of range") for signums
        // outside 1..NSIG.
        return raise_value_error("signal number out of range");
    }
    // SIGKILL (9) and SIGSTOP (17) cannot be handled.
    if signum == 9 || signum == 17 {
        return raise_os_error("[Errno 22] Invalid argument");
    }
    let handler = args[1];
    // A handler is valid iff it is callable or the SIG_DFL/SIG_IGN sentinel.
    // Callable handles may be int-tagged, so callability must win over raw
    // sentinel normalization.
    let is_callable = super::super::builtins::mb_callable(handler).as_bool() == Some(true);
    let handler_int = if is_callable {
        None
    } else {
        signal_int_value(handler)
    };
    let is_sentinel = matches!(handler_int, Some(0) | Some(1));
    if !is_sentinel && !is_callable {
        return raise_type_error(
            "signal handler must be signal.SIG_IGN, signal.SIG_DFL, or a callable object",
        );
    }
    let stored_handler = match (is_callable, handler_int) {
        (true, _) => handler,
        (false, Some(n @ (0 | 1))) => handler_sentinel(n),
        _ => handler,
    };
    // Record the new disposition; return the previous one (default SIG_DFL).
    let previous = HANDLERS.with(|h| {
        let mut map = h.borrow_mut();
        let prev = map
            .get(&signum)
            .copied()
            .unwrap_or_else(|| handler_sentinel(0));
        map.insert(signum, stored_handler);
        prev
    });
    previous
}

/// signal.getsignal(signum) -> handler.
///
/// Returns the currently installed handler recorded for `signum`, falling
/// back to the SIG_DFL sentinel (`0`) when none was installed. Kept cheap —
/// a single map probe — to preserve the #1470 Gate 2 hot path.
pub fn mb_signal_getsignal(signum: MbValue) -> MbValue {
    match as_signum(signum) {
        Some(n) => HANDLERS.with(|h| {
            h.borrow()
                .get(&n)
                .copied()
                .unwrap_or_else(|| handler_sentinel(0))
        }),
        None => MbValue::from_int(0),
    }
}

/// signal.raise_signal(signum) -> None.
///
/// We do not deliver real OS signals, but a synchronous `raise_signal` can
/// honor the installed Python disposition: a custom handler is invoked as
/// `handler(signum, None)`; SIG_IGN is a no-op; SIG_DFL for SIGINT raises
/// KeyboardInterrupt (CPython's default_int_handler). This makes the common
/// "register a handler, raise_signal, observe the callback" pattern work.
pub fn mb_signal_raise_signal(signum: MbValue) -> MbValue {
    let Some(n) = as_signum(signum) else {
        return raise_value_error("signal number out of range");
    };
    if !valid_signal_numbers().contains(&n) {
        return raise_value_error("signal number out of range");
    }
    if is_signal_blocked(n) {
        queue_pending_signal(n);
        return MbValue::none();
    }
    write_wakeup_byte(n);
    let handler = HANDLERS
        .with(|h| h.borrow().get(&n).copied())
        .unwrap_or_else(|| MbValue::from_int(0));
    // A callable handler and the SIG_DFL/SIG_IGN int sentinels can share raw
    // bits (a closure handle is an int too), so test callability FIRST via the
    // closure registry rather than by the integer value.
    if super::super::builtins::mb_callable(handler).as_bool() == Some(true) {
        // Custom handler: call handler(signum, None) synchronously. Pass the
        // original signum value so `seen == [signal.SIGINT]` holds.
        let args_list = MbValue::from_ptr(MbObject::new_list(vec![signum, MbValue::none()]));
        super::super::builtins::mb_call_spread(handler, args_list);
    } else if signal_int_value(handler) == Some(0) && n == 2 {
        // SIG_DFL for SIGINT raises KeyboardInterrupt (default_int_handler);
        // SIG_IGN (1) and other defaults are a no-op here.
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("KeyboardInterrupt".to_string())),
            MbValue::from_ptr(MbObject::new_str(String::new())),
        );
    }
    MbValue::none()
}

/// Deliver a same-process signal only when Mamba owns the disposition.
///
/// This is used by `os.kill(os.getpid(), sig)`: custom Python handlers and
/// SIG_IGN are pure Mamba state, so they can be honored synchronously.
/// `raise_signal` owns the callable-vs-sentinel distinction; this helper only
/// decides whether the signal has a Mamba-owned disposition at all. Missing
/// handlers and SIG_DFL stay on the OS path so ordinary process-signal behavior
/// is not silently replaced by a no-op.
pub(crate) fn mb_signal_deliver_if_registered(signum: MbValue) -> Option<MbValue> {
    let n = as_signum(signum)?;
    if is_signal_blocked(n) {
        queue_pending_signal(n);
        return Some(MbValue::none());
    }
    let handler = HANDLERS.with(|h| h.borrow().get(&n).copied())?;
    if signal_int_value(handler) == Some(0) {
        None
    } else {
        Some(mb_signal_raise_signal(signum))
    }
}

/// signal.set_wakeup_fd(fd) -> previous fd (-1 sentinel).
///
/// Validates the call boundary:
///   - exactly one positional argument (else `TypeError`);
///   - a wildly out-of-range descriptor (e.g. `2**30`) is rejected with
///     `ValueError`, matching CPython's "invalid fd" rejection without
///     probing host descriptor ownership.
pub fn mb_signal_set_wakeup_fd(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(&format!(
            "set_wakeup_fd() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }
    match signal_int_value(args[0]) {
        Some(fd) => {
            // CPython rejects descriptors that cannot refer to an open file.
            // We do not own a wakeup fd, so treat anything outside a small
            // plausible descriptor window as an invalid descriptor.
            if fd < -1 || fd > 1_000_000 {
                return raise_value_error(&format!("invalid fd: {fd}"));
            }
            let previous = WAKEUP_FD.with(|w| {
                let mut slot = w.borrow_mut();
                let previous = *slot;
                *slot = fd;
                previous
            });
            MbValue::from_int(previous)
        }
        None => raise_type_error("an integer is required (got type)"),
    }
}

/// signal.siginterrupt(signum, flag) -> None.
pub fn mb_signal_siginterrupt(_signum: MbValue, _flag: MbValue) -> MbValue {
    MbValue::none()
}

/// signal.strsignal(signum) -> str | None.
///
/// Returns the host's short description string for the signal (the keyword
/// callers assert on, e.g. "Interrupt" for SIGINT), or `None` when the
/// number is not a known signal.
pub fn mb_signal_strsignal(signum: MbValue) -> MbValue {
    match as_signum(signum).and_then(signal_description) {
        Some(desc) => MbValue::from_ptr(MbObject::new_str(desc.to_string())),
        None => MbValue::none(),
    }
}

/// signal.valid_signals() -> set[int].
///
/// Returns the set of valid signal numbers (1..NSIG), excluding the 0 and
/// NSIG boundary markers, matching CPython's `valid_signals()`.
pub fn mb_signal_valid_signals() -> MbValue {
    let elems: Vec<MbValue> = valid_signal_numbers()
        .into_iter()
        .map(MbValue::from_int)
        .collect();
    MbValue::from_ptr(MbObject::new_set(elems))
}

/// signal.default_int_handler(signum, frame) -> None.
///
/// CPython raises KeyboardInterrupt; mamba returns None. Real exception
/// propagation through native dispatch is out of scope for the surface
/// ticket.
pub fn mb_signal_default_int_handler(_signum: MbValue, _frame: MbValue) -> MbValue {
    // CPython's default SIGINT handler raises KeyboardInterrupt.
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("KeyboardInterrupt".to_string())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );
    MbValue::none()
}

/// signal.pthread_kill(tid, signum) -> None.
pub fn mb_signal_pthread_kill(_tid: MbValue, signum: MbValue) -> MbValue {
    let Some(n) = as_signum(signum) else {
        return raise_value_error("signal number out of range");
    };
    if !valid_signal_numbers().contains(&n) {
        return raise_value_error("signal number out of range");
    }
    mb_signal_raise_signal(MbValue::from_int(n))
}

/// signal.pthread_sigmask(how, mask) -> previous mask (empty set).
///
/// Validates the call boundary the way CPython does:
///   - exactly two positional arguments (else `TypeError`);
///   - `how` must be `SIG_BLOCK` / `SIG_UNBLOCK` / `SIG_SETMASK`
///     (else `OSError`, "Invalid argument");
///   - every entry of `mask` must be a valid signal number `0 < n < NSIG`
///     (else `ValueError`; non-integer / huge BigInt entries also fail).
///
pub fn mb_signal_pthread_sigmask(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error(&format!(
            "pthread_sigmask() takes exactly 2 arguments ({} given)",
            args.len()
        ));
    }
    let how = match signal_int_value(args[0]) {
        Some(n @ (1 | 2 | 3)) => n,
        Some(_) => return raise_os_error("[Errno 22] Invalid argument"),
        None => return raise_os_error("[Errno 22] Invalid argument"),
    };
    // Validate every signal number in the mask iterable.
    let Some(mask) = mask_numbers(args[1]) else {
        return raise_value_error("signal number out of range");
    };
    for n in &mask {
        if !(*n > 0 && *n < 32) {
            return raise_value_error("signal number out of range");
        }
    }
    let previous = blocked_snapshot();
    {
        let mut blocked = BLOCKED_SIGNALS.lock().unwrap();
        match how {
            1 => {
                for signum in &mask {
                    blocked.insert(*signum);
                }
            }
            2 => {
                for signum in &mask {
                    blocked.remove(signum);
                }
            }
            3 => {
                blocked.clear();
                for signum in &mask {
                    blocked.insert(*signum);
                }
            }
            _ => {}
        }
    }
    for signum in take_unblocked_pending() {
        mb_signal_raise_signal(MbValue::from_int(signum));
    }
    signal_set(previous)
}

/// Collect the elements of a List/Tuple/Set `MbValue` into a Vec.
fn seq_items(v: MbValue) -> Option<Vec<MbValue>> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            // MbSet derefs to its ordered MbList for read-only access.
            ObjData::Set(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::Tuple(items) | ObjData::FrozenSet(items) => Some(items.clone()),
            _ => None,
        }
    }
}

/// signal.alarm(seconds) -> remaining alarm (0).
pub fn mb_signal_alarm(_seconds: MbValue) -> MbValue {
    MbValue::from_int(0)
}

/// signal.getitimer(which) -> (value, interval) tuple-shaped list.
pub fn mb_signal_getitimer(_which: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// signal.setitimer(which, seconds, interval=0.0) -> previous (value, interval).
pub fn mb_signal_setitimer(args: &[MbValue]) -> MbValue {
    // CPython: `which` must be an int (an ITIMER_* constant). A non-int is a
    // TypeError raised before the timer is touched.
    let which = args.first().copied().unwrap_or_else(MbValue::none);
    if which.as_int().is_none() {
        return raise_type_error("an integer is required");
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// signal.pause() -> None.
pub fn mb_signal_pause() -> MbValue {
    MbValue::none()
}

/// signal.sigpending() -> set of pending signals.
pub fn mb_signal_sigpending() -> MbValue {
    signal_set(PENDING_SIGNALS.lock().unwrap().iter().copied().collect::<Vec<_>>())
}

/// signal.sigwait(sigset) -> received signum.
pub fn mb_signal_sigwait(sigset: MbValue) -> MbValue {
    let Some(mask) = mask_numbers(sigset) else {
        return raise_value_error("signal number out of range");
    };
    loop {
        if let Some(signum) = take_pending_from(&mask) {
            return signal_member_value(signum);
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

// ── Class / type shells ──

fn signal_enum_new(class_name: &str, args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(&format!(
            "{class_name}() takes exactly 1 argument ({} given)",
            args.len()
        ));
    }
    let raw = match signal_int_value(args[0]) {
        Some(value) => value,
        None => {
            return raise_value_error(&format!("{} is not a valid {class_name}", "None"));
        }
    };
    if let Some(member) = lookup_registered_member(class_name, raw) {
        return member;
    }
    raise_value_error(&format!("{raw} is not a valid {class_name}"))
}

/// signal.Signals(value) -> the singleton Signals member for that value.
pub fn mb_signal_signals_new(args: &[MbValue]) -> MbValue {
    signal_enum_new("Signals", args)
}

/// signal.Handlers(value) -> the singleton Handlers member for that value.
pub fn mb_signal_handlers_new(args: &[MbValue]) -> MbValue {
    signal_enum_new("Handlers", args)
}

/// signal.Sigmasks(value) -> the singleton Sigmasks member for that value.
pub fn mb_signal_sigmasks_new(args: &[MbValue]) -> MbValue {
    signal_enum_new("Sigmasks", args)
}

/// signal.ItimerError -> ItimerError Instance (passive OSError subclass shell).
pub fn mb_signal_itimer_error_new(args: &[MbValue]) -> MbValue {
    let message = args.first().copied().unwrap_or_else(MbValue::none);
    let inst_ptr = MbObject::new_instance("ItimerError".to_string());
    unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert("message".to_string(), message);
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str("ItimerError".to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow()
                .get("signal")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    fn reset_mask_state() {
        BLOCKED_SIGNALS.lock().unwrap().clear();
        PENDING_SIGNALS.lock().unwrap().clear();
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        // Integer constants — 30 SIG* + 2 disposition + 3 mask + 3 itimer + NSIG = 39.
        for name in [
            "SIGABRT",
            "SIGALRM",
            "SIGBUS",
            "SIGCHLD",
            "SIGCONT",
            "SIGEMT",
            "SIGFPE",
            "SIGHUP",
            "SIGILL",
            "SIGINFO",
            "SIGINT",
            "SIGIO",
            "SIGIOT",
            "SIGKILL",
            "SIGPIPE",
            "SIGPROF",
            "SIGQUIT",
            "SIGSEGV",
            "SIGSTOP",
            "SIGSYS",
            "SIGTERM",
            "SIGTRAP",
            "SIGTSTP",
            "SIGTTIN",
            "SIGTTOU",
            "SIGURG",
            "SIGUSR1",
            "SIGUSR2",
            "SIGVTALRM",
            "SIGWINCH",
            "SIGXCPU",
            "SIGXFSZ",
            "SIG_DFL",
            "SIG_IGN",
            "SIG_BLOCK",
            "SIG_UNBLOCK",
            "SIG_SETMASK",
            "ITIMER_REAL",
            "ITIMER_VIRTUAL",
            "ITIMER_PROF",
            "NSIG",
            // Callables (16)
            "signal",
            "getsignal",
            "raise_signal",
            "set_wakeup_fd",
            "siginterrupt",
            "strsignal",
            "valid_signals",
            "default_int_handler",
            "pthread_kill",
            "pthread_sigmask",
            "alarm",
            "getitimer",
            "setitimer",
            "pause",
            "sigpending",
            "sigwait",
            // Class shells (4)
            "Signals",
            "Handlers",
            "Sigmasks",
            "ItimerError",
        ] {
            assert!(
                signal_attr(name).is_some(),
                "signal module missing entry: {name}"
            );
        }
    }

    #[test]
    fn test_sig_constants_values() {
        register();
        assert_eq!(signal_attr("SIGINT").and_then(signal_int_value), Some(2));
        assert_eq!(signal_attr("SIGTERM").and_then(signal_int_value), Some(15));
        assert_eq!(signal_attr("SIGKILL").and_then(signal_int_value), Some(9));
        assert_eq!(signal_attr("SIGHUP").and_then(signal_int_value), Some(1));
        assert_eq!(signal_attr("SIG_DFL").and_then(signal_int_value), Some(0));
        assert_eq!(signal_attr("SIG_IGN").and_then(signal_int_value), Some(1));
        assert_eq!(signal_attr("NSIG").and_then(|v| v.as_int()), Some(32));
    }

    #[test]
    fn test_getsignal_no_handler_returns_sig_dfl() {
        // With no handler installed for this signum, getsignal returns the
        // SIG_DFL sentinel (0).
        let r = mb_signal_getsignal(MbValue::from_int(13));
        assert_eq!(signal_int_value(r), Some(0));
    }

    #[test]
    fn test_signal_records_and_returns_previous() {
        // First install over a fresh signum returns SIG_DFL (0); the second
        // install returns the SIG_IGN sentinel we just put in place.
        let prev1 = mb_signal_signal(&[MbValue::from_int(30), MbValue::from_int(1)]);
        assert_eq!(signal_int_value(prev1), Some(0));
        let prev2 = mb_signal_signal(&[MbValue::from_int(30), MbValue::from_int(0)]);
        assert_eq!(signal_int_value(prev2), Some(1));
    }

    #[test]
    fn test_signal_rejects_bad_handler() {
        // Non-sentinel int handler is not callable -> TypeError pending.
        let r = mb_signal_signal(&[MbValue::from_int(30), MbValue::from_int(42)]);
        assert!(r.is_none());
    }

    #[test]
    fn test_noop_callables_return_none() {
        assert!(mb_signal_raise_signal(MbValue::from_int(2)).is_none());
        assert!(mb_signal_siginterrupt(MbValue::from_int(2), MbValue::none()).is_none());
        assert!(mb_signal_default_int_handler(MbValue::from_int(2), MbValue::none()).is_none());
        assert!(mb_signal_pthread_kill(MbValue::from_int(0), MbValue::from_int(2)).is_none());
        assert!(mb_signal_pause().is_none());
    }

    #[test]
    fn test_masked_signal_becomes_pending_and_sigwait_consumes() {
        register();
        reset_mask_state();
        let sig = MbValue::from_int(30);
        let mask = MbValue::from_ptr(MbObject::new_list(vec![sig]));
        let previous = mb_signal_pthread_sigmask(&[MbValue::from_int(1), mask]);
        assert!(seq_items(previous).unwrap().is_empty());

        assert!(mb_signal_deliver_if_registered(sig).unwrap().is_none());
        let pending = seq_items(mb_signal_sigpending()).unwrap();
        assert_eq!(pending.iter().copied().find_map(signal_int_value), Some(30));

        let waited = mb_signal_sigwait(mask);
        assert_eq!(signal_int_value(waited), Some(30));
        assert!(seq_items(mb_signal_sigpending()).unwrap().is_empty());

        let _ = mb_signal_pthread_sigmask(&[MbValue::from_int(2), mask]);
        reset_mask_state();
    }

    #[test]
    fn test_strsignal_returns_description() {
        // SIGINT (2) carries the "Interrupt" keyword; unknown numbers -> None.
        let r = mb_signal_strsignal(MbValue::from_int(2));
        let ptr = r.as_ptr().expect("expected str");
        unsafe {
            if let super::super::super::rc::ObjData::Str(ref s) = (*ptr).data {
                assert!(s.contains("Interrupt"), "got {s:?}");
            } else {
                panic!("expected Str");
            }
        }
        assert!(mb_signal_strsignal(MbValue::from_int(9999)).is_none());
    }

    #[test]
    fn test_set_wakeup_fd_returns_minus_one() {
        let _ = mb_signal_set_wakeup_fd(&[MbValue::from_int(-1)]);
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(3)]).as_int(),
            Some(-1)
        );
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(-1)]).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_set_wakeup_fd_returns_previous() {
        let _ = mb_signal_set_wakeup_fd(&[MbValue::from_int(-1)]);
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(3)]).as_int(),
            Some(-1)
        );
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(4)]).as_int(),
            Some(3)
        );
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(-1)]).as_int(),
            Some(4)
        );
    }

    #[test]
    fn test_signals_class_shell_carries_value() {
        register();
        let sigint = signal_attr("SIGINT").expect("SIGINT");
        let inst = mb_signal_signals_new(&[MbValue::from_int(2)]);
        assert_eq!(inst.to_bits(), sigint.to_bits());
        unsafe {
            if let super::super::super::rc::ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*inst.as_ptr().unwrap()).data
            {
                assert_eq!(class_name, "Signals");
                let f = fields.read().unwrap();
                assert_eq!(f.get("value").and_then(|v| v.as_int()), Some(2));
                let name = f.get("name").and_then(|v| {
                    v.as_ptr().and_then(|p| match &(*p).data {
                        super::super::super::rc::ObjData::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                });
                assert_eq!(name.as_deref(), Some("SIGINT"));
            } else {
                panic!("expected Instance");
            }
        }
    }
}
