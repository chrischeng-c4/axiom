use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// `mmap` module for Mamba (#871).
///
/// Provides `mmap.mmap(fileno, length, flags=MAP_SHARED,
/// prot=PROT_READ|PROT_WRITE, access=ACCESS_DEFAULT, offset=0)` backed by
/// real POSIX `mmap(2)` / `munmap(2)` / `msync(2)` / `madvise(2)` syscalls
/// via the `libc` crate.
///
/// Dependency note (#871): `memmap2` is NOT a dependency of the `mamba`
/// crate itself (only `projects/lumen` and `vendor/cranelift-jit` use it
/// elsewhere in the workspace) — per the issue's explicit fallback
/// instruction, this uses raw `libc` mmap/munmap directly rather than adding
/// an unauthorized dependency. `libc = "0.2"` is already a direct `mamba`
/// dependency (see `Cargo.toml`).
///
/// mamba's `open()` / `os.open()` hand back TABLE-SURROGATE ids, not real
/// OS-level fds; real `libc::mmap` needs a genuine fd, so construction
/// resolves the caller's `fileno` through `file_io::mb_file_raw_fd` /
/// `os_mod::mb_os_fd_raw_fd` (falling back to treating the value as an
/// already-real fd for `os.open`-external or C-level fds) and then
/// `dup(2)`s it — matching real CPython's mmap object, which holds an
/// independent fd so the mapping outlives the originating file object's
/// `close()` (see `tests/cpython/behavior/std-libs/mmap/mmap_tests__test_basic.py`).
///
/// Sequence/state is a native side table (`MMAPS`, keyed by a monotonic id
/// stored in the instance's `_id` field, not the object pointer — avoids
/// pointer-reuse-after-free ambiguity). `object.__new__(mmap)` (used by all
/// of this module's force-typed argument-contract fixtures) yields a BARE
/// instance with no `_id`, so every method type-checks its arguments BEFORE
/// touching state — matching the fixtures, which call methods directly on
/// such a bare instance and expect `TypeError` for a wrong-typed argument.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::raw::c_void;

// ── ACCESS_* constants (CPython mmap.ACCESS_*) ──

const ACCESS_DEFAULT: i64 = 0;
const ACCESS_READ: i64 = 1;
const ACCESS_WRITE: i64 = 2;
const ACCESS_COPY: i64 = 3;

// ── native side table: mmap id -> mapping state ──

struct MmapState {
    ptr: *mut u8,
    len: usize,
    pos: usize,
    access: i64,
    fd: i32, // dup'd fd (this mmap's OWN reference); -1 if anonymous
    anonymous: bool,
    closed: bool,
    #[allow(dead_code)]
    name: Option<String>,
}

thread_local! {
    static MMAPS: RefCell<HashMap<u64, MmapState>> = RefCell::new(HashMap::new());
    static NEXT_MMAP_ID: Cell<u64> = Cell::new(1);
}

fn with_state<R>(id: u64, f: impl FnOnce(&MmapState) -> R) -> Option<R> {
    MMAPS.with(|m| m.borrow().get(&id).map(f))
}

fn with_state_mut<R>(id: u64, f: impl FnOnce(&mut MmapState) -> R) -> Option<R> {
    MMAPS.with(|m| m.borrow_mut().get_mut(&id).map(f))
}

// ── shared error/argument helpers (mirrors select_mod.rs / io_mod.rs) ──

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn raise_exc(kind: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(kind), new_str(msg.into()));
    MbValue::none()
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    raise_exc("TypeError", msg)
}

fn raise_value_error(msg: impl Into<String>) -> MbValue {
    raise_exc("ValueError", msg)
}

fn raise_os_errno_current() -> MbValue {
    let err = std::io::Error::last_os_error();
    let errno = err.raw_os_error().unwrap_or(0);
    raise_exc("OSError", format!("[Errno {errno}] {err}"))
}

fn raise_os_errno_value(errno: i32) -> MbValue {
    let err = std::io::Error::from_raw_os_error(errno);
    raise_exc("OSError", format!("[Errno {errno}] {err}"))
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

/// Short Python type name for error messages.
fn type_name_of(val: MbValue) -> String {
    if val.is_bool() {
        return "bool".into();
    }
    if val.is_int() {
        return "int".into();
    }
    if val.is_none() {
        return "NoneType".into();
    }
    if val.as_float().is_some() {
        return "float".into();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str".into(),
                ObjData::Bytes(_) => "bytes".into(),
                ObjData::ByteArray(_) => "bytearray".into(),
                ObjData::List(_) => "list".into(),
                ObjData::Dict(_) => "dict".into(),
                ObjData::Tuple(_) => "tuple".into(),
                ObjData::Instance { class_name, .. } => class_name.clone(),
                _ => "object".into(),
            };
        }
    }
    "object".into()
}

fn is_dict(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// True when `v` is a slice subscript key. mmap's `__getitem__`/
/// `__setitem__`/`__delitem__` are registered as native-stub (variadic)
/// methods (see `register_variadic_func` in `register_mmap_class`), so per
/// `obj_has_user_dunder` in `runtime/class.rs`, `mb_obj_getitem`/
/// `mb_obj_setitem` normalize a `m[a:b]` slice subscript into a `(start,
/// stop, step)` 3-tuple *before* it reaches this module — it never arrives as
/// a real `slice` instance. Accept the tuple form (the only form that
/// actually occurs) and, defensively, a genuine slice instance too (e.g. a
/// direct `m.__getitem__(slice(...))` call some other caller might construct).
fn is_slice(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::Tuple(items) => items.len() == 3,
                ObjData::Instance { class_name, .. } => class_name == "slice",
                _ => false,
            }
        })
        .unwrap_or(false)
}

/// Positional args of an instance method (the runtime passes a List/Tuple).
fn method_pos(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// Split method positional args from a trailing kwargs dict.
fn split_method_kwargs(args: MbValue) -> (Vec<MbValue>, Option<MbValue>) {
    let mut items = method_pos(args);
    if let Some(&last) = items.last() {
        if is_dict(last) {
            items.pop();
            return (items, Some(last));
        }
    }
    (items, None)
}

/// Split a flat (args_ptr, nargs) slice's trailing kwargs dict (module-level
/// `mmap.mmap(...)` call convention).
fn split_flat_kwargs(a: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(&last) = a.last() {
        if is_dict(last) {
            return (a[..a.len() - 1].to_vec(), Some(last));
        }
    }
    (a.to_vec(), None)
}

fn kw_value(kwargs: Option<MbValue>, name: &str) -> Option<MbValue> {
    let dict = kwargs?;
    let sentinel = MbValue::from_bits(u64::MAX);
    let v = super::super::dict_ops::mb_dict_get(dict, new_str(name), sentinel);
    if v.to_bits() == sentinel.to_bits() {
        None
    } else {
        Some(v)
    }
}

fn slice_fields(key: MbValue) -> (Option<i64>, Option<i64>, Option<i64>) {
    let conv = |v: MbValue| -> Option<i64> {
        if v.is_none() {
            None
        } else {
            v.as_int_pyint()
        }
    };
    if let Some(ptr) = key.as_ptr() {
        unsafe {
            match &(*ptr).data {
                // The normalized (start, stop, step) tuple form — see
                // `is_slice` for why this is the form actually received.
                ObjData::Tuple(items) if items.len() == 3 => {
                    return (conv(items[0]), conv(items[1]), conv(items[2]));
                }
                ObjData::Instance { ref fields, .. } => {
                    let g = fields.read().unwrap();
                    let get = |name: &str| g.get(name).copied().and_then(conv);
                    return (get("start"), get("stop"), get("step"));
                }
                _ => {}
            }
        }
    }
    (None, None, None)
}

/// Python's `slice.indices()` normalization algorithm (mirrors
/// `PySlice_AdjustIndices`). Returns `(start, stop, step, slicelength)`.
fn slice_indices(
    len: i64,
    start: Option<i64>,
    stop: Option<i64>,
    step: Option<i64>,
) -> Result<(i64, i64, i64, i64), MbValue> {
    let step = step.unwrap_or(1);
    if step == 0 {
        return Err(raise_value_error("slice step cannot be zero"));
    }
    let (lower, upper) = if step < 0 {
        (-1i64, len - 1)
    } else {
        (0i64, len)
    };
    let clamp = |mut s: i64| -> i64 {
        if s < 0 {
            s += len;
            if s < lower {
                s = lower;
            }
        } else if s > upper {
            s = upper;
        }
        s
    };
    let start_v = match start {
        None => {
            if step < 0 {
                upper
            } else {
                lower
            }
        }
        Some(s) => clamp(s),
    };
    let stop_v = match stop {
        None => {
            if step < 0 {
                lower
            } else {
                upper
            }
        }
        Some(s) => clamp(s),
    };
    let slicelength = if step > 0 {
        if stop_v > start_v {
            (stop_v - start_v + step - 1) / step
        } else {
            0
        }
    } else if stop_v < start_v {
        (start_v - stop_v - step - 1) / (-step)
    } else {
        0
    };
    Ok((start_v, stop_v, step, slicelength.max(0)))
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn rfind_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(hay.len());
    }
    if needle.len() > hay.len() {
        return None;
    }
    hay.windows(needle.len())
        .enumerate()
        .rev()
        .find(|(_, w)| *w == needle)
        .map(|(i, _)| i)
}

fn normalize_index(i: i64, len: i64) -> i64 {
    if i < 0 {
        (i + len).max(0)
    } else {
        i
    }
}

// ── genuine-fd resolution (#871: bridge mamba's table-surrogate ids to a
//    real OS-level fd that `libc::mmap` can use) ──

fn resolve_fileno_to_fd(fileno: i64) -> i32 {
    if let Some(fd) = super::super::file_io::mb_file_raw_fd(MbValue::from_int(fileno)) {
        return fd;
    }
    if let Some(fd) = super::os_mod::mb_os_fd_raw_fd(fileno) {
        return fd;
    }
    fileno as i32
}

// ── instance <-> native state plumbing ──

fn new_mmap_instance(id: u64) -> MbValue {
    let inst_ptr = MbObject::new_instance("mmap".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert("__class__".to_string(), new_str("mmap"));
            map.insert("_id".to_string(), MbValue::from_int(id as i64));
            map.insert("closed".to_string(), MbValue::from_bool(false));
        }
    }
    MbValue::from_ptr(inst_ptr)
}

fn get_id(self_v: MbValue) -> Option<u64> {
    let ptr = self_v.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            return fields
                .read()
                .unwrap()
                .get("_id")
                .and_then(|v| v.as_int())
                .map(|i| i as u64);
        }
    }
    None
}

/// Look up the live, open state id for `self_v`, raising `ValueError` (the
/// same message CPython uses for a use-after-close mmap) both when the
/// instance never held real state (`object.__new__(mmap)` bare shell) and
/// when it has been `close()`d.
fn require_open(self_v: MbValue) -> Result<u64, MbValue> {
    match get_id(self_v) {
        Some(id) => {
            let closed = with_state(id, |s| s.closed).unwrap_or(true);
            if closed {
                Err(raise_value_error("mmap closed or invalid"))
            } else {
                Ok(id)
            }
        }
        None => Err(raise_value_error("mmap closed or invalid")),
    }
}

fn check_writable(id: u64) -> Option<MbValue> {
    let readonly = with_state(id, |s| s.access == ACCESS_READ).unwrap_or(false);
    if readonly {
        Some(raise_type_error("mmap can't modify a readonly memory map."))
    } else {
        None
    }
}

// ── construction ──

/// Shared validation + real `mmap(2)` construction for both entry points
/// (`mmap.mmap(...)` module call and `obj.__new__(...)` bound-instance
/// call). POSIX signature order:
/// `(fileno, length, flags=MAP_SHARED, prot=PROT_READ|PROT_WRITE,
/// access=ACCESS_DEFAULT, offset=0)`.
fn build_mmap(pos: &[MbValue], kwargs: Option<MbValue>) -> Result<MbValue, MbValue> {
    let arg = |i: usize, name: &str| -> Option<MbValue> {
        pos.get(i).copied().or_else(|| kw_value(kwargs, name))
    };

    let Some(fileno_v) = arg(0, "fileno") else {
        return Err(raise_type_error(
            "mmap() missing required argument 'fileno' (pos 1)",
        ));
    };
    let Some(fileno) = fileno_v.as_int_pyint() else {
        return Err(raise_type_error("mmap: fileno must be an integer"));
    };

    let Some(length_v) = arg(1, "length") else {
        return Err(raise_type_error(
            "mmap() missing required argument 'length' (pos 2)",
        ));
    };
    let Some(length_raw) = length_v.as_int_pyint() else {
        return Err(raise_type_error("mmap: length must be an integer"));
    };
    if length_raw < 0 {
        return Err(raise_exc(
            "OverflowError",
            "memory mapped length must be positive",
        ));
    }

    let flags = match arg(2, "flags") {
        Some(v) => match v.as_int_pyint() {
            Some(i) => i as i32,
            None => return Err(raise_type_error("mmap: flags must be an integer")),
        },
        None => libc::MAP_SHARED,
    };
    let prot = match arg(3, "prot") {
        Some(v) => match v.as_int_pyint() {
            Some(i) => i as i32,
            None => return Err(raise_type_error("mmap: prot must be an integer")),
        },
        None => libc::PROT_READ | libc::PROT_WRITE,
    };
    let access = match arg(4, "access") {
        Some(v) => match v.as_int_pyint() {
            Some(i) => i,
            None => return Err(raise_type_error("mmap: access must be an integer")),
        },
        None => ACCESS_DEFAULT,
    };
    let offset_raw = match arg(5, "offset") {
        Some(v) => match v.as_int_pyint() {
            Some(i) => i,
            None => return Err(raise_type_error("mmap: offset must be an integer")),
        },
        None => 0,
    };
    if offset_raw < 0 {
        return Err(raise_exc(
            "OverflowError",
            "memory mapped offset must be positive",
        ));
    }

    // access vs flags/prot conflict — literal-value comparison (matches CPython).
    if access != ACCESS_DEFAULT
        && (flags != libc::MAP_SHARED || prot != (libc::PROT_READ | libc::PROT_WRITE))
    {
        return Err(raise_value_error(
            "mmap can't specify both access and flags, prot.",
        ));
    }
    let (flags, prot) = if access != ACCESS_DEFAULT {
        match access {
            ACCESS_READ => (libc::MAP_SHARED, libc::PROT_READ),
            ACCESS_WRITE => (libc::MAP_SHARED, libc::PROT_READ | libc::PROT_WRITE),
            ACCESS_COPY => (libc::MAP_PRIVATE, libc::PROT_READ | libc::PROT_WRITE),
            _ => return Err(raise_value_error("mmap invalid access parameter.")),
        }
    } else {
        (flags, prot)
    };

    let anonymous = fileno == -1;
    let mut length = length_raw as u64;
    let mut real_fd: i32 = -1;

    if anonymous {
        if length == 0 || offset_raw != 0 {
            return Err(raise_os_errno_value(libc::EINVAL));
        }
    } else {
        let base_fd = resolve_fileno_to_fd(fileno);
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::fstat(base_fd, &mut st) };
        if rc != 0 {
            return Err(raise_os_errno_current());
        }
        let file_size = st.st_size.max(0) as u64;
        if length == 0 {
            length = file_size.saturating_sub(offset_raw as u64);
        } else if offset_raw as u64 + length > file_size {
            return Err(raise_value_error("mmap length is greater than file size"));
        }
        let dup_fd = unsafe { libc::dup(base_fd) };
        if dup_fd < 0 {
            return Err(raise_os_errno_current());
        }
        real_fd = dup_fd;
    }

    let map_len = length as usize;
    let raw_fd_for_mmap = if anonymous { -1 } else { real_fd };
    let mmap_flags = if anonymous {
        flags | libc::MAP_ANON
    } else {
        flags
    };

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            map_len,
            prot,
            mmap_flags,
            raw_fd_for_mmap,
            offset_raw as libc::off_t,
        )
    };
    if ptr == libc::MAP_FAILED {
        if real_fd >= 0 {
            unsafe {
                libc::close(real_fd);
            }
        }
        return Err(raise_os_errno_current());
    }

    let id = NEXT_MMAP_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    MMAPS.with(|m| {
        m.borrow_mut().insert(
            id,
            MmapState {
                ptr: ptr as *mut u8,
                len: map_len,
                pos: 0,
                access,
                fd: real_fd,
                anonymous,
                closed: false,
                name: None,
            },
        );
    });

    let inst = new_mmap_instance(id);
    Ok(inst)
}

// ── module-level constructor entry point (flat ABI) ──

unsafe extern "C" fn dispatch_mmap_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let a = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let (pos, kwargs) = split_flat_kwargs(a);
    match build_mmap(&pos, kwargs) {
        Ok(v) => v,
        Err(e) => e,
    }
}

// ── bound-instance-method entry point (self+args ABI): `obj.__new__(...)` ──

unsafe extern "C" fn m_new(_self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kwargs) = split_method_kwargs(args);
    match build_mmap(&pos, kwargs) {
        Ok(v) => v,
        Err(e) => e,
    }
}

// ── sequence protocol ──

unsafe extern "C" fn m_len(self_v: MbValue, _args: MbValue) -> MbValue {
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    MbValue::from_int(with_state(id, |s| s.len as i64).unwrap_or(0))
}

fn getitem_index(id: u64, idx: i64) -> MbValue {
    let result = MMAPS.with(|m| {
        let map = m.borrow();
        let s = map.get(&id)?;
        let len = s.len as i64;
        let real = if idx < 0 { idx + len } else { idx };
        if real < 0 || real >= len {
            return None;
        }
        Some(unsafe { *s.ptr.add(real as usize) })
    });
    match result {
        Some(b) => MbValue::from_int(b as i64),
        None => raise_exc("IndexError", "mmap index out of range"),
    }
}

fn getitem_slice(id: u64, key: MbValue) -> MbValue {
    let (start, stop, step) = slice_fields(key);
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let (start_v, _stop_v, step_v, slicelen) = match slice_indices(len, start, stop, step) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let bytes = MMAPS.with(|m| {
        let map = m.borrow();
        let Some(s) = map.get(&id) else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(slicelen.max(0) as usize);
        let mut i = start_v;
        for _ in 0..slicelen {
            out.push(unsafe { *s.ptr.add(i as usize) });
            i += step_v;
        }
        out
    });
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

unsafe extern "C" fn m_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(key) = pos.first().copied() else {
        return raise_type_error("__getitem__() missing required argument: 'key' (pos 1)");
    };
    if is_slice(key) {
        let id = match require_open(self_v) {
            Ok(v) => v,
            Err(e) => return e,
        };
        return getitem_slice(id, key);
    }
    let Some(idx) = key.as_int_pyint() else {
        return raise_type_error("mmap indices must be integers");
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    getitem_index(id, idx)
}

fn setitem_index(id: u64, idx: i64, byte: u8) -> MbValue {
    let ok = MMAPS.with(|m| {
        let map = m.borrow();
        let Some(s) = map.get(&id) else {
            return false;
        };
        let len = s.len as i64;
        let real = if idx < 0 { idx + len } else { idx };
        if real < 0 || real >= len {
            return false;
        }
        unsafe {
            *s.ptr.add(real as usize) = byte;
        }
        true
    });
    if ok {
        MbValue::none()
    } else {
        raise_exc("IndexError", "mmap index out of range")
    }
}

fn setitem_slice(id: u64, key: MbValue, bytes: Vec<u8>) -> MbValue {
    let (start, stop, step) = slice_fields(key);
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let (start_v, _stop_v, step_v, slicelen) = match slice_indices(len, start, stop, step) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if bytes.len() as i64 != slicelen {
        return raise_exc("IndexError", "mmap slice assignment is wrong size");
    }
    MMAPS.with(|m| {
        let map = m.borrow();
        if let Some(s) = map.get(&id) {
            let mut i = start_v;
            for &b in &bytes {
                unsafe {
                    *s.ptr.add(i as usize) = b;
                }
                i += step_v;
            }
        }
    });
    MbValue::none()
}

unsafe extern "C" fn m_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(key) = pos.first().copied() else {
        return raise_type_error("__setitem__() missing required argument: 'key' (pos 1)");
    };
    let Some(value) = pos.get(1).copied() else {
        return raise_type_error("__setitem__() missing required argument: 'value' (pos 2)");
    };
    let slice_mode = is_slice(key);
    if !slice_mode && key.as_int_pyint().is_none() {
        return raise_type_error("mmap indices must be integers");
    }

    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Some(e) = check_writable(id) {
        return e;
    }

    if slice_mode {
        let Some(bytes) = super::super::builtins::try_bytes_like(value) else {
            return raise_type_error(format!(
                "a bytes-like object is required, not '{}'",
                type_name_of(value)
            ));
        };
        return setitem_slice(id, key, bytes);
    }

    let idx = key.as_int_pyint().unwrap();
    let byte = if let Some(iv) = value.as_int_pyint() {
        if !(0..256).contains(&iv) {
            return raise_value_error("mmap item value must be in range(0, 256)");
        }
        iv as u8
    } else if let Some(b) = super::super::builtins::try_bytes_like(value) {
        if b.len() != 1 {
            return raise_type_error("mmap item value must be an int");
        }
        b[0]
    } else {
        return raise_type_error("mmap item value must be an int");
    };
    setitem_index(id, idx, byte)
}

unsafe extern "C" fn m_delitem(_self_v: MbValue, _args: MbValue) -> MbValue {
    // CPython: mmap objects don't support item deletion, regardless of key.
    raise_type_error("mmap doesn't support item deletion")
}

// ── find / rfind ──

fn m_find_impl(self_v: MbValue, pos: &[MbValue], reverse: bool) -> MbValue {
    let name = if reverse { "rfind" } else { "find" };
    let Some(sub_v) = pos.first().copied() else {
        return raise_type_error(format!("{name}() missing required argument: 'sub' (pos 1)"));
    };
    let Some(sub) = super::super::builtins::try_bytes_like(sub_v) else {
        return raise_type_error(format!(
            "argument should be a bytes-like object or ASCII string, not '{}'",
            type_name_of(sub_v)
        ));
    };
    let start_i = match pos.get(1).copied() {
        None => None,
        Some(v) => match v.as_int_pyint() {
            Some(i) => Some(i),
            None => return raise_type_error("start must be an integer"),
        },
    };
    let end_i = match pos.get(2).copied() {
        None => None,
        Some(v) => match v.as_int_pyint() {
            Some(i) => Some(i),
            None => return raise_type_error("end must be an integer"),
        },
    };

    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let start = normalize_index(start_i.unwrap_or(0), len).clamp(0, len);
    let end = normalize_index(end_i.unwrap_or(len), len).clamp(0, len);

    let result = MMAPS.with(|m| {
        let map = m.borrow();
        let s = map.get(&id)?;
        if start > end {
            return None;
        }
        let full = unsafe { std::slice::from_raw_parts(s.ptr, s.len) };
        let window = &full[start as usize..end as usize];
        let off = if reverse {
            rfind_subslice(window, &sub)
        } else {
            find_subslice(window, &sub)
        };
        off.map(|o| start as usize + o)
    });
    match result {
        Some(off) => MbValue::from_int(off as i64),
        None => MbValue::from_int(-1),
    }
}

unsafe extern "C" fn m_find(self_v: MbValue, args: MbValue) -> MbValue {
    m_find_impl(self_v, &method_pos(args), false)
}

unsafe extern "C" fn m_rfind(self_v: MbValue, args: MbValue) -> MbValue {
    m_find_impl(self_v, &method_pos(args), true)
}

// ── read / read_byte / readline / write / write_byte ──

unsafe extern "C" fn m_read(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let n = match pos.first().copied() {
        None => None,
        Some(v) if v.is_none() => None,
        Some(v) => match v.as_int_pyint() {
            Some(i) => Some(i),
            None => return raise_type_error("read() argument must be int or None"),
        },
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let bytes = MMAPS.with(|m| {
        let mut map = m.borrow_mut();
        let Some(s) = map.get_mut(&id) else {
            return Vec::new();
        };
        let avail = s.len.saturating_sub(s.pos);
        let take = match n {
            Some(v) if v >= 0 => (v as usize).min(avail),
            _ => avail,
        };
        let out = unsafe { std::slice::from_raw_parts(s.ptr.add(s.pos), take) }.to_vec();
        s.pos += take;
        out
    });
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

unsafe extern "C" fn m_read_byte(self_v: MbValue, _args: MbValue) -> MbValue {
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let result = MMAPS.with(|m| {
        let mut map = m.borrow_mut();
        let s = map.get_mut(&id)?;
        if s.pos >= s.len {
            return None;
        }
        let b = unsafe { *s.ptr.add(s.pos) };
        s.pos += 1;
        Some(b)
    });
    match result {
        Some(b) => MbValue::from_int(b as i64),
        None => raise_value_error("read byte out of range"),
    }
}

unsafe extern "C" fn m_readline(self_v: MbValue, _args: MbValue) -> MbValue {
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let bytes = MMAPS.with(|m| {
        let mut map = m.borrow_mut();
        let Some(s) = map.get_mut(&id) else {
            return Vec::new();
        };
        if s.pos >= s.len {
            return Vec::new();
        }
        let full = unsafe { std::slice::from_raw_parts(s.ptr, s.len) };
        let rest = &full[s.pos..];
        let end = rest
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| p + 1)
            .unwrap_or(rest.len());
        let out = rest[..end].to_vec();
        s.pos += end;
        out
    });
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

unsafe extern "C" fn m_write(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(data_v) = pos.first().copied() else {
        return raise_type_error("write() missing required argument: 'bytes' (pos 1)");
    };
    let Some(data) = super::super::builtins::try_bytes_like(data_v) else {
        return raise_type_error(format!(
            "a bytes-like object is required, not '{}'",
            type_name_of(data_v)
        ));
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Some(e) = check_writable(id) {
        return e;
    }
    let result = MMAPS.with(|m| {
        let mut map = m.borrow_mut();
        let Some(s) = map.get_mut(&id) else {
            return Err(());
        };
        if s.pos + data.len() > s.len {
            return Err(());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), s.ptr.add(s.pos), data.len());
        }
        s.pos += data.len();
        Ok(data.len())
    });
    match result {
        Ok(n) => MbValue::from_int(n as i64),
        Err(()) => raise_value_error("data out of range"),
    }
}

unsafe extern "C" fn m_write_byte(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(byte_v) = pos.first().copied() else {
        return raise_type_error("write_byte() missing required argument: 'byte' (pos 1)");
    };
    let Some(iv) = byte_v.as_int_pyint() else {
        return raise_type_error("write_byte() argument must be int");
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Some(e) = check_writable(id) {
        return e;
    }
    if !(0..256).contains(&iv) {
        return raise_value_error("byte must be in range(0, 256)");
    }
    let ok = MMAPS.with(|m| {
        let mut map = m.borrow_mut();
        let Some(s) = map.get_mut(&id) else {
            return false;
        };
        if s.pos >= s.len {
            return false;
        }
        unsafe {
            *s.ptr.add(s.pos) = iv as u8;
        }
        s.pos += 1;
        true
    });
    if ok {
        MbValue::none()
    } else {
        raise_value_error("write byte out of range")
    }
}

// ── seek / tell / flush / resize / size / move ──

unsafe extern "C" fn m_seek(self_v: MbValue, args: MbValue) -> MbValue {
    let pos_args = method_pos(args);
    let Some(pos_v) = pos_args.first().copied() else {
        return raise_type_error("seek() missing required argument: 'pos' (pos 1)");
    };
    let Some(pos_i) = pos_v.as_int_pyint() else {
        return raise_type_error("seek() argument must be int");
    };
    let whence = match pos_args.get(1).copied() {
        None => 0i64,
        Some(v) => match v.as_int_pyint() {
            Some(i) => i,
            None => return raise_type_error("seek() argument must be int"),
        },
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let cur = with_state(id, |s| s.pos as i64).unwrap_or(0);
    let new_pos = match whence {
        0 => pos_i,
        1 => cur + pos_i,
        2 => len + pos_i,
        _ => return raise_value_error("unknown seek type"),
    };
    if new_pos < 0 || new_pos > len {
        return raise_value_error("seek out of range");
    }
    with_state_mut(id, |s| s.pos = new_pos as usize);
    MbValue::none()
}

unsafe extern "C" fn m_tell(self_v: MbValue, _args: MbValue) -> MbValue {
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    MbValue::from_int(with_state(id, |s| s.pos as i64).unwrap_or(0))
}

unsafe extern "C" fn m_flush(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let offset = match pos.first().copied() {
        None => 0i64,
        Some(v) => match v.as_int_pyint() {
            Some(i) => i,
            None => return raise_type_error("flush() argument must be int"),
        },
    };
    let size = match pos.get(1).copied() {
        None => None,
        Some(v) => match v.as_int_pyint() {
            Some(i) => Some(i),
            None => return raise_type_error("flush() argument must be int"),
        },
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let sz = size.unwrap_or(len - offset);
    if offset < 0 || sz < 0 || offset + sz > len {
        return raise_value_error("flush values out of range");
    }
    let rc = MMAPS.with(|m| {
        let map = m.borrow();
        let Some(s) = map.get(&id) else { return 0 };
        unsafe {
            libc::msync(
                s.ptr.add(offset as usize) as *mut c_void,
                sz as usize,
                libc::MS_SYNC,
            )
        }
    });
    if rc != 0 {
        return raise_os_errno_current();
    }
    MbValue::none()
}

unsafe extern "C" fn m_resize(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(newsize_v) = pos.first().copied() else {
        return raise_type_error("resize() missing required argument: 'newsize' (pos 1)");
    };
    if newsize_v.as_int_pyint().is_none() {
        return raise_type_error("resize() argument must be int");
    }
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let access = with_state(id, |s| s.access).unwrap_or(ACCESS_WRITE);
    if access == ACCESS_READ || access == ACCESS_COPY {
        return raise_type_error("mmap can't resize a readonly or copy-on-write memory map.");
    }
    // Darwin/macOS has no mremap(2); real CPython raises the same SystemError here.
    raise_exc("SystemError", "mmap: resizing not available--no mremap()")
}

unsafe extern "C" fn m_size(self_v: MbValue, _args: MbValue) -> MbValue {
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (fd, anon, len) = with_state(id, |s| (s.fd, s.anonymous, s.len)).unwrap_or((-1, true, 0));
    if anon || fd < 0 {
        return MbValue::from_int(len as i64);
    }
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::fstat(fd, &mut st) };
    if rc != 0 {
        return raise_os_errno_current();
    }
    MbValue::from_int(st.st_size)
}

unsafe extern "C" fn m_move(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(dest_v) = pos.first().copied() else {
        return raise_type_error("move() missing required argument: 'dest' (pos 1)");
    };
    let Some(dest) = dest_v.as_int_pyint() else {
        return raise_type_error("move() argument 1 must be int");
    };
    let Some(src_v) = pos.get(1).copied() else {
        return raise_type_error("move() missing required argument: 'src' (pos 2)");
    };
    let Some(src) = src_v.as_int_pyint() else {
        return raise_type_error("move() argument 2 must be int");
    };
    let Some(count_v) = pos.get(2).copied() else {
        return raise_type_error("move() missing required argument: 'count' (pos 3)");
    };
    let Some(count) = count_v.as_int_pyint() else {
        return raise_type_error("move() argument 3 must be int");
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Some(e) = check_writable(id) {
        return e;
    }
    if dest < 0 || src < 0 || count < 0 {
        return raise_value_error("negative move values not supported");
    }
    let ok = with_state(id, |s| {
        (dest as usize).saturating_add(count as usize) <= s.len
            && (src as usize).saturating_add(count as usize) <= s.len
    })
    .unwrap_or(false);
    if !ok {
        return raise_value_error("source, destination, or count out of range");
    }
    MMAPS.with(|m| {
        let map = m.borrow();
        if let Some(s) = map.get(&id) {
            unsafe {
                std::ptr::copy(
                    s.ptr.add(src as usize),
                    s.ptr.add(dest as usize),
                    count as usize,
                );
            }
        }
    });
    MbValue::none()
}

// ── close / context manager ──

unsafe extern "C" fn m_close(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(id) = get_id(self_v) {
        MMAPS.with(|m| {
            let mut map = m.borrow_mut();
            if let Some(s) = map.get_mut(&id) {
                if !s.closed {
                    unsafe {
                        libc::munmap(s.ptr as *mut c_void, s.len);
                    }
                    if s.fd >= 0 {
                        unsafe {
                            libc::close(s.fd);
                        }
                    }
                    s.closed = true;
                }
            }
        });
    }
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .write()
                    .unwrap()
                    .insert("closed".to_string(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn m_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(self_v);
    }
    self_v
}

/// `exc_type: Unused` — typeshed types it as `type[BaseException] | None`.
/// Accept `None` or anything that resolves to a class name (real exception
/// classes, native type func-ptrs, `type` instances); reject a plain
/// instance (e.g. `_W()`), matching the force-typed argument contract.
unsafe extern "C" fn m_exit(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let exc_type = pos.first().copied().unwrap_or_else(MbValue::none);
    if !exc_type.is_none() && super::super::class::resolve_class_name(exc_type).is_none() {
        return raise_type_error("__exit__ requires a type or None");
    }
    m_close(self_v, args)
}

// ── madvise / set_name ──

unsafe extern "C" fn m_madvise(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(option_v) = pos.first().copied() else {
        return raise_type_error("madvise() missing required argument: 'option' (pos 1)");
    };
    let Some(option) = option_v.as_int_pyint() else {
        return raise_type_error("madvise() argument 1 must be int");
    };
    let start = match pos.get(1).copied() {
        None => 0i64,
        Some(v) => match v.as_int_pyint() {
            Some(i) => i,
            None => return raise_type_error("madvise() argument 2 must be int"),
        },
    };
    let length = match pos.get(2).copied() {
        None => None,
        Some(v) => match v.as_int_pyint() {
            Some(i) => Some(i),
            None => return raise_type_error("madvise() argument 3 must be int"),
        },
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let len = with_state(id, |s| s.len as i64).unwrap_or(0);
    let sz = length.unwrap_or(len - start);
    if start < 0 || sz < 0 || start + sz > len {
        return raise_value_error("madvise start or length out of range");
    }
    let rc = MMAPS.with(|m| {
        let map = m.borrow();
        let Some(s) = map.get(&id) else { return 0 };
        unsafe {
            libc::madvise(
                s.ptr.add(start as usize) as *mut c_void,
                sz as usize,
                option as i32,
            )
        }
    });
    if rc != 0 {
        return raise_os_errno_current();
    }
    MbValue::none()
}

unsafe extern "C" fn m_set_name(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(name_v) = pos.first().copied() else {
        return raise_type_error("set_name() missing required argument: 'name' (pos 1)");
    };
    let Some(name) = extract_str(name_v) else {
        return raise_type_error("set_name() argument must be str");
    };
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    with_state_mut(id, |s| s.name = Some(name));
    MbValue::none()
}

// ── buffer protocol ──

unsafe extern "C" fn m_buffer(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(flags_v) = pos.first().copied() else {
        return raise_type_error("__buffer__() missing required argument: 'flags' (pos 1)");
    };
    if flags_v.as_int_pyint().is_none() {
        return raise_type_error("__buffer__() argument must be int");
    }
    let id = match require_open(self_v) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let snapshot = MMAPS.with(|m| {
        let map = m.borrow();
        let Some(s) = map.get(&id) else {
            return Vec::new();
        };
        unsafe { std::slice::from_raw_parts(s.ptr, s.len) }.to_vec()
    });
    let ba = MbValue::from_ptr(MbObject::new_bytearray(snapshot));
    super::super::builtins::mb_memoryview(ba)
}

unsafe extern "C" fn m_release_buffer(_self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(buf_v) = pos.first().copied() else {
        return raise_type_error(
            "__release_buffer__() missing required argument: 'buffer' (pos 1)",
        );
    };
    let is_mv = buf_v
        .as_ptr()
        .map(|p| unsafe {
            matches!(&(*p).data, ObjData::Instance { class_name, .. } if class_name == "memoryview")
        })
        .unwrap_or(false);
    if !is_mv {
        return raise_type_error("__release_buffer__() argument must be a memoryview");
    }
    MbValue::none()
}

// ── module registration ──

fn register_mmap_class() {
    let methods: Vec<(&str, usize)> = vec![
        ("__new__", m_new as usize),
        ("__len__", m_len as usize),
        ("__getitem__", m_getitem as usize),
        ("__setitem__", m_setitem as usize),
        ("__delitem__", m_delitem as usize),
        ("find", m_find as usize),
        ("rfind", m_rfind as usize),
        ("read", m_read as usize),
        ("read_byte", m_read_byte as usize),
        ("readline", m_readline as usize),
        ("write", m_write as usize),
        ("write_byte", m_write_byte as usize),
        ("seek", m_seek as usize),
        ("tell", m_tell as usize),
        ("flush", m_flush as usize),
        ("resize", m_resize as usize),
        ("size", m_size as usize),
        ("move", m_move as usize),
        ("close", m_close as usize),
        ("__enter__", m_enter as usize),
        ("__exit__", m_exit as usize),
        ("madvise", m_madvise as usize),
        ("set_name", m_set_name as usize),
        ("__buffer__", m_buffer as usize),
        ("__release_buffer__", m_release_buffer as usize),
    ];
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in &methods {
        map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    super::super::class::mb_class_register("mmap", Vec::new(), map);
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_new = dispatch_mmap_new as usize;
    attrs.insert("mmap".to_string(), MbValue::from_func(addr_new));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr_new as u64);
    });
    super::super::module::register_native_type_name(addr_new as u64, "mmap".to_string());

    register_mmap_class();

    // ── ACCESS_* constants ──
    attrs.insert(
        "ACCESS_DEFAULT".to_string(),
        MbValue::from_int(ACCESS_DEFAULT),
    );
    attrs.insert("ACCESS_READ".to_string(), MbValue::from_int(ACCESS_READ));
    attrs.insert("ACCESS_WRITE".to_string(), MbValue::from_int(ACCESS_WRITE));
    attrs.insert("ACCESS_COPY".to_string(), MbValue::from_int(ACCESS_COPY));

    // ── mmap/prot flags (real libc values; match CPython's POSIX surface) ──
    attrs.insert(
        "MAP_SHARED".to_string(),
        MbValue::from_int(libc::MAP_SHARED as i64),
    );
    attrs.insert(
        "MAP_PRIVATE".to_string(),
        MbValue::from_int(libc::MAP_PRIVATE as i64),
    );
    attrs.insert(
        "MAP_ANON".to_string(),
        MbValue::from_int(libc::MAP_ANON as i64),
    );
    attrs.insert(
        "MAP_ANONYMOUS".to_string(),
        MbValue::from_int(libc::MAP_ANON as i64),
    );
    attrs.insert(
        "PROT_READ".to_string(),
        MbValue::from_int(libc::PROT_READ as i64),
    );
    attrs.insert(
        "PROT_WRITE".to_string(),
        MbValue::from_int(libc::PROT_WRITE as i64),
    );
    attrs.insert(
        "PROT_EXEC".to_string(),
        MbValue::from_int(libc::PROT_EXEC as i64),
    );

    // ── madvise() option constants (POSIX-standard, cross-platform in libc). ──
    attrs.insert(
        "MADV_NORMAL".to_string(),
        MbValue::from_int(libc::MADV_NORMAL as i64),
    );
    attrs.insert(
        "MADV_RANDOM".to_string(),
        MbValue::from_int(libc::MADV_RANDOM as i64),
    );
    attrs.insert(
        "MADV_SEQUENTIAL".to_string(),
        MbValue::from_int(libc::MADV_SEQUENTIAL as i64),
    );
    attrs.insert(
        "MADV_WILLNEED".to_string(),
        MbValue::from_int(libc::MADV_WILLNEED as i64),
    );
    attrs.insert(
        "MADV_DONTNEED".to_string(),
        MbValue::from_int(libc::MADV_DONTNEED as i64),
    );

    let pagesize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    attrs.insert("PAGESIZE".to_string(), MbValue::from_int(pagesize));
    attrs.insert(
        "ALLOCATIONGRANULARITY".to_string(),
        MbValue::from_int(pagesize),
    );

    // `mmap.error is OSError` — a literal alias, matching CPython (and the
    // select.error / os.error convention already used elsewhere in mamba).
    attrs.insert("error".to_string(), new_str("OSError"));

    super::register_module("mmap", attrs);
}
