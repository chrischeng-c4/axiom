//! Minimal Unix `syslog` module surface for strict type-wall accounting.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

macro_rules! dispatch_varargs {
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

macro_rules! dispatch_nullary {
    ($name:ident, $func:path) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $func()
        }
    };
}

dispatch_varargs!(dispatch_log_mask, mb_log_mask);
dispatch_varargs!(dispatch_log_upto, mb_log_upto);
dispatch_nullary!(dispatch_closelog, mb_closelog);
dispatch_varargs!(dispatch_openlog, mb_openlog);
dispatch_varargs!(dispatch_setlogmask, mb_setlogmask);
dispatch_varargs!(dispatch_syslog, mb_syslog);

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg.into()));
    MbValue::none()
}

fn as_str(value: MbValue) -> Option<String> {
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
}

fn expect_int(value: MbValue, name: &str) -> Result<i64, MbValue> {
    value
        .as_int_pyint()
        .ok_or_else(|| raise_type_error(format!("{name} must be int")))
}

fn expect_str(value: MbValue, name: &str) -> Result<String, MbValue> {
    as_str(value).ok_or_else(|| raise_type_error(format!("{name} must be str")))
}

fn register_func(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
    attrs.insert(name.to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

pub fn register() {
    let mut attrs = HashMap::new();
    register_func(&mut attrs, "LOG_MASK", dispatch_log_mask as *const () as usize);
    register_func(&mut attrs, "LOG_UPTO", dispatch_log_upto as *const () as usize);
    register_func(&mut attrs, "closelog", dispatch_closelog as *const () as usize);
    register_func(&mut attrs, "openlog", dispatch_openlog as *const () as usize);
    register_func(
        &mut attrs,
        "setlogmask",
        dispatch_setlogmask as *const () as usize,
    );
    register_func(&mut attrs, "syslog", dispatch_syslog as *const () as usize);

    for (name, value) in [
        ("LOG_EMERG", 0),
        ("LOG_ALERT", 1),
        ("LOG_CRIT", 2),
        ("LOG_ERR", 3),
        ("LOG_WARNING", 4),
        ("LOG_NOTICE", 5),
        ("LOG_INFO", 6),
        ("LOG_DEBUG", 7),
        ("LOG_PID", 1),
        ("LOG_CONS", 2),
        ("LOG_ODELAY", 4),
        ("LOG_NDELAY", 8),
        ("LOG_NOWAIT", 16),
        ("LOG_PERROR", 32),
        ("LOG_KERN", 0),
        ("LOG_USER", 8),
        ("LOG_MAIL", 16),
        ("LOG_DAEMON", 24),
        ("LOG_AUTH", 32),
        ("LOG_SYSLOG", 40),
        ("LOG_LPR", 48),
        ("LOG_NEWS", 56),
        ("LOG_UUCP", 64),
        ("LOG_CRON", 72),
        ("LOG_AUTHPRIV", 80),
        ("LOG_FTP", 88),
        ("LOG_LOCAL0", 128),
        ("LOG_LOCAL1", 136),
        ("LOG_LOCAL2", 144),
        ("LOG_LOCAL3", 152),
        ("LOG_LOCAL4", 160),
        ("LOG_LOCAL5", 168),
        ("LOG_LOCAL6", 176),
        ("LOG_LOCAL7", 184),
    ] {
        attrs.insert(name.to_string(), MbValue::from_int(value));
    }

    super::register_module("syslog", attrs);
}

pub fn mb_log_mask(args: &[MbValue]) -> MbValue {
    let Some(pri) = args.first().copied() else {
        return raise_type_error("LOG_MASK() missing required argument 'pri'");
    };
    match expect_int(pri, "pri") {
        Ok(pri) => MbValue::from_int(1_i64 << (pri.clamp(0, 62) as u32)),
        Err(err) => err,
    }
}

pub fn mb_log_upto(args: &[MbValue]) -> MbValue {
    let Some(pri) = args.first().copied() else {
        return raise_type_error("LOG_UPTO() missing required argument 'pri'");
    };
    match expect_int(pri, "pri") {
        Ok(pri) => MbValue::from_int((1_i64 << ((pri.clamp(0, 62) + 1) as u32)) - 1),
        Err(err) => err,
    }
}

pub fn mb_closelog() -> MbValue {
    MbValue::none()
}

pub fn mb_openlog(args: &[MbValue]) -> MbValue {
    if let Some(ident) = args.first().copied() {
        if !ident.is_none() {
            if let Err(err) = expect_str(ident, "ident") {
                return err;
            }
        }
    }
    if let Some(logoption) = args.get(1).copied() {
        if let Err(err) = expect_int(logoption, "logoption") {
            return err;
        }
    }
    if let Some(facility) = args.get(2).copied() {
        if let Err(err) = expect_int(facility, "facility") {
            return err;
        }
    }
    MbValue::none()
}

pub fn mb_setlogmask(args: &[MbValue]) -> MbValue {
    let Some(maskpri) = args.first().copied() else {
        return raise_type_error("setlogmask() missing required argument 'maskpri'");
    };
    match expect_int(maskpri, "maskpri") {
        Ok(maskpri) => MbValue::from_int(maskpri),
        Err(err) => err,
    }
}

pub fn mb_syslog(args: &[MbValue]) -> MbValue {
    match args.len() {
        0 => raise_type_error("syslog() missing required argument"),
        1 => match expect_str(args[0], "message") {
            Ok(_) => MbValue::none(),
            Err(err) => err,
        },
        _ => {
            if let Err(err) = expect_int(args[0], "priority") {
                return err;
            }
            match expect_str(args[1], "message") {
                Ok(_) => MbValue::none(),
                Err(err) => err,
            }
        }
    }
}
