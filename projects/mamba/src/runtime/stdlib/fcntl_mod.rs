use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::os::raw::{c_int, c_ulong};

/// Minimal Unix `fcntl` module.
///
/// Mamba file objects expose a runtime file-handle id, not a host OS file
/// descriptor. For those virtual handles, lock/control operations are
/// side-effect-free success paths. Real integer fds still dispatch to libc.

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

disp_variadic!(dispatch_fcntl, mb_fcntl);
disp_variadic!(dispatch_ioctl, mb_ioctl);
disp_variadic!(dispatch_flock, mb_flock);
disp_variadic!(dispatch_lockf, mb_lockf);

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

fn raise_overflow_error(msg: impl Into<String>) -> MbValue {
    raise_exc("OverflowError", msg)
}

fn raise_os_error(context: &str) -> MbValue {
    let err = std::io::Error::last_os_error();
    raise_exc("OSError", format!("{context}: {err}"))
}

fn extract_c_int(value: MbValue, what: &str) -> Result<c_int, MbValue> {
    let Some(raw) = value.as_int_pyint() else {
        return Err(raise_type_error(format!("{what} must be an integer")));
    };
    if raw < c_int::MIN as i64 || raw > c_int::MAX as i64 {
        return Err(raise_overflow_error(format!(
            "{what} is greater than maximum file descriptor"
        )));
    }
    Ok(raw as c_int)
}

fn extract_fd(value: MbValue) -> Result<c_int, MbValue> {
    let fd = if value.as_int_pyint().is_some() {
        extract_c_int(value, "file descriptor")?
    } else {
        let method = MbValue::from_ptr(MbObject::new_str("fileno".to_string()));
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

fn is_virtual_file_fd(fd: c_int) -> bool {
    fd >= 0 && super::super::file_io::is_file_handle(fd as u64)
}

fn bytes_like(value: MbValue) -> Option<Vec<u8>> {
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(data) => Some(data.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    }
}

fn register_func(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
    attrs.insert(name.to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

pub fn register() {
    let mut attrs = HashMap::new();
    register_func(&mut attrs, "fcntl", dispatch_fcntl as *const () as usize);
    register_func(&mut attrs, "ioctl", dispatch_ioctl as *const () as usize);
    register_func(&mut attrs, "flock", dispatch_flock as *const () as usize);
    register_func(&mut attrs, "lockf", dispatch_lockf as *const () as usize);

    attrs.insert("F_DUPFD".into(), MbValue::from_int(libc::F_DUPFD as i64));
    attrs.insert("F_GETFD".into(), MbValue::from_int(libc::F_GETFD as i64));
    attrs.insert("F_SETFD".into(), MbValue::from_int(libc::F_SETFD as i64));
    attrs.insert("F_GETFL".into(), MbValue::from_int(libc::F_GETFL as i64));
    attrs.insert("F_SETFL".into(), MbValue::from_int(libc::F_SETFL as i64));
    attrs.insert("F_GETLK".into(), MbValue::from_int(libc::F_GETLK as i64));
    attrs.insert("F_SETLK".into(), MbValue::from_int(libc::F_SETLK as i64));
    attrs.insert("F_SETLKW".into(), MbValue::from_int(libc::F_SETLKW as i64));
    attrs.insert("F_RDLCK".into(), MbValue::from_int(libc::F_RDLCK as i64));
    attrs.insert("F_WRLCK".into(), MbValue::from_int(libc::F_WRLCK as i64));
    attrs.insert("F_UNLCK".into(), MbValue::from_int(libc::F_UNLCK as i64));

    attrs.insert("LOCK_SH".into(), MbValue::from_int(libc::LOCK_SH as i64));
    attrs.insert("LOCK_EX".into(), MbValue::from_int(libc::LOCK_EX as i64));
    attrs.insert("LOCK_NB".into(), MbValue::from_int(libc::LOCK_NB as i64));
    attrs.insert("LOCK_UN".into(), MbValue::from_int(libc::LOCK_UN as i64));

    #[cfg(target_os = "macos")]
    {
        attrs.insert(
            "F_GETPATH".into(),
            MbValue::from_int(libc::F_GETPATH as i64),
        );
    }
    #[cfg(target_os = "linux")]
    {
        attrs.insert(
            "F_GETPIPE_SZ".into(),
            MbValue::from_int(libc::F_GETPIPE_SZ as i64),
        );
        attrs.insert(
            "F_SETPIPE_SZ".into(),
            MbValue::from_int(libc::F_SETPIPE_SZ as i64),
        );
    }

    super::register_module("fcntl", attrs);
}

fn mb_fcntl(args: &[MbValue]) -> MbValue {
    if args.len() < 2 {
        return raise_type_error("fcntl() requires at least 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let cmd = match extract_c_int(args[1], "command") {
        Ok(cmd) => cmd,
        Err(err) => return err,
    };

    if is_virtual_file_fd(fd) {
        if let Some(arg) = args.get(2).copied() {
            if let Some(data) = bytes_like(arg) {
                return MbValue::from_ptr(MbObject::new_bytes(data));
            }
        }
        return MbValue::from_int(0);
    }

    if let Some(arg) = args.get(2).copied() {
        if let Some(raw) = arg.as_int_pyint() {
            if raw < c_int::MIN as i64 || raw > c_int::MAX as i64 {
                return raise_overflow_error("fcntl integer argument out of range");
            }
            let rv = unsafe { libc::fcntl(fd, cmd, raw as c_int) };
            if rv < 0 {
                return raise_os_error("fcntl");
            }
            return MbValue::from_int(rv as i64);
        }
        if let Some(mut data) = bytes_like(arg) {
            let rv = unsafe { libc::fcntl(fd, cmd, data.as_mut_ptr()) };
            if rv < 0 {
                return raise_os_error("fcntl");
            }
            return MbValue::from_ptr(MbObject::new_bytes(data));
        }
        return raise_type_error("fcntl() argument must be an int or bytes-like object");
    }

    let rv = unsafe { libc::fcntl(fd, cmd, 0) };
    if rv < 0 {
        return raise_os_error("fcntl");
    }
    MbValue::from_int(rv as i64)
}

fn mb_ioctl(args: &[MbValue]) -> MbValue {
    if args.len() < 2 {
        return raise_type_error("ioctl() requires at least 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let request = match args[1].as_int_pyint() {
        Some(raw) if raw >= 0 => raw as c_ulong,
        Some(_) => return raise_value_error("ioctl request cannot be negative"),
        None => return raise_type_error("ioctl request must be an integer"),
    };

    if is_virtual_file_fd(fd) {
        if let Some(arg) = args.get(2).copied() {
            if let Some(data) = bytes_like(arg) {
                return MbValue::from_ptr(MbObject::new_bytes(data));
            }
        }
        return MbValue::from_int(0);
    }

    if let Some(arg) = args.get(2).copied() {
        if let Some(raw) = arg.as_int_pyint() {
            let rv = unsafe { libc::ioctl(fd, request, raw as c_int) };
            if rv < 0 {
                return raise_os_error("ioctl");
            }
            return MbValue::from_int(rv as i64);
        }
        if let Some(mut data) = bytes_like(arg) {
            let rv = unsafe { libc::ioctl(fd, request, data.as_mut_ptr()) };
            if rv < 0 {
                return raise_os_error("ioctl");
            }
            return MbValue::from_ptr(MbObject::new_bytes(data));
        }
        return raise_type_error("ioctl() argument must be an int or bytes-like object");
    }

    let rv = unsafe { libc::ioctl(fd, request, 0) };
    if rv < 0 {
        return raise_os_error("ioctl");
    }
    MbValue::from_int(rv as i64)
}

fn mb_flock(args: &[MbValue]) -> MbValue {
    if args.len() < 2 {
        return raise_type_error("flock() requires 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let op = match extract_c_int(args[1], "operation") {
        Ok(op) => op,
        Err(err) => return err,
    };
    if is_virtual_file_fd(fd) {
        return MbValue::none();
    }
    if unsafe { libc::flock(fd, op) } < 0 {
        return raise_os_error("flock");
    }
    MbValue::none()
}

fn mb_lockf(args: &[MbValue]) -> MbValue {
    if args.len() < 2 {
        return raise_type_error("lockf() requires at least 2 arguments");
    }
    let fd = match extract_fd(args[0]) {
        Ok(fd) => fd,
        Err(err) => return err,
    };
    let op = match extract_c_int(args[1], "operation") {
        Ok(op) => op,
        Err(err) => return err,
    };
    if is_virtual_file_fd(fd) {
        return MbValue::none();
    }
    if unsafe { libc::flock(fd, op) } < 0 {
        return raise_os_error("lockf");
    }
    MbValue::none()
}
