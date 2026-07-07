use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// `select` module for Mamba (#869).
///
/// Provides the CPython 3.12 `select` surface backed by real fd readiness:
///   - `select.select(rlist, wlist, xlist[, timeout])` — real readiness via
///     `poll(2)` (grouping the three fd lists into one poll call).
///   - `select.poll()` / `select.devpoll()` / `select.epoll()` — a shared
///     `poll(2)`-backed engine (`register`/`unregister`/`modify`/`poll`).
///     Real macOS CPython only exposes `poll`; `devpoll`/`epoll` are
///     Solaris/Linux-only. Mamba is force-typed (per repo convention) and
///     registers all three unconditionally so their FileDescriptorLike/
///     timeout type-wall fixtures reach a real TypeError instead of failing
///     at import with `AttributeError`/`ImportError`.
///   - `select.kqueue()` / `select.kevent(...)` — real kqueue(2)/kevent(2)
///     syscalls, `cfg(target_os = "macos")` (matches host reality: kqueue is
///     BSD/macOS-only, same as real CPython).
///   - `select.error` — a literal alias for `OSError` (`select.error is
///     OSError`), matching CPython.
///
/// FileDescriptorLike arguments accept either a plain int fd or any object
/// exposing `.fileno() -> int` (sockets, files, ...), matching typeshed.
use std::collections::HashMap;
use std::os::raw::c_int;

// ── Variadic (args_ptr, nargs) dispatchers — module free functions, class
//    constructors, and classmethod-style "fromfd" bridges all share this ABI.

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            crate::icf_guard!();
            let a = if nargs == 0 || args_ptr.is_null() {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
            };
            $fn(a)
        }
    };
}

disp_variadic!(d_select, mb_select);
disp_variadic!(d_poll_new, poll_engine_new_poll);
disp_variadic!(d_devpoll_new, poll_engine_new_devpoll);
disp_variadic!(d_epoll_new, poll_engine_new_epoll);
disp_variadic!(d_epoll_fromfd, epoll_fromfd);

#[cfg(target_os = "macos")]
disp_variadic!(d_kqueue_new, kqueue_new);
#[cfg(target_os = "macos")]
disp_variadic!(d_kevent_new, kevent_new);
#[cfg(target_os = "macos")]
disp_variadic!(d_kqueue_fromfd, kqueue_fromfd);

// ── Shared error/argument helpers (mirrors fcntl_mod.rs / selectors_mod.rs) ──

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

fn select_errno() -> MbValue {
    let err = std::io::Error::last_os_error();
    let errno = err.raw_os_error().unwrap_or(0);
    raise_exc("OSError", format!("[Errno {errno}] {err}"))
}

/// FileDescriptorLike: an int fd, or any object with `.fileno() -> int`.
/// Mirrors `fcntl_mod::extract_fd` — a plain wrong-typed object (no `fileno`)
/// falls through `mb_call_method` to a non-int result, which `extract_c_int`
/// turns into a `TypeError` (matching the type-wall contract).
fn extract_c_int(value: MbValue, what: &str) -> Result<c_int, MbValue> {
    let Some(raw) = value.as_int_pyint() else {
        return Err(raise_type_error(format!("{what} must be an integer")));
    };
    if raw < c_int::MIN as i64 || raw > c_int::MAX as i64 {
        return Err(raise_value_error(format!("{what} is out of range")));
    }
    Ok(raw as c_int)
}

fn extract_fd(value: MbValue) -> Result<c_int, MbValue> {
    let fd = if value.as_int_pyint().is_some() {
        extract_c_int(value, "file descriptor")?
    } else {
        let method = new_str("fileno");
        let args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let result = super::super::class::mb_call_method(value, method, args);
        extract_c_int(result, "file descriptor")?
    };
    if fd < 0 {
        return Err(raise_value_error(format!(
            "file descriptor cannot be a negative integer ({fd})"
        )));
    }
    Ok(fd)
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

fn is_dict(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
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

/// Recognized-iterable shapes; used for `select()`'s rlist/wlist/xlist and
/// `kqueue.control()`'s changelist. `None` is accepted as "empty" (kqueue
/// allows `changelist=None`). A `_W()`-style plain object raises TypeError.
fn require_iterable(value: MbValue, what: &str) -> Result<Vec<MbValue>, MbValue> {
    if value.is_none() {
        return Ok(Vec::new());
    }
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            if matches!(
                (*ptr).data,
                ObjData::List(_)
                    | ObjData::Tuple(_)
                    | ObjData::Set(_)
                    | ObjData::FrozenSet(_)
                    | ObjData::Str(_)
                    | ObjData::Dict(_)
                    | ObjData::Bytes(_)
                    | ObjData::ByteArray(_)
            ) {
                return Ok(super::super::builtins::extract_items(value));
            }
        }
    }
    Err(raise_type_error(format!("{what} must be iterable")))
}

/// `select.select()`'s timeout is SECONDS (float/int/None); poll(2)'s
/// timeout wants milliseconds. Strict: anything else raises `TypeError`
/// (unlike `selectors_mod::poll_timeout_ms`, which silently defaults).
fn strict_timeout_ms_from_seconds(value: Option<MbValue>) -> Result<c_int, MbValue> {
    let Some(v) = value else { return Ok(-1) };
    if v.is_none() {
        return Ok(-1);
    }
    let secs = if let Some(f) = v.as_float() {
        f
    } else if let Some(i) = v.as_int_pyint() {
        i as f64
    } else {
        return Err(raise_type_error("timeout must be a float, int, or None"));
    };
    if secs < 0.0 {
        return Ok(-1);
    }
    Ok((secs * 1000.0).ceil().min(c_int::MAX as f64) as c_int)
}

/// `poll`/`devpoll`/`epoll` `.poll(timeout)` — timeout is already
/// MILLISECONDS (poll(2) convention), not seconds. Strict-typed.
fn strict_timeout_ms_direct(value: Option<MbValue>) -> Result<c_int, MbValue> {
    let Some(v) = value else { return Ok(-1) };
    if v.is_none() {
        return Ok(-1);
    }
    let ms = if let Some(f) = v.as_float() {
        f
    } else if let Some(i) = v.as_int_pyint() {
        i as f64
    } else {
        return Err(raise_type_error("timeout must be a number or None"));
    };
    if ms < 0.0 {
        return Ok(-1);
    }
    Ok(ms.min(c_int::MAX as f64) as c_int)
}

/// `kqueue.control()`'s timeout is SECONDS (float/int/None -> block).
/// Returns `None` for "no timeout" (block indefinitely).
fn strict_timeout_secs(value: Option<MbValue>) -> Result<Option<f64>, MbValue> {
    let Some(v) = value else { return Ok(None) };
    if v.is_none() {
        return Ok(None);
    }
    if let Some(f) = v.as_float() {
        return Ok(Some(f.max(0.0)));
    }
    if let Some(i) = v.as_int_pyint() {
        return Ok(Some((i as f64).max(0.0)));
    }
    Err(raise_type_error("timeout must be a float, int, or None"))
}

// ── select.select(rlist, wlist, xlist[, timeout]) ──

/// Builds `(ready_r, ready_w, ready_x)` from one `poll(2)` call covering the
/// union of all three fd lists. Preserves the caller's original fileobj
/// values (not just raw fds) in the returned ready lists, matching CPython.
fn mb_select(args: &[MbValue]) -> MbValue {
    if args.len() < 3 {
        return raise_type_error(format!(
            "select() takes at least 3 arguments ({} given)",
            args.len()
        ));
    }
    let rlist = match require_iterable(args[0], "rlist") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let wlist = match require_iterable(args[1], "wlist") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let xlist = match require_iterable(args[2], "xlist") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let timeout_ms = match strict_timeout_ms_from_seconds(args.get(3).copied()) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut r_items: Vec<(MbValue, c_int)> = Vec::new();
    let mut w_items: Vec<(MbValue, c_int)> = Vec::new();
    let mut x_items: Vec<(MbValue, c_int)> = Vec::new();
    for obj in &rlist {
        match extract_fd(*obj) {
            Ok(fd) => r_items.push((*obj, fd)),
            Err(e) => return e,
        }
    }
    for obj in &wlist {
        match extract_fd(*obj) {
            Ok(fd) => w_items.push((*obj, fd)),
            Err(e) => return e,
        }
    }
    for obj in &xlist {
        match extract_fd(*obj) {
            Ok(fd) => x_items.push((*obj, fd)),
            Err(e) => return e,
        }
    }

    let mut fd_events: HashMap<c_int, i16> = HashMap::new();
    for (_, fd) in &r_items {
        *fd_events.entry(*fd).or_insert(0) |= libc::POLLIN;
    }
    for (_, fd) in &w_items {
        *fd_events.entry(*fd).or_insert(0) |= libc::POLLOUT;
    }
    for (_, fd) in &x_items {
        *fd_events.entry(*fd).or_insert(0) |= libc::POLLPRI;
    }

    if fd_events.is_empty() {
        if timeout_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(timeout_ms as u64));
        }
        return triple_result(Vec::new(), Vec::new(), Vec::new());
    }

    let mut pollfds: Vec<libc::pollfd> = fd_events
        .iter()
        .map(|(&fd, &events)| libc::pollfd {
            fd,
            events,
            revents: 0,
        })
        .collect();
    let mut fd_index: HashMap<c_int, usize> = HashMap::new();
    for (i, p) in pollfds.iter().enumerate() {
        fd_index.insert(p.fd, i);
    }

    let rc = unsafe {
        libc::poll(
            pollfds.as_mut_ptr(),
            pollfds.len() as libc::nfds_t,
            timeout_ms,
        )
    };
    if rc < 0 {
        return select_errno();
    }

    let mut r_ready = Vec::new();
    let mut w_ready = Vec::new();
    let mut x_ready = Vec::new();
    for (obj, fd) in &r_items {
        if let Some(&i) = fd_index.get(fd) {
            if pollfds[i].revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) != 0 {
                r_ready.push(*obj);
            }
        }
    }
    for (obj, fd) in &w_items {
        if let Some(&i) = fd_index.get(fd) {
            if pollfds[i].revents & (libc::POLLOUT | libc::POLLERR) != 0 {
                w_ready.push(*obj);
            }
        }
    }
    for (obj, fd) in &x_items {
        if let Some(&i) = fd_index.get(fd) {
            if pollfds[i].revents & (libc::POLLPRI | libc::POLLERR) != 0 {
                x_ready.push(*obj);
            }
        }
    }
    triple_result(r_ready, w_ready, x_ready)
}

fn triple_result(r: Vec<MbValue>, w: Vec<MbValue>, x: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple_borrowed(vec![
        MbValue::from_ptr(MbObject::new_list(r)),
        MbValue::from_ptr(MbObject::new_list(w)),
        MbValue::from_ptr(MbObject::new_list(x)),
    ]))
}

// ── Shared poll(2)-backed engine: `poll` / `devpoll` / `epoll` ──
//
// Real macOS CPython only has `select.poll` (backed by poll(2)); `devpoll`
// and `epoll` are Solaris/Linux-only. Mamba registers all three against the
// same real poll(2) engine so their FileDescriptorLike/timeout type-wall
// fixtures reach a real TypeError, and register/poll functionally work.

const DEFAULT_EVENTMASK: i64 = (libc::POLLIN | libc::POLLPRI | libc::POLLOUT) as i64;

fn new_instance_shell(class_name: &str, extra: &[(&str, MbValue)]) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert("__class__".to_string(), new_str(class_name));
            for (k, v) in extra {
                map.insert((*k).to_string(), *v);
            }
        }
    }
    MbValue::from_ptr(inst_ptr)
}

fn poll_engine_new_poll(_args: &[MbValue]) -> MbValue {
    new_instance_shell("poll", &[])
}

fn poll_engine_new_devpoll(_args: &[MbValue]) -> MbValue {
    new_instance_shell("devpoll", &[])
}

fn poll_engine_new_epoll(args: &[MbValue]) -> MbValue {
    match epoll_new_checked(args) {
        Ok(v) => v,
        Err(e) => e,
    }
}

fn epoll_new_checked(args: &[MbValue]) -> Result<MbValue, MbValue> {
    if let Some(sizehint) = args.first() {
        if !sizehint.is_none() && sizehint.as_int_pyint().is_none() {
            return Err(raise_type_error("sizehint must be an integer"));
        }
    }
    Ok(new_instance_shell(
        "epoll",
        &[("closed", MbValue::from_bool(false))],
    ))
}

/// `epoll.fromfd(fd)` — classmethod-style unbound call. Reached via the
/// func-as-receiver bridge in `class.rs` (`epoll` bound to `NATIVE_TYPE_NAMES`
/// -> the "epoll" class table), so the raw args are the call args directly.
fn epoll_fromfd(args: &[MbValue]) -> MbValue {
    let Some(fd_arg) = args.first().copied() else {
        return raise_type_error("fromfd() missing required argument: 'fd' (pos 1)");
    };
    let fd = match extract_fd(fd_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    new_instance_shell(
        "epoll",
        &[
            ("fd", MbValue::from_int(fd as i64)),
            ("closed", MbValue::from_bool(false)),
        ],
    )
}

/// Lazily fetch (or create) the instance registration List `_map`: a List
/// of `[fd, eventmask]` pairs.
fn ensure_poll_map(self_v: MbValue) -> MbValue {
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(existing) = fields.read().unwrap().get("_map").copied() {
                    return existing;
                }
                let list = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                super::super::rc::retain_if_ptr(list);
                fields.write().unwrap().insert("_map".to_string(), list);
                return list;
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

fn read_poll_map(self_v: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(m) = fields.read().unwrap().get("_map").copied() {
                    return method_pos(m);
                }
            }
        }
    }
    Vec::new()
}

fn poll_map_contains(self_v: MbValue, fd: i64) -> bool {
    read_poll_map(self_v)
        .iter()
        .any(|pair| method_pos(*pair).first().and_then(|v| v.as_int()) == Some(fd))
}

fn poll_map_upsert(self_v: MbValue, fd: i64, eventmask: i64) {
    let map = ensure_poll_map(self_v);
    let pair = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(fd),
        MbValue::from_int(eventmask),
    ]));
    if let Some(ptr) = map.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut v = lock.write().unwrap();
                for slot in v.iter_mut() {
                    if method_pos(*slot).first().and_then(|x| x.as_int()) == Some(fd) {
                        super::super::rc::retain_if_ptr(pair);
                        let old = *slot;
                        *slot = pair;
                        super::super::rc::release_if_ptr(old);
                        return;
                    }
                }
                super::super::rc::retain_if_ptr(pair);
                v.push(pair);
            }
        }
    }
}

fn poll_map_remove(self_v: MbValue, fd: i64) -> bool {
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(m) = fields.read().unwrap().get("_map").copied() {
                    if let Some(mptr) = m.as_ptr() {
                        if let ObjData::List(ref lock) = (*mptr).data {
                            let mut v = lock.write().unwrap();
                            let before = v.len();
                            v.retain(|pair| {
                                method_pos(*pair).first().and_then(|x| x.as_int()) != Some(fd)
                            });
                            return v.len() != before;
                        }
                    }
                }
            }
        }
    }
    false
}

unsafe extern "C" fn pe_register(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kwargs) = split_method_kwargs(args);
    let Some(fd_arg) = pos.first().copied() else {
        return raise_type_error("register() missing required argument: 'fd' (pos 1)");
    };
    let fd = match extract_fd(fd_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let eventmask = pos
        .get(1)
        .copied()
        .or_else(|| kw_value(kwargs, "eventmask"))
        .and_then(|v| v.as_int_pyint())
        .unwrap_or(DEFAULT_EVENTMASK);
    poll_map_upsert(self_v, fd as i64, eventmask);
    MbValue::none()
}

unsafe extern "C" fn pe_unregister(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    let Some(fd_arg) = pos.first().copied() else {
        return raise_type_error("unregister() missing required argument: 'fd' (pos 1)");
    };
    let fd = match extract_fd(fd_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if !poll_map_remove(self_v, fd as i64) {
        return raise_exc("KeyError", fd.to_string());
    }
    MbValue::none()
}

unsafe extern "C" fn pe_modify(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kwargs) = split_method_kwargs(args);
    let Some(fd_arg) = pos.first().copied() else {
        return raise_type_error("modify() missing required argument: 'fd' (pos 1)");
    };
    let fd = match extract_fd(fd_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let eventmask = pos
        .get(1)
        .copied()
        .or_else(|| kw_value(kwargs, "eventmask"))
        .and_then(|v| v.as_int_pyint())
        .unwrap_or(DEFAULT_EVENTMASK);
    if !poll_map_contains(self_v, fd as i64) {
        return raise_exc("KeyError", fd.to_string());
    }
    poll_map_upsert(self_v, fd as i64, eventmask);
    MbValue::none()
}

unsafe extern "C" fn pe_poll(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kwargs) = split_method_kwargs(args);
    let timeout_arg = pos.first().copied().or_else(|| kw_value(kwargs, "timeout"));
    let timeout_ms = match strict_timeout_ms_direct(timeout_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let entries = read_poll_map(self_v);
    if entries.is_empty() {
        if timeout_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(timeout_ms as u64));
        }
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    let mut pollfds: Vec<libc::pollfd> = entries
        .iter()
        .filter_map(|pair| {
            let parts = method_pos(*pair);
            let fd = parts.first().and_then(|v| v.as_int())?;
            let ev = parts.get(1).and_then(|v| v.as_int()).unwrap_or(0);
            Some(libc::pollfd {
                fd: fd as c_int,
                events: ev as i16,
                revents: 0,
            })
        })
        .collect();
    let rc = unsafe {
        libc::poll(
            pollfds.as_mut_ptr(),
            pollfds.len() as libc::nfds_t,
            timeout_ms,
        )
    };
    if rc < 0 {
        return select_errno();
    }
    let mut ready = Vec::new();
    for p in &pollfds {
        if p.revents != 0 {
            ready.push(MbValue::from_ptr(MbObject::new_tuple_borrowed(vec![
                MbValue::from_int(p.fd as i64),
                MbValue::from_int(p.revents as i64),
            ])));
        }
    }
    MbValue::from_ptr(MbObject::new_list(ready))
}

/// `epoll.__new__(cls, sizehint=-1, flags=0)` called as a bound instance
/// method (`obj.__new__(x)`) — validates `sizehint`, returns a fresh shell.
unsafe extern "C" fn e_new(_self_v: MbValue, args: MbValue) -> MbValue {
    let pos = method_pos(args);
    match epoll_new_checked(&pos) {
        Ok(v) => v,
        Err(e) => e,
    }
}

unsafe extern "C" fn e_close(self_v: MbValue, _args: MbValue) -> MbValue {
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

unsafe extern "C" fn e_fileno(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(fd) = fields.read().unwrap().get("fd").copied() {
                    return fd;
                }
            }
        }
    }
    MbValue::from_int(-1)
}

unsafe extern "C" fn e_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::rc::retain_if_ptr(self_v);
    self_v
}

unsafe extern "C" fn e_exit(self_v: MbValue, args: MbValue) -> MbValue {
    e_close(self_v, args)
}

/// Register the shared `register`/`unregister`/`modify`/`poll` method table
/// for `poll` / `devpoll` / `epoll`, plus epoll's extra
/// `__new__`/`close`/`fileno`/`__enter__`/`__exit__`/`fromfd`.
fn register_poll_engine() {
    let shared: Vec<(&str, usize)> = vec![
        ("register", pe_register as usize),
        ("unregister", pe_unregister as usize),
        ("modify", pe_modify as usize),
        ("poll", pe_poll as usize),
    ];
    let mut shared_map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in &shared {
        shared_map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    super::super::class::mb_class_register("poll", Vec::new(), shared_map.clone());
    super::super::class::mb_class_register("devpoll", Vec::new(), shared_map.clone());

    let mut epoll_map = shared_map;
    let epoll_bound: Vec<(&str, usize)> = vec![
        ("__new__", e_new as usize),
        ("close", e_close as usize),
        ("fileno", e_fileno as usize),
        ("__enter__", e_enter as usize),
        ("__exit__", e_exit as usize),
    ];
    for (name, addr) in &epoll_bound {
        epoll_map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    // `fromfd` is a classmethod (raw (args_ptr, nargs) ABI) so the
    // func-as-receiver bridge in `class.rs` can call it directly.
    epoll_map.insert(
        "fromfd".to_string(),
        MbValue::from_func(d_epoll_fromfd as usize),
    );
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(d_epoll_fromfd as usize as u64);
    });
    super::super::class::mb_class_register("epoll", Vec::new(), epoll_map);
}

// ── kqueue / kevent (macOS only — matches real CPython platform surface) ──

#[cfg(target_os = "macos")]
mod kq {
    use super::*;

    fn kq_field_int(self_v: MbValue, name: &str) -> Option<i64> {
        self_v.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(name).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    }

    fn kq_set_field(self_v: MbValue, name: &str, value: MbValue) {
        if let Some(ptr) = self_v.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.write().unwrap().insert(name.to_string(), value);
                }
            }
        }
    }

    pub(super) fn kqueue_new(_args: &[MbValue]) -> MbValue {
        let fd = unsafe { libc::kqueue() };
        if fd < 0 {
            return select_errno();
        }
        new_instance_shell(
            "kqueue",
            &[
                ("fd", MbValue::from_int(fd as i64)),
                ("closed", MbValue::from_bool(false)),
            ],
        )
    }

    /// `kqueue.fromfd(fd)` — classmethod-style unbound call (func-as-receiver
    /// bridge in `class.rs`), wraps an existing kqueue fd without opening one.
    pub(super) fn kqueue_fromfd(args: &[MbValue]) -> MbValue {
        let Some(fd_arg) = args.first().copied() else {
            return raise_type_error("fromfd() missing required argument: 'fd' (pos 1)");
        };
        let fd = match extract_fd(fd_arg) {
            Ok(v) => v,
            Err(e) => return e,
        };
        new_instance_shell(
            "kqueue",
            &[
                ("fd", MbValue::from_int(fd as i64)),
                ("closed", MbValue::from_bool(false)),
            ],
        )
    }

    unsafe extern "C" fn k_close(self_v: MbValue, _args: MbValue) -> MbValue {
        if let Some(fd) = kq_field_int(self_v, "fd") {
            if kq_field_int(self_v, "closed") != Some(1) {
                unsafe {
                    libc::close(fd as c_int);
                }
            }
        }
        kq_set_field(self_v, "closed", MbValue::from_bool(true));
        MbValue::none()
    }

    unsafe extern "C" fn k_fileno(self_v: MbValue, _args: MbValue) -> MbValue {
        if kq_field_int(self_v, "closed") == Some(1) {
            return raise_exc("ValueError", "I/O operation on closed kqueue object");
        }
        MbValue::from_int(kq_field_int(self_v, "fd").unwrap_or(-1))
    }

    unsafe extern "C" fn k_enter(self_v: MbValue, _args: MbValue) -> MbValue {
        super::super::super::rc::retain_if_ptr(self_v);
        self_v
    }

    unsafe extern "C" fn k_exit(self_v: MbValue, args: MbValue) -> MbValue {
        k_close(self_v, args)
    }

    fn kevent_field(v: MbValue, name: &str, default: i64) -> i64 {
        v.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.read().unwrap().get(name).and_then(|f| f.as_int())
                } else {
                    None
                }
            })
            .unwrap_or(default)
    }

    /// `kqueue.control(changelist, max_events[, timeout])`.
    unsafe extern "C" fn k_control(self_v: MbValue, args: MbValue) -> MbValue {
        let (pos, kwargs) = split_method_kwargs(args);
        let Some(changelist_arg) = pos.first().copied() else {
            return raise_type_error("control() missing required argument: 'changelist' (pos 1)");
        };
        let changelist = match require_iterable(changelist_arg, "changelist") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let max_events = pos
            .get(1)
            .copied()
            .and_then(|v| v.as_int_pyint())
            .unwrap_or(0)
            .max(0) as usize;
        let timeout_arg = pos.get(2).copied().or_else(|| kw_value(kwargs, "timeout"));
        let timeout_secs = match strict_timeout_secs(timeout_arg) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let Some(kq_fd) = kq_field_int(self_v, "fd") else {
            return raise_exc("ValueError", "I/O operation on closed kqueue object");
        };

        let changes: Vec<libc::kevent> = changelist
            .iter()
            .map(|item| libc::kevent {
                ident: kevent_field(*item, "ident", 0) as usize,
                filter: kevent_field(*item, "filter", 0) as i16,
                flags: kevent_field(*item, "flags", 0) as u16,
                fflags: kevent_field(*item, "fflags", 0) as u32,
                data: kevent_field(*item, "data", 0) as isize,
                udata: kevent_field(*item, "udata", 0) as *mut std::ffi::c_void,
            })
            .collect();

        let mut eventlist: Vec<libc::kevent> = vec![
            libc::kevent {
                ident: 0,
                filter: 0,
                flags: 0,
                fflags: 0,
                data: 0,
                udata: std::ptr::null_mut(),
            };
            max_events.max(1)
        ];

        let ts = timeout_secs.map(|secs| libc::timespec {
            tv_sec: secs.trunc() as i64,
            tv_nsec: (secs.fract() * 1_000_000_000.0) as i64,
        });
        let ts_ptr = ts
            .as_ref()
            .map(|t| t as *const libc::timespec)
            .unwrap_or(std::ptr::null());

        let rc = unsafe {
            libc::kevent(
                kq_fd as c_int,
                changes.as_ptr(),
                changes.len() as c_int,
                eventlist.as_mut_ptr(),
                max_events as c_int,
                ts_ptr,
            )
        };
        if rc < 0 {
            return select_errno();
        }
        let mut out = Vec::with_capacity(rc as usize);
        for ev in eventlist.into_iter().take(rc as usize) {
            out.push(new_kevent_instance(
                ev.ident as i64,
                ev.filter as i64,
                ev.flags as i64,
                ev.fflags as i64,
                ev.data as i64,
                ev.udata as i64,
            ));
        }
        MbValue::from_ptr(MbObject::new_list(out))
    }

    pub(super) fn register_kqueue_kevent() {
        let methods: Vec<(&str, usize)> = vec![
            ("control", k_control as usize),
            ("close", k_close as usize),
            ("fileno", k_fileno as usize),
            ("__enter__", k_enter as usize),
            ("__exit__", k_exit as usize),
        ];
        let mut map: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in &methods {
            map.insert((*name).to_string(), MbValue::from_func(*addr));
            super::super::super::module::register_variadic_func(*addr as u64);
        }
        map.insert(
            "fromfd".to_string(),
            MbValue::from_func(super::d_kqueue_fromfd as usize),
        );
        super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut()
                .insert(super::d_kqueue_fromfd as usize as u64);
        });
        super::super::super::class::mb_class_register("kqueue", Vec::new(), map);
        super::super::super::class::mb_class_register("kevent", Vec::new(), HashMap::new());
        super::super::super::module::register_native_type_name(
            super::d_kqueue_new as usize as u64,
            "kqueue".to_string(),
        );
        super::super::super::module::register_native_type_name(
            super::d_kevent_new as usize as u64,
            "kevent".to_string(),
        );
    }
}

#[cfg(target_os = "macos")]
fn new_kevent_instance(
    ident: i64,
    filter: i64,
    flags: i64,
    fflags: i64,
    data: i64,
    udata: i64,
) -> MbValue {
    new_instance_shell(
        "kevent",
        &[
            ("ident", MbValue::from_int(ident)),
            ("filter", MbValue::from_int(filter)),
            ("flags", MbValue::from_int(flags)),
            ("fflags", MbValue::from_int(fflags)),
            ("data", MbValue::from_int(data)),
            ("udata", MbValue::from_int(udata)),
        ],
    )
}

/// `kevent(ident, filter=KQ_FILTER_READ, flags=KQ_EV_ADD, fflags=0, data=0,
/// udata=0)`. `ident` is FileDescriptorLike (typeshed contract); the rest
/// are loosely-coerced ints (not exercised by a type-wall fixture).
#[cfg(target_os = "macos")]
fn kevent_new(args: &[MbValue]) -> MbValue {
    let Some(ident_arg) = args.first().copied() else {
        return raise_type_error("kevent() missing required argument: 'ident' (pos 1)");
    };
    let ident = match extract_fd(ident_arg) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let filter = args
        .get(1)
        .and_then(|v| v.as_int_pyint())
        .unwrap_or(libc::EVFILT_READ as i64);
    let flags = args
        .get(2)
        .and_then(|v| v.as_int_pyint())
        .unwrap_or(libc::EV_ADD as i64);
    let fflags = args.get(3).and_then(|v| v.as_int_pyint()).unwrap_or(0);
    let data = args.get(4).and_then(|v| v.as_int_pyint()).unwrap_or(0);
    let udata = args.get(5).and_then(|v| v.as_int_pyint()).unwrap_or(0);
    new_kevent_instance(ident as i64, filter, flags, fflags, data, udata)
}

#[cfg(target_os = "macos")]
fn kqueue_new(args: &[MbValue]) -> MbValue {
    kq::kqueue_new(args)
}

#[cfg(target_os = "macos")]
fn kqueue_fromfd(args: &[MbValue]) -> MbValue {
    kq::kqueue_fromfd(args)
}

// ── Module registration ──

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("select", d_select as usize),
        ("poll", d_poll_new as usize),
        ("devpoll", d_devpoll_new as usize),
        ("epoll", d_epoll_new as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::super::module::register_native_type_name(d_poll_new as usize as u64, "poll".to_string());
    super::super::module::register_native_type_name(
        d_devpoll_new as usize as u64,
        "devpoll".to_string(),
    );
    super::super::module::register_native_type_name(
        d_epoll_new as usize as u64,
        "epoll".to_string(),
    );

    #[cfg(target_os = "macos")]
    {
        attrs.insert(
            "kqueue".to_string(),
            MbValue::from_func(d_kqueue_new as usize),
        );
        attrs.insert(
            "kevent".to_string(),
            MbValue::from_func(d_kevent_new as usize),
        );
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            let mut set = s.borrow_mut();
            set.insert(d_kqueue_new as usize as u64);
            set.insert(d_kevent_new as usize as u64);
        });
    }

    // ── poll(2) event-mask constants (real libc values; match CPython). ──
    attrs.insert("POLLIN".to_string(), MbValue::from_int(libc::POLLIN as i64));
    attrs.insert(
        "POLLOUT".to_string(),
        MbValue::from_int(libc::POLLOUT as i64),
    );
    attrs.insert(
        "POLLPRI".to_string(),
        MbValue::from_int(libc::POLLPRI as i64),
    );
    attrs.insert(
        "POLLERR".to_string(),
        MbValue::from_int(libc::POLLERR as i64),
    );
    attrs.insert(
        "POLLHUP".to_string(),
        MbValue::from_int(libc::POLLHUP as i64),
    );
    attrs.insert(
        "POLLNVAL".to_string(),
        MbValue::from_int(libc::POLLNVAL as i64),
    );
    attrs.insert(
        "POLLRDNORM".to_string(),
        MbValue::from_int(libc::POLLRDNORM as i64),
    );
    attrs.insert(
        "POLLRDBAND".to_string(),
        MbValue::from_int(libc::POLLRDBAND as i64),
    );
    attrs.insert(
        "POLLWRNORM".to_string(),
        MbValue::from_int(libc::POLLWRNORM as i64),
    );
    attrs.insert(
        "POLLWRBAND".to_string(),
        MbValue::from_int(libc::POLLWRBAND as i64),
    );
    #[cfg(target_os = "linux")]
    {
        attrs.insert(
            "POLLMSG".to_string(),
            MbValue::from_int(libc::POLLMSG as i64),
        );
    }

    // ── kqueue/kevent constants (real macOS libc values). ──
    #[cfg(target_os = "macos")]
    {
        attrs.insert(
            "KQ_FILTER_READ".to_string(),
            MbValue::from_int(libc::EVFILT_READ as i64),
        );
        attrs.insert(
            "KQ_FILTER_WRITE".to_string(),
            MbValue::from_int(libc::EVFILT_WRITE as i64),
        );
        attrs.insert(
            "KQ_EV_ADD".to_string(),
            MbValue::from_int(libc::EV_ADD as i64),
        );
        attrs.insert(
            "KQ_EV_DELETE".to_string(),
            MbValue::from_int(libc::EV_DELETE as i64),
        );
        attrs.insert(
            "KQ_EV_ENABLE".to_string(),
            MbValue::from_int(libc::EV_ENABLE as i64),
        );
        attrs.insert(
            "KQ_EV_DISABLE".to_string(),
            MbValue::from_int(libc::EV_DISABLE as i64),
        );
        attrs.insert(
            "KQ_EV_ONESHOT".to_string(),
            MbValue::from_int(libc::EV_ONESHOT as i64),
        );
        attrs.insert(
            "KQ_EV_CLEAR".to_string(),
            MbValue::from_int(libc::EV_CLEAR as i64),
        );
        attrs.insert(
            "KQ_EV_EOF".to_string(),
            MbValue::from_int(libc::EV_EOF as i64),
        );
        attrs.insert(
            "KQ_EV_ERROR".to_string(),
            MbValue::from_int(libc::EV_ERROR as i64),
        );
    }

    // `select.error is OSError` — a literal alias, matching CPython (and the
    // socket.error / os.error convention already used elsewhere in mamba).
    super::super::class::mb_class_register(
        "OSError",
        vec!["Exception".to_string()],
        HashMap::new(),
    );
    attrs.insert("error".to_string(), new_str("OSError"));

    super::register_module("select", attrs);

    register_poll_engine();
    #[cfg(target_os = "macos")]
    kq::register_kqueue_kevent();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn select_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow()
                .get("select")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_core_surface() {
        register();
        for name in [
            "select", "poll", "devpoll", "epoll", "POLLIN", "POLLOUT", "POLLPRI", "POLLERR",
            "POLLHUP", "POLLNVAL", "error",
        ] {
            assert!(select_attr(name).is_some(), "select module missing: {name}");
        }
        #[cfg(target_os = "macos")]
        for name in ["kqueue", "kevent", "KQ_FILTER_READ", "KQ_EV_ADD"] {
            assert!(select_attr(name).is_some(), "select module missing: {name}");
        }
    }

    #[test]
    fn test_poll_constants_match_cpython_values() {
        register();
        assert_eq!(select_attr("POLLIN").and_then(|v| v.as_int()), Some(1));
        assert_eq!(select_attr("POLLPRI").and_then(|v| v.as_int()), Some(2));
        assert_eq!(select_attr("POLLOUT").and_then(|v| v.as_int()), Some(4));
        assert_eq!(select_attr("POLLERR").and_then(|v| v.as_int()), Some(8));
        assert_eq!(select_attr("POLLHUP").and_then(|v| v.as_int()), Some(16));
    }

    #[test]
    fn test_select_reports_pipe_readable() {
        register();
        let mut fds = [0; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        unsafe {
            let byte = [1u8];
            assert_eq!(
                libc::write(fds[1], byte.as_ptr() as *const libc::c_void, 1),
                1
            );
        }
        let rlist = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(fds[0] as i64)]));
        let wlist = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let xlist = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let result = mb_select(&[rlist, wlist, xlist, MbValue::from_float(1.0)]);
        let parts = method_pos(result);
        let ready_r = method_pos(parts[0]);
        assert_eq!(ready_r.len(), 1);
        assert_eq!(ready_r[0].as_int(), Some(fds[0] as i64));
        unsafe {
            libc::close(fds[0]);
            libc::close(fds[1]);
        }
    }

    #[test]
    fn test_select_empty_rlist_times_out() {
        register();
        let rlist = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let wlist = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let xlist = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let start = std::time::Instant::now();
        let result = mb_select(&[rlist, wlist, xlist, MbValue::from_float(0.05)]);
        assert!(start.elapsed() >= std::time::Duration::from_millis(40));
        let parts = method_pos(result);
        assert_eq!(method_pos(parts[0]).len(), 0);
        assert_eq!(method_pos(parts[1]).len(), 0);
        assert_eq!(method_pos(parts[2]).len(), 0);
    }

    #[test]
    fn test_poll_register_and_poll_pipe() {
        register();
        let mut fds = [0; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        let p = poll_engine_new_poll(&[]);
        unsafe {
            let args =
                MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(fds[0] as i64)]));
            pe_register(p, args);
            let byte = [1u8];
            assert_eq!(
                libc::write(fds[1], byte.as_ptr() as *const libc::c_void, 1),
                1
            );
            let ready = pe_poll(
                p,
                MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1000)])),
            );
            let entries = method_pos(ready);
            assert_eq!(entries.len(), 1);
            let pair = method_pos(entries[0]);
            assert_eq!(pair[0].as_int(), Some(fds[0] as i64));
            libc::close(fds[0]);
            libc::close(fds[1]);
        }
    }
}
