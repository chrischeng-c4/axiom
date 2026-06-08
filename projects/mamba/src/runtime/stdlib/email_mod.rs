use super::super::rc::MbObject;
use super::super::value::MbValue;
/// email module + submodules for Mamba (#1422, #1261 long-tail).
///
/// Surface-only shim covering the most-imported entry points across
/// `email`, `email.utils`, `email.message`, `email.policy`,
/// `email.parser`, `email.header`, and the `email.mime.*` submodules.
/// Every dispatcher returns an identity-stable sentinel so that 3p
/// libraries (Flask, requests, httpx) can `from email.utils import ...`
/// at probe time without exploding. Full functional conformance is
/// tracked separately under #1422; this batch ships the Gate 2
/// module-attr-read perf surface plus the dispatcher class shells.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_dict_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_parseaddr(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython returns a 2-tuple ('', addr) for unparseable inputs; we return a 2-element list.
    let empty = || MbValue::from_ptr(MbObject::new_str(String::new()));
    MbValue::from_ptr(MbObject::new_list(vec![empty(), empty()]))
}

unsafe extern "C" fn dispatch_formataddr(a: *const MbValue, n: usize) -> MbValue {
    // formataddr((realname, email_addr)) -> "realname <addr>"
    if n == 0 {
        return MbValue::from_ptr(MbObject::new_str(String::new()));
    }
    let arg = unsafe { *a };
    use super::super::rc::ObjData;
    if let Some(p) = arg.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                let list = lock.read().unwrap();
                if list.len() == 2 {
                    let realname = list[0]
                        .as_ptr()
                        .and_then(|p2| match &(*p2).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let addr = list[1]
                        .as_ptr()
                        .and_then(|p2| match &(*p2).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let s = if realname.is_empty() {
                        addr
                    } else {
                        format!("{realname} <{addr}>")
                    };
                    return MbValue::from_ptr(MbObject::new_str(s));
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_make_msgid(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython format: <UID.timestamp@host>. Sentinel just returns "<id@localhost>".
    MbValue::from_ptr(MbObject::new_str("<mamba@localhost>".to_string()))
}

pub fn register() {
    register_email_root();
    register_email_utils();
    register_email_message();
    register_email_policy();
    register_email_parser();
    register_email_header();
    register_email_mime();
}

fn register_email_root() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        (
            "message_from_string",
            dispatch_dict_shell as *const () as usize,
        ),
        (
            "message_from_bytes",
            dispatch_dict_shell as *const () as usize,
        ),
        (
            "message_from_file",
            dispatch_dict_shell as *const () as usize,
        ),
        (
            "message_from_binary_file",
            dispatch_dict_shell as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("email", attrs);
}

fn register_email_utils() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("formatdate", dispatch_empty_str as *const () as usize),
        ("format_datetime", dispatch_empty_str as *const () as usize),
        ("parseaddr", dispatch_parseaddr as *const () as usize),
        ("formataddr", dispatch_formataddr as *const () as usize),
        ("getaddresses", dispatch_empty_list as *const () as usize),
        ("parsedate", dispatch_empty_list as *const () as usize),
        ("parsedate_tz", dispatch_empty_list as *const () as usize),
        (
            "parsedate_to_datetime",
            dispatch_dict_shell as *const () as usize,
        ),
        ("mktime_tz", dispatch_int_zero as *const () as usize),
        ("quote", dispatch_empty_str as *const () as usize),
        ("unquote", dispatch_empty_str as *const () as usize),
        ("make_msgid", dispatch_make_msgid as *const () as usize),
        (
            "collapse_rfc2231_value",
            dispatch_empty_str as *const () as usize,
        ),
        ("decode_rfc2231", dispatch_empty_list as *const () as usize),
        ("encode_rfc2231", dispatch_empty_str as *const () as usize),
        ("decode_params", dispatch_empty_list as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("email.utils", attrs);
}

fn register_email_message() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("Message", dispatch_dict_shell as *const () as usize),
        ("EmailMessage", dispatch_dict_shell as *const () as usize),
        ("MIMEPart", dispatch_dict_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("email.message", attrs);
}

fn register_email_policy() {
    let mut attrs = HashMap::new();
    // Policy singletons in CPython are module-level instances, not callables.
    // Surface stub uses empty dicts so `email.policy.default` reads cleanly.
    for name in &["compat32", "default", "SMTP", "SMTPUTF8", "HTTP", "strict"] {
        attrs.insert((*name).into(), MbValue::from_ptr(MbObject::new_dict()));
    }
    // Policy class shell.
    let addr = dispatch_dict_shell as *const () as usize;
    attrs.insert("Policy".into(), MbValue::from_func(addr));
    attrs.insert("EmailPolicy".into(), MbValue::from_func(addr));
    attrs.insert("Compat32".into(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    super::register_module("email.policy", attrs);
}

fn register_email_parser() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("Parser", dispatch_dict_shell as *const () as usize),
        ("BytesParser", dispatch_dict_shell as *const () as usize),
        ("HeaderParser", dispatch_dict_shell as *const () as usize),
        ("FeedParser", dispatch_dict_shell as *const () as usize),
        ("BytesFeedParser", dispatch_dict_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("email.parser", attrs);
}

fn register_email_header() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("Header", dispatch_dict_shell as *const () as usize),
        ("decode_header", dispatch_empty_list as *const () as usize),
        ("make_header", dispatch_dict_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("email.header", attrs);
}

fn register_email_mime() {
    // Each MIME submodule registers exactly one class shell with the
    // canonical class name (CPython preserves the per-submodule
    // namespace so `from email.mime.text import MIMEText` resolves).
    let dispatchers: &[(&str, &str)] = &[
        ("email.mime", "MIMEBase"),
        ("email.mime.base", "MIMEBase"),
        ("email.mime.text", "MIMEText"),
        ("email.mime.multipart", "MIMEMultipart"),
        ("email.mime.image", "MIMEImage"),
        ("email.mime.audio", "MIMEAudio"),
        ("email.mime.application", "MIMEApplication"),
        ("email.mime.message", "MIMEMessage"),
        ("email.mime.nonmultipart", "MIMENonMultipart"),
    ];
    let addr = dispatch_dict_shell as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    for (mod_name, cls_name) in dispatchers {
        let mut attrs = HashMap::new();
        attrs.insert((*cls_name).into(), MbValue::from_func(addr));
        super::register_module(mod_name, attrs);
    }
}
