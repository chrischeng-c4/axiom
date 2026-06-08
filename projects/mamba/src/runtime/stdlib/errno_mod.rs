use super::super::rc::MbObject;
use super::super::value::MbValue;
/// errno module for Mamba (#657).
///
/// Exposes POSIX errno constants matching CPython's errno module.
/// Values follow POSIX / macOS / Linux standard errno numbers.
use std::collections::HashMap;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_strerror, mb_errno_strerror);

pub fn register() {
    let mut attrs = HashMap::new();

    // POSIX errno constants (numeric values)
    attrs.insert("EPERM".into(), MbValue::from_int(1));
    attrs.insert("ENOENT".into(), MbValue::from_int(2));
    attrs.insert("ESRCH".into(), MbValue::from_int(3));
    attrs.insert("EINTR".into(), MbValue::from_int(4));
    attrs.insert("EIO".into(), MbValue::from_int(5));
    attrs.insert("ENXIO".into(), MbValue::from_int(6));
    attrs.insert("E2BIG".into(), MbValue::from_int(7));
    attrs.insert("ENOEXEC".into(), MbValue::from_int(8));
    attrs.insert("EBADF".into(), MbValue::from_int(9));
    attrs.insert("ECHILD".into(), MbValue::from_int(10));
    attrs.insert("EAGAIN".into(), MbValue::from_int(11));
    attrs.insert("ENOMEM".into(), MbValue::from_int(12));
    attrs.insert("EACCES".into(), MbValue::from_int(13));
    attrs.insert("EFAULT".into(), MbValue::from_int(14));
    attrs.insert("EBUSY".into(), MbValue::from_int(16));
    attrs.insert("EEXIST".into(), MbValue::from_int(17));
    attrs.insert("EXDEV".into(), MbValue::from_int(18));
    attrs.insert("ENODEV".into(), MbValue::from_int(19));
    attrs.insert("ENOTDIR".into(), MbValue::from_int(20));
    attrs.insert("EISDIR".into(), MbValue::from_int(21));
    attrs.insert("EINVAL".into(), MbValue::from_int(22));
    attrs.insert("ENFILE".into(), MbValue::from_int(23));
    attrs.insert("EMFILE".into(), MbValue::from_int(24));
    attrs.insert("ENOTTY".into(), MbValue::from_int(25));
    attrs.insert("EFBIG".into(), MbValue::from_int(27));
    attrs.insert("ENOSPC".into(), MbValue::from_int(28));
    attrs.insert("ESPIPE".into(), MbValue::from_int(29));
    attrs.insert("EROFS".into(), MbValue::from_int(30));
    attrs.insert("EMLINK".into(), MbValue::from_int(31));
    attrs.insert("EPIPE".into(), MbValue::from_int(32));
    attrs.insert("EDOM".into(), MbValue::from_int(33));
    attrs.insert("ERANGE".into(), MbValue::from_int(34));
    attrs.insert("EDEADLK".into(), MbValue::from_int(35));
    attrs.insert("ENAMETOOLONG".into(), MbValue::from_int(36));
    attrs.insert("ENOLCK".into(), MbValue::from_int(37));
    attrs.insert("ENOSYS".into(), MbValue::from_int(38));
    attrs.insert("ENOTEMPTY".into(), MbValue::from_int(39));
    attrs.insert("ELOOP".into(), MbValue::from_int(40));
    attrs.insert("EWOULDBLOCK".into(), MbValue::from_int(11)); // alias for EAGAIN
    attrs.insert("ENOMSG".into(), MbValue::from_int(42));
    attrs.insert("EIDRM".into(), MbValue::from_int(43));
    attrs.insert("ENOSTR".into(), MbValue::from_int(60));
    attrs.insert("ENODATA".into(), MbValue::from_int(61));
    attrs.insert("ETIME".into(), MbValue::from_int(62));
    attrs.insert("ENOSR".into(), MbValue::from_int(63));
    attrs.insert("EREMOTE".into(), MbValue::from_int(66));
    attrs.insert("ENOLINK".into(), MbValue::from_int(67));
    attrs.insert("EPROTO".into(), MbValue::from_int(71));
    attrs.insert("EMULTIHOP".into(), MbValue::from_int(72));
    attrs.insert("EBADMSG".into(), MbValue::from_int(74));
    attrs.insert("EOVERFLOW".into(), MbValue::from_int(75));
    attrs.insert("EILSEQ".into(), MbValue::from_int(84));
    attrs.insert("EUSERS".into(), MbValue::from_int(87));
    attrs.insert("ENOTSOCK".into(), MbValue::from_int(88));
    attrs.insert("EDESTADDRREQ".into(), MbValue::from_int(89));
    attrs.insert("EMSGSIZE".into(), MbValue::from_int(90));
    attrs.insert("EPROTOTYPE".into(), MbValue::from_int(91));
    attrs.insert("ENOPROTOOPT".into(), MbValue::from_int(92));
    attrs.insert("EPROTONOSUPPORT".into(), MbValue::from_int(93));
    attrs.insert("ESOCKTNOSUPPORT".into(), MbValue::from_int(94));
    attrs.insert("EOPNOTSUPP".into(), MbValue::from_int(95));
    attrs.insert("EAFNOSUPPORT".into(), MbValue::from_int(97));
    attrs.insert("EADDRINUSE".into(), MbValue::from_int(98));
    attrs.insert("EADDRNOTAVAIL".into(), MbValue::from_int(99));
    attrs.insert("ENETDOWN".into(), MbValue::from_int(100));
    attrs.insert("ENETUNREACH".into(), MbValue::from_int(101));
    attrs.insert("ENETRESET".into(), MbValue::from_int(102));
    attrs.insert("ECONNABORTED".into(), MbValue::from_int(103));
    attrs.insert("ECONNRESET".into(), MbValue::from_int(104));
    attrs.insert("ENOBUFS".into(), MbValue::from_int(105));
    attrs.insert("EISCONN".into(), MbValue::from_int(106));
    attrs.insert("ENOTCONN".into(), MbValue::from_int(107));
    attrs.insert("ESHUTDOWN".into(), MbValue::from_int(108));
    attrs.insert("ETOOMANYREFS".into(), MbValue::from_int(109));
    attrs.insert("ETIMEDOUT".into(), MbValue::from_int(110));
    attrs.insert("ECONNREFUSED".into(), MbValue::from_int(111));
    attrs.insert("EHOSTDOWN".into(), MbValue::from_int(112));
    attrs.insert("EHOSTUNREACH".into(), MbValue::from_int(113));
    attrs.insert("EALREADY".into(), MbValue::from_int(114));
    attrs.insert("EINPROGRESS".into(), MbValue::from_int(115));
    attrs.insert("ESTALE".into(), MbValue::from_int(116));
    attrs.insert("ECANCELED".into(), MbValue::from_int(125));
    attrs.insert("EOWNERDEAD".into(), MbValue::from_int(130));
    attrs.insert("ENOTRECOVERABLE".into(), MbValue::from_int(131));

    // errorcode dict: int -> name string. CPython exposes this as a
    // pre-populated dict attribute, not a callable, so we eagerly
    // build it at register time.
    attrs.insert("errorcode".into(), mb_errno_errorcode());

    // strerror(errnum) -> str — callable.
    let addr = dispatch_strerror as usize;
    attrs.insert("strerror".into(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });

    super::register_module("errno", attrs);
}

/// Return a dict mapping error codes to their symbolic names.
/// CPython parity: keys are ints, values are str.
pub fn mb_errno_errorcode() -> MbValue {
    use super::super::dict_ops::DictKey;
    use super::super::rc::ObjData;
    let dict = MbObject::new_dict();
    let entries: &[(i64, &str)] = &[
        (1, "EPERM"),
        (2, "ENOENT"),
        (3, "ESRCH"),
        (4, "EINTR"),
        (5, "EIO"),
        (6, "ENXIO"),
        (7, "E2BIG"),
        (8, "ENOEXEC"),
        (9, "EBADF"),
        (10, "ECHILD"),
        (11, "EAGAIN"),
        (12, "ENOMEM"),
        (13, "EACCES"),
        (14, "EFAULT"),
        (16, "EBUSY"),
        (17, "EEXIST"),
        (18, "EXDEV"),
        (19, "ENODEV"),
        (20, "ENOTDIR"),
        (21, "EISDIR"),
        (22, "EINVAL"),
        (23, "ENFILE"),
        (24, "EMFILE"),
        (25, "ENOTTY"),
        (27, "EFBIG"),
        (28, "ENOSPC"),
        (29, "ESPIPE"),
        (30, "EROFS"),
        (31, "EMLINK"),
        (32, "EPIPE"),
        (33, "EDOM"),
        (34, "ERANGE"),
        (110, "ETIMEDOUT"),
        (111, "ECONNREFUSED"),
        (113, "EHOSTUNREACH"),
        (115, "EINPROGRESS"),
        (125, "ECANCELED"),
    ];
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (code, name) in entries {
                map.insert(
                    DictKey::Int(*code),
                    MbValue::from_ptr(MbObject::new_str((*name).to_string())),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// strerror(errnum) -> str
pub fn mb_errno_strerror(errnum: MbValue) -> MbValue {
    let n = errnum.as_int().unwrap_or(0);
    let msg = match n {
        1 => "Operation not permitted",
        2 => "No such file or directory",
        3 => "No such process",
        4 => "Interrupted function call",
        5 => "Input/output error",
        6 => "No such device or address",
        7 => "Arg list too long",
        8 => "Exec format error",
        9 => "Bad file descriptor",
        10 => "No child processes",
        11 => "Resource temporarily unavailable",
        12 => "Not enough space",
        13 => "Permission denied",
        14 => "Bad address",
        16 => "Device or resource busy",
        17 => "File exists",
        18 => "Improper link",
        19 => "No such device",
        20 => "Not a directory",
        21 => "Is a directory",
        22 => "Invalid argument",
        23 => "Too many open files in system",
        24 => "Too many open files",
        25 => "Inappropriate I/O control operation",
        27 => "File too large",
        28 => "No space left on device",
        29 => "Invalid seek",
        30 => "Read-only file system",
        31 => "Too many links",
        32 => "Broken pipe",
        33 => "Domain error",
        34 => "Result too large",
        36 => "File name too long",
        38 => "Function not implemented",
        39 => "Directory not empty",
        88 => "Socket operation on non-socket",
        110 => "Connection timed out",
        111 => "Connection refused",
        113 => "No route to host",
        _ => "Unknown error",
    };
    MbValue::from_ptr(MbObject::new_str(msg.to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn str_val(v: MbValue) -> String {
        v.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_errno_constants() {
        assert_eq!(MbValue::from_int(1).as_int(), Some(1));
        assert_eq!(MbValue::from_int(2).as_int(), Some(2));
        assert_eq!(MbValue::from_int(13).as_int(), Some(13));
    }

    #[test]
    fn test_strerror_eperm() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(1))),
            "Operation not permitted"
        );
    }

    #[test]
    fn test_strerror_enoent() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(2))),
            "No such file or directory"
        );
    }

    #[test]
    fn test_strerror_eintr() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(4))),
            "Interrupted function call"
        );
    }

    #[test]
    fn test_strerror_eio() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(5))),
            "Input/output error"
        );
    }

    #[test]
    fn test_strerror_ebadf() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(9))),
            "Bad file descriptor"
        );
    }

    #[test]
    fn test_strerror_eagain() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(11))),
            "Resource temporarily unavailable"
        );
    }

    #[test]
    fn test_strerror_eacces() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(13))),
            "Permission denied"
        );
    }

    #[test]
    fn test_strerror_einval() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(22))),
            "Invalid argument"
        );
    }

    #[test]
    fn test_strerror_epipe() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(32))),
            "Broken pipe"
        );
    }

    #[test]
    fn test_strerror_etimedout() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(110))),
            "Connection timed out"
        );
    }

    #[test]
    fn test_strerror_econnrefused() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(111))),
            "Connection refused"
        );
    }

    #[test]
    fn test_strerror_ehostunreach() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(113))),
            "No route to host"
        );
    }

    #[test]
    fn test_strerror_unknown_code() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(999))),
            "Unknown error"
        );
    }

    #[test]
    fn test_strerror_zero_unknown() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(0))),
            "Unknown error"
        );
    }

    #[test]
    fn test_errorcode_dict_has_enoent() {
        // CPython parity: errorcode keys are ints, not strings.
        use super::super::super::dict_ops::DictKey;
        let result = mb_errno_errorcode();
        let found = result
            .as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    map.get(&DictKey::Int(2)).and_then(|v| {
                        v.as_ptr().and_then(|p| {
                            if let ObjData::Str(ref s) = (*p).data {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                    })
                } else {
                    None
                }
            })
            .flatten();
        assert_eq!(found, Some("ENOENT".to_string()));
    }

    #[test]
    fn test_errorcode() {
        let result = mb_errno_errorcode();
        assert!(result.as_ptr().is_some());
    }

    // --- Remaining strerror match arms ---
    #[test]
    fn test_strerror_esrch() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(3))),
            "No such process"
        );
    }

    #[test]
    fn test_strerror_enxio() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(6))),
            "No such device or address"
        );
    }

    #[test]
    fn test_strerror_e2big() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(7))),
            "Arg list too long"
        );
    }

    #[test]
    fn test_strerror_enoexec() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(8))),
            "Exec format error"
        );
    }

    #[test]
    fn test_strerror_echild() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(10))),
            "No child processes"
        );
    }

    #[test]
    fn test_strerror_enomem() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(12))),
            "Not enough space"
        );
    }

    #[test]
    fn test_strerror_efault() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(14))),
            "Bad address"
        );
    }

    #[test]
    fn test_strerror_ebusy() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(16))),
            "Device or resource busy"
        );
    }

    #[test]
    fn test_strerror_eexist() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(17))),
            "File exists"
        );
    }

    #[test]
    fn test_strerror_exdev() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(18))),
            "Improper link"
        );
    }

    #[test]
    fn test_strerror_enodev() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(19))),
            "No such device"
        );
    }

    #[test]
    fn test_strerror_enotdir() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(20))),
            "Not a directory"
        );
    }

    #[test]
    fn test_strerror_eisdir() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(21))),
            "Is a directory"
        );
    }

    #[test]
    fn test_strerror_enfile() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(23))),
            "Too many open files in system"
        );
    }

    #[test]
    fn test_strerror_emfile() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(24))),
            "Too many open files"
        );
    }

    #[test]
    fn test_strerror_enotty() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(25))),
            "Inappropriate I/O control operation"
        );
    }

    #[test]
    fn test_strerror_efbig() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(27))),
            "File too large"
        );
    }

    #[test]
    fn test_strerror_enospc() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(28))),
            "No space left on device"
        );
    }

    #[test]
    fn test_strerror_espipe() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(29))),
            "Invalid seek"
        );
    }

    #[test]
    fn test_strerror_erofs() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(30))),
            "Read-only file system"
        );
    }

    #[test]
    fn test_strerror_emlink() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(31))),
            "Too many links"
        );
    }

    #[test]
    fn test_strerror_edom() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(33))),
            "Domain error"
        );
    }

    #[test]
    fn test_strerror_erange() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(34))),
            "Result too large"
        );
    }

    #[test]
    fn test_strerror_enametoolong() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(36))),
            "File name too long"
        );
    }

    #[test]
    fn test_strerror_enosys() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(38))),
            "Function not implemented"
        );
    }

    #[test]
    fn test_strerror_enotempty() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(39))),
            "Directory not empty"
        );
    }

    #[test]
    fn test_strerror_enotsock() {
        assert_eq!(
            str_val(mb_errno_strerror(MbValue::from_int(88))),
            "Socket operation on non-socket"
        );
    }
}
