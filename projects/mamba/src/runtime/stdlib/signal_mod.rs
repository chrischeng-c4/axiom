use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
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
///   - **`raise_signal`, `pthread_kill`, `pause`, `sigwait`, `sigpending`,
///     `setitimer`, `getitimer`, `pthread_sigmask`, `set_wakeup_fd`,
///     `siginterrupt`** — return `None` or an empty/zero value as
///     CPython would on a no-op fast path. These are surface-presence
///     stubs; sending real signals from a mamba process is out of scope.
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
///   - No actual signal delivery / Unix `sigaction` plumbing — every
///     callable that would mutate process state in CPython is a no-op
///     here. Real signal handling can be wired later behind a separate
///     issue once exception propagation across the JIT boundary lands.
///   - Full EnumMeta behavior is not modeled; only the value/name/identity
///     surface required by the signal module constants is provided.
///   - SIG* integer values reflect the host (macOS/darwin) signal table.
///     Linux-only signal numbers (e.g. `SIGRTMIN`, `SIGPWR`) are not
///     exposed; surface-presence does not test them.
use std::collections::HashMap;

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
}

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

fn make_int_enum_member(class_name: &str, member_name: &str, value: i64) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    let inst = MbValue::from_ptr(inst_ptr);
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "name".to_string(),
                MbValue::from_ptr(MbObject::new_str(member_name.to_string())),
            );
            map.insert(
                "_name_".to_string(),
                MbValue::from_ptr(MbObject::new_str(member_name.to_string())),
            );
            map.insert("value".to_string(), MbValue::from_int(value));
            map.insert("_value_".to_string(), MbValue::from_int(value));
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
        }
    }
    inst
}

fn insert_int_enum_members(
    attrs: &mut HashMap<String, MbValue>,
    class_name: &str,
    defs: &[(&'static str, i64)],
) {
    let mut by_value: HashMap<i64, MbValue> = HashMap::new();
    for (name, value) in defs {
        let member = *by_value
            .entry(*value)
            .or_insert_with(|| make_int_enum_member(class_name, name, *value));
        attrs.insert((*name).to_string(), member);
    }
}

fn lookup_registered_member(class_name: &str, value: i64) -> Option<MbValue> {
    let member_name = enum_defs(class_name)
        .iter()
        .find_map(|(name, v)| (*v == value).then_some(*name))?;
    super::super::module::MODULES.with(|mods| {
        mods.borrow()
            .get("signal")
            .and_then(|m| m.attrs.get(member_name).copied())
    })
}

fn member_value(v: MbValue) -> Option<i64> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if matches!(class_name.as_str(), "Signals" | "Handlers" | "Sigmasks") {
                return fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("value").and_then(|value| value.as_int_pyint()));
            }
        }
        None
    })
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

/// Coerce a value to a Python signal number. Accepts plain ints, bools, and
/// the module's IntEnum-style `Signals` members.
fn as_signum(v: MbValue) -> Option<i64> {
    signal_int_value(v)
}

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

/// signal.set_wakeup_fd(fd) -> previous fd (-1 sentinel).
///
/// Validates the call boundary:
///   - exactly one positional argument (else `TypeError`);
///   - a wildly out-of-range descriptor (e.g. `2**30`) is rejected with
///     `ValueError`, matching CPython's "invalid fd" rejection without
///     actually installing a wakeup fd.
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
            MbValue::from_int(-1)
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
pub fn mb_signal_pthread_kill(_tid: MbValue, _signum: MbValue) -> MbValue {
    MbValue::none()
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
/// Process signal masks are not actually mutated; the returned previous
/// mask is an empty set.
pub fn mb_signal_pthread_sigmask(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error(&format!(
            "pthread_sigmask() takes exactly 2 arguments ({} given)",
            args.len()
        ));
    }
    // `how` must be one of the three mask-op constants.
    match signal_int_value(args[0]) {
        Some(1) | Some(2) | Some(3) => {}
        Some(_) => return raise_os_error("[Errno 22] Invalid argument"),
        None => return raise_os_error("[Errno 22] Invalid argument"),
    }
    // Validate every signal number in the mask iterable.
    if let Some(items) = seq_items(args[1]) {
        for item in items {
            match signal_int_value(item) {
                // Valid signal numbers are strictly between 0 and NSIG.
                Some(n) if n > 0 && n < 32 => {}
                // 0, NSIG, negatives, and out-of-range numbers are invalid.
                Some(_) => return raise_value_error("signal number out of range"),
                // Non-int entries (including huge BigInts that don't fit
                // i64) cannot be valid signal numbers.
                None => return raise_value_error("signal number out of range"),
            }
        }
    }
    MbValue::from_ptr(MbObject::new_set(Vec::new()))
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

/// signal.sigpending() -> set of pending signals (empty).
pub fn mb_signal_sigpending() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// signal.sigwait(sigset) -> received signum (0 sentinel).
pub fn mb_signal_sigwait(_sigset: MbValue) -> MbValue {
    MbValue::from_int(0)
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
        assert_eq!(
            mb_signal_set_wakeup_fd(&[MbValue::from_int(3)]).as_int(),
            Some(-1)
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
