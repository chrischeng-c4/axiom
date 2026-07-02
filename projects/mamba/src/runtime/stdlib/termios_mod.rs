//! `termios` module for Mamba — POSIX terminal control (issue #872).
///
/// Wraps the libc `termios(3)`/`tcgetattr(3)`/`tcsetattr(3)` family plus the
/// `TIOCGWINSZ`/`TIOCSWINSZ` ioctls (`tcgetwinsize`/`tcsetwinsize`, CPython
/// 3.11+). This is the piece that unblocks the pure-Python `tty`/`pty`
/// modules (not vendored by this change — see issue scope).
///
/// `termios.error` is a plain `Exception` subclass, NOT an `OSError`
/// subclass — verified against the CPython 3.12 oracle
/// (`issubclass(termios.error, OSError) is False`) and asserted by
/// `test_termios.TestModule.test_exception`. Each raise carries a 2-tuple
/// `(errno, strerror)` as `.args`, with no `.errno`/`.strerror` attributes
/// (unlike `OSError`).
///
/// Dispatch wrappers use the native-ABI calling convention
/// (`extern "C" fn(*const MbValue, usize) -> MbValue`, addresses registered
/// in `NATIVE_FUNC_ADDRS`) — see `pwd_mod.rs`/`select_mod.rs`. The
/// `fn(MbValue) -> MbValue` packed-list convention in `posix_mod.rs` is
/// broken for `mod.func(arg)`-style calls (issue #874) and must not be used.
use super::super::rc::{InstanceFields, MbObject, MbObjectHeader, MbRwLock, ObjData, ObjKind};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::os::raw::{c_int, c_ulong};
use std::sync::atomic::AtomicU32;

macro_rules! disp_variadic {
    ($name:ident, $func:path) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let args = if nargs == 0 || args_ptr.is_null() {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
            };
            $func(args)
        }
    };
}

disp_variadic!(dispatch_tcgetattr, mb_tcgetattr);
disp_variadic!(dispatch_tcsetattr, mb_tcsetattr);
disp_variadic!(dispatch_tcsendbreak, mb_tcsendbreak);
disp_variadic!(dispatch_tcdrain, mb_tcdrain);
disp_variadic!(dispatch_tcflush, mb_tcflush);
disp_variadic!(dispatch_tcflow, mb_tcflow);
disp_variadic!(dispatch_tcgetwinsize, mb_tcgetwinsize);
disp_variadic!(dispatch_tcsetwinsize, mb_tcsetwinsize);

// ── Small local helpers (mirrors the convention duplicated across stdlib/*) ──

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn raise(exc_type: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(exc_type), new_str(msg.into()));
    MbValue::none()
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    raise("TypeError", msg)
}

fn raise_value_error(msg: impl Into<String>) -> MbValue {
    raise("ValueError", msg)
}

fn raise_overflow_error(msg: impl Into<String>) -> MbValue {
    raise("OverflowError", msg)
}

fn type_name_of(val: MbValue) -> &'static str {
    if val.is_none() {
        "NoneType"
    } else if val.is_bool() {
        "bool"
    } else if val.is_int() {
        "int"
    } else if val.as_float().is_some() {
        "float"
    } else {
        match val.as_ptr() {
            Some(ptr) => unsafe {
                match &(*ptr).data {
                    ObjData::Str(_) => "str",
                    ObjData::List(_) => "list",
                    ObjData::Tuple(_) => "tuple",
                    ObjData::Dict(_) => "dict",
                    ObjData::Bytes(_) => "bytes",
                    ObjData::ByteArray(_) => "bytearray",
                    ObjData::BigInt(_) => "int",
                    ObjData::Instance { class_name, .. } => class_name.as_str(),
                    _ => "object",
                }
            },
            None => "object",
        }
    }
}

fn is_bigint(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::BigInt(_)) })
        .unwrap_or(false)
}

/// Raises `termios.error((errno, strerror))` — a plain `Exception`
/// subclass, not `OSError` (confirmed against the CPython 3.12 oracle).
fn raise_termios_error(errno: i32) -> MbValue {
    let strerror = unsafe {
        let p = libc::strerror(errno);
        if p.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    };
    let args_tuple = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(errno as i64),
        new_str(strerror),
    ]));
    let message = super::super::builtins::mb_repr(args_tuple);
    let mut fields = InstanceFields::default();
    fields.insert("message".to_string(), message);
    fields.insert(
        "__type__".to_string(),
        new_str("termios.error".to_string()),
    );
    fields.insert("__cause__".to_string(), MbValue::none());
    fields.insert("__context__".to_string(), MbValue::none());
    fields.insert(
        "__suppress_context__".to_string(),
        MbValue::from_bool(false),
    );
    fields.insert("args".to_string(), args_tuple);
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "termios.error".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    super::super::class::mb_raise_instance(MbValue::from_ptr(Box::into_raw(obj)));
    MbValue::none()
}

fn raise_if_errno(rv: c_int) -> Option<MbValue> {
    if rv < 0 {
        Some(raise_termios_error(std::io::Error::last_os_error().raw_os_error().unwrap_or(0)))
    } else {
        None
    }
}

/// Convert a Python-int argument to a C `int`, matching CPython's
/// `PyArg_ParseTuple(..., "i", ...)` contract: fixnums out of `c_int` range
/// and heap `BigInt`s raise `OverflowError`; non-ints raise `TypeError`.
fn extract_c_int_arg(val: MbValue, what: &str) -> Result<c_int, MbValue> {
    if let Some(i) = val.as_int_pyint() {
        if i < c_int::MIN as i64 || i > c_int::MAX as i64 {
            return Err(raise_overflow_error(format!(
                "Python int too large to convert to C int"
            )));
        }
        return Ok(i as c_int);
    }
    if is_bigint(val) {
        return Err(raise_overflow_error("Python int too large to convert to C int"));
    }
    Err(raise_type_error(format!(
        "an integer is required (got type {})",
        type_name_of(val)
    )))
    .map_err(|e| {
        let _ = what;
        e
    })
}

/// mamba's `open()` / `os.open()` hand back TABLE-SURROGATE ids, not real
/// OS-level fds; real `libc::tcgetattr`/`ioctl` need a genuine fd, so any
/// plain int is resolved through `file_io::mb_file_raw_fd` /
/// `os_mod::mb_os_fd_raw_fd` first (same bridge `mmap_mod.rs` uses for
/// #871), falling back to treating the value as an already-real fd (0/1/2,
/// or one obtained via a genuine syscall wrapper).
fn resolve_fd(raw: i64) -> i32 {
    if let Some(fd) = super::super::file_io::mb_file_raw_fd(MbValue::from_int(raw)) {
        return fd;
    }
    if let Some(fd) = super::os_mod::mb_os_fd_raw_fd(raw) {
        return fd;
    }
    raw as i32
}

/// `fd` argument: accepts an int fd or an object with a `.fileno()` method
/// (matching CPython's `PyArg_ParseTuple(..., "O&", ...)` fd-conversion
/// contract used across `os`/`fcntl`/`termios`). BigInt/negative/wrong-type
/// dispatch to OverflowError/ValueError/TypeError respectively.
fn extract_fd(value: MbValue) -> Result<c_int, MbValue> {
    let raw = if value.as_int_pyint().is_some() || is_bigint(value) {
        extract_c_int_arg(value, "file descriptor")?
    } else {
        let method = new_str("fileno");
        let args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let result = super::super::class::mb_call_method(value, method, args);
        if result.as_int_pyint().is_none() && !is_bigint(result) {
            return Err(raise_type_error(
                "argument must be an int, or have a fileno() method.",
            ));
        }
        extract_c_int_arg(result, "file descriptor")?
    };
    if raw < 0 {
        return Err(raise_value_error(format!(
            "file descriptor cannot be a negative integer ({raw})"
        )));
    }
    Ok(resolve_fd(raw as i64))
}

/// Strict `list`-only extraction for `tcsetattr`'s `attrs` argument — a
/// bare tuple of the right shape is explicitly rejected by CPython
/// (`test_tcsetattr_errors`: `tuple(attrs)` -> `TypeError`).
fn extract_strict_list(value: MbValue) -> Option<Vec<MbValue>> {
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            _ => None,
        }
    }
}

/// `tcsetwinsize`'s `winsz` argument accepts any 2-item list OR tuple.
fn extract_seq_items(value: MbValue) -> Option<Vec<MbValue>> {
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    }
}

/// One of the six top-level flag/speed fields: any Python int converts via
/// a C `long`-style truncating reinterpret; only `BigInt`s that don't fit
/// in a C `long` raise `OverflowError` (matches the oracle: `2**1000`
/// overflows, ordinary out-of-range fixnums do not get special-cased here
/// since real flag words never approach `i64` bounds).
fn extract_flag_field(value: MbValue) -> Result<i64, MbValue> {
    if let Some(i) = value.as_int_pyint() {
        return Ok(i);
    }
    if is_bigint(value) {
        return Err(raise_overflow_error("Python int too large to convert to C long"));
    }
    Err(raise_type_error(format!(
        "an integer is required (got type {})",
        type_name_of(value)
    )))
}

/// One `cc[]` element: a length-1 `bytes`/`bytearray`, or any int
/// (truncated to a byte via two's-complement, matching the observed
/// oracle behavior where `256 -> 0` and `-1 -> 0xff`).
fn extract_cc_byte(value: MbValue) -> Result<u8, MbValue> {
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) if b.len() == 1 => return Ok(b[0]),
                ObjData::ByteArray(lock) => {
                    let data = lock.read().unwrap();
                    if data.len() == 1 {
                        return Ok(data[0]);
                    }
                    return Err(raise_type_error("cc must be a list of 1-character bytes"));
                }
                ObjData::Bytes(_) => {
                    return Err(raise_type_error("cc must be a list of 1-character bytes"));
                }
                ObjData::BigInt(_) => {
                    return Err(raise_overflow_error(
                        "Python int too large to convert to C long",
                    ));
                }
                _ => {}
            }
        }
    }
    if let Some(i) = value.as_int_pyint() {
        return Ok((i as i64) as u8);
    }
    Err(raise_type_error("cc must be a list of 1-character bytes"))
}

fn register_func(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
    attrs.insert(name.to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

pub fn register() {
    let mut attrs = HashMap::new();

    register_func(
        &mut attrs,
        "tcgetattr",
        dispatch_tcgetattr as *const () as usize,
    );
    register_func(
        &mut attrs,
        "tcsetattr",
        dispatch_tcsetattr as *const () as usize,
    );
    register_func(
        &mut attrs,
        "tcsendbreak",
        dispatch_tcsendbreak as *const () as usize,
    );
    register_func(
        &mut attrs,
        "tcdrain",
        dispatch_tcdrain as *const () as usize,
    );
    register_func(
        &mut attrs,
        "tcflush",
        dispatch_tcflush as *const () as usize,
    );
    register_func(&mut attrs, "tcflow", dispatch_tcflow as *const () as usize);
    register_func(
        &mut attrs,
        "tcgetwinsize",
        dispatch_tcgetwinsize as *const () as usize,
    );
    register_func(
        &mut attrs,
        "tcsetwinsize",
        dispatch_tcsetwinsize as *const () as usize,
    );

    attrs.insert("error".to_string(), new_str("termios.error"));

    // tcsetattr() `when` actions.
    attrs.insert("TCSANOW".into(), MbValue::from_int(libc::TCSANOW as i64));
    attrs.insert("TCSADRAIN".into(), MbValue::from_int(libc::TCSADRAIN as i64));
    attrs.insert("TCSAFLUSH".into(), MbValue::from_int(libc::TCSAFLUSH as i64));

    // tcflush() queue selectors.
    attrs.insert("TCIFLUSH".into(), MbValue::from_int(libc::TCIFLUSH as i64));
    attrs.insert("TCOFLUSH".into(), MbValue::from_int(libc::TCOFLUSH as i64));
    attrs.insert("TCIOFLUSH".into(), MbValue::from_int(libc::TCIOFLUSH as i64));

    // tcflow() actions.
    attrs.insert("TCOOFF".into(), MbValue::from_int(libc::TCOOFF as i64));
    attrs.insert("TCOON".into(), MbValue::from_int(libc::TCOON as i64));
    attrs.insert("TCIOFF".into(), MbValue::from_int(libc::TCIOFF as i64));
    attrs.insert("TCION".into(), MbValue::from_int(libc::TCION as i64));

    // cc[] indices.
    attrs.insert("NCCS".into(), MbValue::from_int(libc::NCCS as i64));
    attrs.insert("VEOF".into(), MbValue::from_int(libc::VEOF as i64));
    attrs.insert("VEOL".into(), MbValue::from_int(libc::VEOL as i64));
    attrs.insert("VEOL2".into(), MbValue::from_int(libc::VEOL2 as i64));
    attrs.insert("VERASE".into(), MbValue::from_int(libc::VERASE as i64));
    attrs.insert("VWERASE".into(), MbValue::from_int(libc::VWERASE as i64));
    attrs.insert("VKILL".into(), MbValue::from_int(libc::VKILL as i64));
    attrs.insert("VREPRINT".into(), MbValue::from_int(libc::VREPRINT as i64));
    attrs.insert("VINTR".into(), MbValue::from_int(libc::VINTR as i64));
    attrs.insert("VQUIT".into(), MbValue::from_int(libc::VQUIT as i64));
    attrs.insert("VSUSP".into(), MbValue::from_int(libc::VSUSP as i64));
    attrs.insert("VSTART".into(), MbValue::from_int(libc::VSTART as i64));
    attrs.insert("VSTOP".into(), MbValue::from_int(libc::VSTOP as i64));
    attrs.insert("VLNEXT".into(), MbValue::from_int(libc::VLNEXT as i64));
    attrs.insert("VDISCARD".into(), MbValue::from_int(libc::VDISCARD as i64));
    attrs.insert("VMIN".into(), MbValue::from_int(libc::VMIN as i64));
    attrs.insert("VTIME".into(), MbValue::from_int(libc::VTIME as i64));

    // c_iflag bits.
    attrs.insert("IGNBRK".into(), MbValue::from_int(libc::IGNBRK as i64));
    attrs.insert("BRKINT".into(), MbValue::from_int(libc::BRKINT as i64));
    attrs.insert("IGNPAR".into(), MbValue::from_int(libc::IGNPAR as i64));
    attrs.insert("PARMRK".into(), MbValue::from_int(libc::PARMRK as i64));
    attrs.insert("INPCK".into(), MbValue::from_int(libc::INPCK as i64));
    attrs.insert("ISTRIP".into(), MbValue::from_int(libc::ISTRIP as i64));
    attrs.insert("INLCR".into(), MbValue::from_int(libc::INLCR as i64));
    attrs.insert("IGNCR".into(), MbValue::from_int(libc::IGNCR as i64));
    attrs.insert("ICRNL".into(), MbValue::from_int(libc::ICRNL as i64));
    attrs.insert("IXON".into(), MbValue::from_int(libc::IXON as i64));
    attrs.insert("IXANY".into(), MbValue::from_int(libc::IXANY as i64));
    attrs.insert("IXOFF".into(), MbValue::from_int(libc::IXOFF as i64));
    attrs.insert("IMAXBEL".into(), MbValue::from_int(libc::IMAXBEL as i64));

    // c_oflag bits.
    attrs.insert("OPOST".into(), MbValue::from_int(libc::OPOST as i64));
    attrs.insert("ONLCR".into(), MbValue::from_int(libc::ONLCR as i64));

    // c_cflag bits.
    attrs.insert("CSIZE".into(), MbValue::from_int(libc::CSIZE as i64));
    attrs.insert("CS5".into(), MbValue::from_int(libc::CS5 as i64));
    attrs.insert("CS6".into(), MbValue::from_int(libc::CS6 as i64));
    attrs.insert("CS7".into(), MbValue::from_int(libc::CS7 as i64));
    attrs.insert("CS8".into(), MbValue::from_int(libc::CS8 as i64));
    attrs.insert("CSTOPB".into(), MbValue::from_int(libc::CSTOPB as i64));
    attrs.insert("CREAD".into(), MbValue::from_int(libc::CREAD as i64));
    attrs.insert("PARENB".into(), MbValue::from_int(libc::PARENB as i64));
    attrs.insert("PARODD".into(), MbValue::from_int(libc::PARODD as i64));
    attrs.insert("HUPCL".into(), MbValue::from_int(libc::HUPCL as i64));
    attrs.insert("CLOCAL".into(), MbValue::from_int(libc::CLOCAL as i64));

    // c_lflag bits.
    attrs.insert("ECHO".into(), MbValue::from_int(libc::ECHO as i64));
    attrs.insert("ECHOE".into(), MbValue::from_int(libc::ECHOE as i64));
    attrs.insert("ECHOK".into(), MbValue::from_int(libc::ECHOK as i64));
    attrs.insert("ECHONL".into(), MbValue::from_int(libc::ECHONL as i64));
    attrs.insert("ISIG".into(), MbValue::from_int(libc::ISIG as i64));
    attrs.insert("ICANON".into(), MbValue::from_int(libc::ICANON as i64));
    attrs.insert("IEXTEN".into(), MbValue::from_int(libc::IEXTEN as i64));
    attrs.insert("TOSTOP".into(), MbValue::from_int(libc::TOSTOP as i64));
    attrs.insert("NOFLSH".into(), MbValue::from_int(libc::NOFLSH as i64));
    attrs.insert("ECHOCTL".into(), MbValue::from_int(libc::ECHOCTL as i64));
    attrs.insert("ECHOPRT".into(), MbValue::from_int(libc::ECHOPRT as i64));
    attrs.insert("ECHOKE".into(), MbValue::from_int(libc::ECHOKE as i64));
    attrs.insert("FLUSHO".into(), MbValue::from_int(libc::FLUSHO as i64));
    attrs.insert("PENDIN".into(), MbValue::from_int(libc::PENDIN as i64));

    // Baud rate constants.
    attrs.insert("B0".into(), MbValue::from_int(libc::B0 as i64));
    attrs.insert("B50".into(), MbValue::from_int(libc::B50 as i64));
    attrs.insert("B75".into(), MbValue::from_int(libc::B75 as i64));
    attrs.insert("B110".into(), MbValue::from_int(libc::B110 as i64));
    attrs.insert("B134".into(), MbValue::from_int(libc::B134 as i64));
    attrs.insert("B150".into(), MbValue::from_int(libc::B150 as i64));
    attrs.insert("B200".into(), MbValue::from_int(libc::B200 as i64));
    attrs.insert("B300".into(), MbValue::from_int(libc::B300 as i64));
    attrs.insert("B600".into(), MbValue::from_int(libc::B600 as i64));
    attrs.insert("B1200".into(), MbValue::from_int(libc::B1200 as i64));
    attrs.insert("B1800".into(), MbValue::from_int(libc::B1800 as i64));
    attrs.insert("B2400".into(), MbValue::from_int(libc::B2400 as i64));
    attrs.insert("B4800".into(), MbValue::from_int(libc::B4800 as i64));
    attrs.insert("B9600".into(), MbValue::from_int(libc::B9600 as i64));
    attrs.insert("B19200".into(), MbValue::from_int(libc::B19200 as i64));
    attrs.insert("B38400".into(), MbValue::from_int(libc::B38400 as i64));
    attrs.insert("B57600".into(), MbValue::from_int(libc::B57600 as i64));
    attrs.insert("B115200".into(), MbValue::from_int(libc::B115200 as i64));
    attrs.insert("B230400".into(), MbValue::from_int(libc::B230400 as i64));

    super::register_module("termios", attrs);
}

fn mb_tcgetattr(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error("tcgetattr() takes exactly 1 argument");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };

    let mut t: libc::termios = unsafe { std::mem::zeroed() };
    if let Some(err) = raise_if_errno(unsafe { libc::tcgetattr(fd, &mut t) }) {
        return err;
    }

    let ispeed = unsafe { libc::cfgetispeed(&t) } as i64;
    let ospeed = unsafe { libc::cfgetospeed(&t) } as i64;
    let lflag = t.c_lflag as i64;

    let mut cc_items: Vec<MbValue> = Vec::with_capacity(libc::NCCS);
    for i in 0..libc::NCCS {
        let byte = t.c_cc[i];
        if (lflag as libc::tcflag_t & libc::ICANON) == 0 && (i == libc::VMIN || i == libc::VTIME) {
            cc_items.push(MbValue::from_int(byte as i64));
        } else {
            cc_items.push(MbValue::from_ptr(MbObject::new_bytes(vec![byte])));
        }
    }

    let result = vec![
        MbValue::from_int(t.c_iflag as i64),
        MbValue::from_int(t.c_oflag as i64),
        MbValue::from_int(t.c_cflag as i64),
        MbValue::from_int(lflag),
        MbValue::from_int(ispeed),
        MbValue::from_int(ospeed),
        MbValue::from_ptr(MbObject::new_list(cc_items)),
    ];
    MbValue::from_ptr(MbObject::new_list(result))
}

fn mb_tcsetattr(args: &[MbValue]) -> MbValue {
    if args.len() != 3 {
        return raise_type_error("tcsetattr() takes exactly 3 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let when = match extract_c_int_arg(args[1], "when") {
        Ok(w) => w,
        Err(err) => return err,
    };
    let Some(items) = extract_strict_list(args[2]) else {
        return raise_type_error("tcsetattr, arg 3: must be 7 element list");
    };
    if items.len() != 7 {
        return raise_type_error("tcsetattr, arg 3: must be 7 element list");
    }

    let mut flags: [i64; 6] = [0; 6];
    for i in 0..6 {
        match extract_flag_field(items[i]) {
            Ok(v) => flags[i] = v,
            Err(err) => return err,
        }
    }

    let Some(cc_seq) = extract_seq_items(items[6]) else {
        return raise_type_error("tcsetattr, arg 3: must be 7 element list");
    };
    if cc_seq.len() != libc::NCCS {
        return raise_type_error("tcsetattr, arg 3: must be 7 element list");
    }
    let mut cc: [u8; 20] = [0; 20];
    if libc::NCCS > cc.len() {
        return raise_value_error("unsupported NCCS size");
    }
    for i in 0..libc::NCCS {
        match extract_cc_byte(cc_seq[i]) {
            Ok(b) => cc[i] = b,
            Err(err) => return err,
        }
    }

    let mut t: libc::termios = unsafe { std::mem::zeroed() };
    t.c_iflag = flags[0] as libc::tcflag_t;
    t.c_oflag = flags[1] as libc::tcflag_t;
    t.c_cflag = flags[2] as libc::tcflag_t;
    t.c_lflag = flags[3] as libc::tcflag_t;
    for i in 0..libc::NCCS {
        t.c_cc[i] = cc[i];
    }
    unsafe {
        libc::cfsetispeed(&mut t, flags[4] as libc::speed_t);
        libc::cfsetospeed(&mut t, flags[5] as libc::speed_t);
    }

    if let Some(err) = raise_if_errno(unsafe { libc::tcsetattr(fd, when, &t) }) {
        return err;
    }
    MbValue::none()
}

fn mb_tcsendbreak(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error("tcsendbreak() takes exactly 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let duration = match extract_c_int_arg(args[1], "duration") {
        Ok(d) => d,
        Err(err) => return err,
    };
    if let Some(err) = raise_if_errno(unsafe { libc::tcsendbreak(fd, duration) }) {
        return err;
    }
    MbValue::none()
}

fn mb_tcdrain(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error("tcdrain() takes exactly 1 argument");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    if let Some(err) = raise_if_errno(unsafe { libc::tcdrain(fd) }) {
        return err;
    }
    MbValue::none()
}

fn mb_tcflush(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error("tcflush() takes exactly 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let queue = match extract_c_int_arg(args[1], "queue selector") {
        Ok(q) => q,
        Err(err) => return err,
    };
    if let Some(err) = raise_if_errno(unsafe { libc::tcflush(fd, queue) }) {
        return err;
    }
    MbValue::none()
}

fn mb_tcflow(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error("tcflow() takes exactly 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let action = match extract_c_int_arg(args[1], "action") {
        Ok(a) => a,
        Err(err) => return err,
    };
    if let Some(err) = raise_if_errno(unsafe { libc::tcflow(fd, action) }) {
        return err;
    }
    MbValue::none()
}

fn mb_tcgetwinsize(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error("tcgetwinsize() takes exactly 1 argument");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let rv = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ as c_ulong, &mut ws as *mut libc::winsize) };
    if let Some(err) = raise_if_errno(rv as c_int) {
        return err;
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(ws.ws_row as i64),
        MbValue::from_int(ws.ws_col as i64),
    ]))
}

fn mb_tcsetwinsize(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error("tcsetwinsize() takes exactly 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let Some(items) = extract_seq_items(args[1]) else {
        return raise_type_error("tcsetwinsize, arg 2: must be 2 element list or tuple");
    };
    if items.len() != 2 {
        return raise_type_error("tcsetwinsize, arg 2: must be 2 element list or tuple");
    }
    let row = match extract_c_int_arg(items[0], "row") {
        Ok(v) => v,
        Err(err) => return err,
    };
    let col = match extract_c_int_arg(items[1], "col") {
        Ok(v) => v,
        Err(err) => return err,
    };

    // Preserve xpixel/ypixel via a get-modify-set round trip (matches
    // CPython's `termios.c` implementation).
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let get_rv = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ as c_ulong, &mut ws as *mut libc::winsize) };
    if let Some(err) = raise_if_errno(get_rv as c_int) {
        return err;
    }
    ws.ws_row = row as libc::c_ushort;
    ws.ws_col = col as libc::c_ushort;
    let set_rv = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ as c_ulong, &ws as *const libc::winsize) };
    if let Some(err) = raise_if_errno(set_rv as c_int) {
        return err;
    }
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_cc_byte_truncates_ints() {
        assert_eq!(extract_cc_byte(MbValue::from_int(4)).unwrap(), 4);
        assert_eq!(extract_cc_byte(MbValue::from_int(256)).unwrap(), 0);
        assert_eq!(extract_cc_byte(MbValue::from_int(-1)).unwrap(), 0xff);
    }

    #[test]
    fn extract_cc_byte_accepts_single_byte_bytes() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![0x7f]));
        assert_eq!(extract_cc_byte(b).unwrap(), 0x7f);
    }

    #[test]
    fn extract_cc_byte_rejects_wrong_length_bytes() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        assert!(extract_cc_byte(b).is_err());
        let b2 = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2]));
        assert!(extract_cc_byte(b2).is_err());
    }
}
