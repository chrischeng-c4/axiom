use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// Long-tail stdlib stub modules for Mamba (#1261).
///
/// Surface-only shims for stdlib modules legacy library probes import but
/// Mamba doesn't host any real machinery for. Each registered module is a
/// dict with callable class shells / no-op dispatchers so `import X` and
/// the usual attribute-existence checks don't crash.
///
/// Covered (alphabetical):
///   cgi, cgitb, filecmp, ftplib, imaplib, mailbox, marshal,
///   netrc, nntplib, ntpath, optparse, pickletools, plistlib, poplib,
///   posixpath, genericpath, pydoc, quopri, rlcompleter,
///   shelve, smtplib, stringprep, telnetlib, _thread, webbrowser,
///   xdrlib.
use std::collections::HashMap;

// #962 follow-up: `build_attrs` used to hand every "class" name (across every
// `register_*` call in this file, e.g. ftplib's FTP and _thread's LockType)
// the SAME single `dispatch_class_shell` address. Because NATIVE_TYPE_NAMES/
// FUNC_NAMES are address-keyed, whichever name registered last won
// `X.__name__` for every other name sharing that address — observed as
// `ftplib.FTP.__name__` reading back as `"LockType"` (a *different* module's
// class, registered later in this file's `register()`), and, even after
// giving each call its own shell, still misreading as `"error_perm"` (one of
// ftplib's OWN sibling shell classes, sharing that call's one shell). Plain
// `icf_guard!()` can't fix either case: it stops the *compiler* from folding
// distinct bodies onto one address, but here ONE symbol is deliberately
// reused, on purpose, across many class names. The real fix: give every
// shell class name a genuinely distinct function pointer, drawn from a pool
// of `SHELL_POOL_SIZE` individually fold-immune trivial stub functions,
// indexed by a compile-time-computed, non-overlapping `pool_start` per
// `build_attrs` call.
//
// IMPORTANT: this pool does NOT use `icf_guard!()` directly. That macro
// derives its fingerprint from `module_path!()`/`line!()`/`column!()`, which
// are resolved at the span of the *macro definition's* literal tokens — and
// for a single `macro_rules!` invocation that expands a `$(...)* `
// repetition into N functions (as here, one `def_shell_pool!(...)` call
// generating all 96 shells), every repetition shares that ONE span, so
// `line!()`/`column!()` come back IDENTICAL for all N and `icf_guard!()`
// silently fails to discriminate them (verified empirically: a minimal
// repro macro prints the same `file:line:col` for every repeated item).
// LLVM then folds all 96 "distinct" shells back onto a single address,
// reproducing the exact #954/#962 symptom one level down. The fix here
// instead fingerprints on `stringify!($name)`, which DOES vary per
// repetition (it's driven by the captured `$name` token's text, not by
// span), giving every pool slot a genuinely distinct compiled body.
const SHELL_POOL_SIZE: usize = 96;
type ShellFn = unsafe extern "C" fn(*const MbValue, usize) -> MbValue;

macro_rules! def_shell_pool {
    ($($name:ident),* $(,)?) => {
        $(
            unsafe extern "C" fn $name(_a: *const MbValue, _n: usize) -> MbValue {
                ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
                    module_path!(),
                    "::",
                    stringify!($name)
                )));
                MbValue::from_ptr(MbObject::new_dict())
            }
        )*
        const SHELL_POOL: [ShellFn; SHELL_POOL_SIZE] = [$($name),*];
    };
}
def_shell_pool!(
    shell_00, shell_01, shell_02, shell_03, shell_04, shell_05, shell_06, shell_07, shell_08,
    shell_09, shell_10, shell_11, shell_12, shell_13, shell_14, shell_15, shell_16, shell_17,
    shell_18, shell_19, shell_20, shell_21, shell_22, shell_23, shell_24, shell_25, shell_26,
    shell_27, shell_28, shell_29, shell_30, shell_31, shell_32, shell_33, shell_34, shell_35,
    shell_36, shell_37, shell_38, shell_39, shell_40, shell_41, shell_42, shell_43, shell_44,
    shell_45, shell_46, shell_47, shell_48, shell_49, shell_50, shell_51, shell_52, shell_53,
    shell_54, shell_55, shell_56, shell_57, shell_58, shell_59, shell_60, shell_61, shell_62,
    shell_63, shell_64, shell_65, shell_66, shell_67, shell_68, shell_69, shell_70, shell_71,
    shell_72, shell_73, shell_74, shell_75, shell_76, shell_77, shell_78, shell_79, shell_80,
    shell_81, shell_82, shell_83, shell_84, shell_85, shell_86, shell_87, shell_88, shell_89,
    shell_90, shell_91, shell_92, shell_93, shell_94, shell_95,
);

/// Pool slot at `idx` as a raw function-pointer address. Each `build_attrs`
/// call site passes a `pool_start` computed so its `classes` slice's slots
/// (`pool_start..pool_start+classes.len()`) never overlap another call's.
fn shell_addr(idx: usize) -> usize {
    SHELL_POOL[idx] as usize
}

unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}
unsafe extern "C" fn dispatch_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}
unsafe extern "C" fn dispatch_empty_dict(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}
unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn make_type_obj(name: &str, module: &str) -> MbValue {
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut map = fields.write().unwrap();
            map.insert("__name__".to_string(), new_str(name));
            map.insert("__qualname__".to_string(), new_str(name));
            map.insert("__module__".to_string(), new_str(module));
        }
    }
    MbValue::from_ptr(obj)
}

fn extract_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn is_bytes_like(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg));
    MbValue::none()
}

unsafe extern "C" fn telnet_write(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    let data = items.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(data) {
        return raise_type_error("Telnet.write() argument must be bytes-like");
    }
    MbValue::none()
}

unsafe extern "C" fn imap_idler_exit(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

unsafe extern "C" fn pydoc_error_during_import_init(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    let filename = items.first().copied().unwrap_or_else(MbValue::none);
    if !filename.is_none()
        && filename
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
            != Some(true)
    {
        return raise_type_error("ErrorDuringImport.__init__() filename argument must be str");
    }
    MbValue::none()
}

fn is_str_value(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
        == Some(true)
}

fn shelve_kw_filename(v: MbValue) -> Option<Option<MbValue>> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Dict(lock) => {
                let map = lock.read().unwrap();
                if map.keys().all(|key| {
                    matches!(
                        key.as_str(),
                        Some("filename")
                            | Some("flag")
                            | Some("protocol")
                            | Some("writeback")
                    )
                }) {
                    Some(
                        map.iter()
                            .find_map(|(key, value)| (key.as_str() == Some("filename")).then_some(*value)),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

unsafe extern "C" fn dispatch_shelve_dbfilename_shelf(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };

    let mut positional = args;
    let mut kw_filename = None;
    if let Some(filename) = args.last().copied().and_then(shelve_kw_filename) {
        positional = &args[..args.len().saturating_sub(1)];
        kw_filename = filename;
    }

    if let Some(filename) = positional.first().copied().or(kw_filename) {
        if !is_str_value(filename) {
            return raise_type_error("DbfilenameShelf.__init__() filename argument must be str");
        }
    }

    dispatch_empty_dict(args_ptr, nargs)
}

fn register_variadic_method_class(class_name: &str, method_name: &str, addr: usize) {
    super::super::module::register_variadic_func(addr as u64);
    let mut methods = HashMap::new();
    methods.insert(method_name.to_string(), MbValue::from_func(addr));
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], methods);
}

fn register_variadic_method_class_with_bases(
    class_name: &str,
    bases: Vec<String>,
    method_name: &str,
    addr: usize,
) {
    super::super::module::register_variadic_func(addr as u64);
    let mut methods = HashMap::new();
    methods.insert(method_name.to_string(), MbValue::from_func(addr));
    super::super::class::mb_class_register(class_name, bases, methods);
}

fn register_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

fn build_attrs(
    classes: &[&str],
    pool_start: usize,
    dispatchers: &[(&str, usize)],
    consts_int: &[(&str, i64)],
    consts_str: &[(&str, &str)],
) -> HashMap<String, MbValue> {
    let mut attrs = HashMap::new();
    let mut addrs: Vec<usize> = Vec::new();
    for (i, name) in classes.iter().enumerate() {
        let f = shell_addr(pool_start + i);
        attrs.insert((*name).into(), MbValue::from_func(f));
        addrs.push(f);
    }
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
        addrs.push(*addr);
    }
    for (name, v) in consts_int {
        attrs.insert((*name).into(), MbValue::from_int(*v));
    }
    for (name, v) in consts_str {
        attrs.insert(
            (*name).into(),
            MbValue::from_ptr(MbObject::new_str((*v).to_string())),
        );
    }
    register_addrs(&addrs);
    attrs
}

pub fn register() {
    register_smtplib();
    register_ftplib();
    register_poplib();
    register_imaplib();
    register_telnetlib();
    register_nntplib();
    // mailbox is provided by the shared vendored Lib/ tree (vendor_lib,
    // #867), which materializes py_src/mailbox.py once for the whole
    // process instead of this module's own per-call materialize_mailbox_src.
    // cgi is registered as a real module (cgi_mod) elsewhere; the
    // long_tail stub returned empty dicts/lists for every parse_*
    // function and empty strings for escape, breaking any old CGI
    // code path. Class shells stay (FieldStorage etc.) but the
    // pure-function subset now does real work.
    register_cgitb();
    // webbrowser is registered as a real module (webbrowser_mod) elsewhere;
    // the long_tail stub returned False from every open* call, breaking
    // scripts that try to launch the system URL handler.
    // quopri is registered as a real module (quopri_mod) elsewhere.
    // uu is registered as a real module (uu_mod) elsewhere; the long_tail
    // stub returned None from every encode/decode call, so any caller got
    // empty output instead of the uuencoded form.
    // stringprep is registered as a real module (stringprep_mod) elsewhere;
    // the long_tail stub returned False for every in_table_xxx check and
    // "" for the map_table_b2/b3 case-folding tables.
    // filecmp is registered as a real module (filecmp_mod) elsewhere; the
    // long_tail stub returned False for every `cmp()` call, breaking
    // anyone diffing files.
    // netrc is registered as a real module (netrc_mod) elsewhere; the
    // long_tail stub returned bare class shells, so `netrc.netrc(path)`
    // gave `{}` instead of parsed credentials.
    // plistlib is now registered as a real Python-source module
    // (plistlib_mod) with full FMT_XML / FMT_BINARY round-tripping, the UID
    // type, and the InvalidFileException hierarchy. The old long_tail stub
    // returned empty strings/dicts from every dump/load call.
    register_shelve();
    register_pickletools();
    register_xdrlib();
    register_marshal();
    register_optparse();
    // ntpath is registered as a real module (ntpath_mod) elsewhere; the
    // long_tail stub returned empty strings for every path-string op,
    // breaking any Windows path consumer.
    // posixpath and genericpath are registered as real modules (posixpath_mod)
    // elsewhere; the long_tail stub versions returned empty strings for
    // join/basename/dirname which broke every consumer.
    register_pydoc();
    // readline is registered as a real module (readline_mod) elsewhere; the
    // long_tail stub was no-op/empty-string for every history op, so any
    // CPython program reading or writing readline state got nothing back.
    register_rlcompleter();
    register_thread();
    // encodings (top-level package + .aliases + .utf_8 + .idna) is registered
    // as a real module (encodings_mod) elsewhere; the long_tail stub returned
    // "" from normalize_encoding and left aliases.aliases as an empty dict,
    // breaking any codec-name normalization path.
}

fn register_smtplib() {
    let attrs = build_attrs(
        &[
            "SMTP",
            "SMTP_SSL",
            "LMTP",
            "SMTPException",
            "SMTPServerDisconnected",
            "SMTPResponseException",
            "SMTPSenderRefused",
            "SMTPRecipientsRefused",
            "SMTPDataError",
            "SMTPConnectError",
            "SMTPHeloError",
            "SMTPNotSupportedError",
            "SMTPAuthenticationError",
            "quoteaddr",
            "quotedata",
        ],
        0,
        &[
            ("SMTP_PORT", dispatch_int_zero as *const () as usize),
            ("SMTP_SSL_PORT", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("SMTP_PORT", 25),
            ("SMTP_SSL_PORT", 465),
            ("LMTP_PORT", 2003),
        ],
        &[("CRLF", "\r\n"), ("bCRLF", "\r\n")],
    );
    super::register_module("smtplib", attrs);
}

fn register_ftplib() {
    let attrs = build_attrs(
        &[
            "FTP",
            "FTP_TLS",
            "Netrc",
            "error_reply",
            "error_temp",
            "error_perm",
            "error_proto",
            "all_errors",
        ],
        15,
        &[],
        &[("FTP_PORT", 21), ("MSG_OOB", 1), ("MAXLINE", 8192)],
        &[("CRLF", "\r\n"), ("B_CRLF", "\r\n")],
    );
    super::register_module("ftplib", attrs);
}

fn register_poplib() {
    let attrs = build_attrs(
        &["POP3", "POP3_SSL", "error_proto"],
        23,
        &[],
        &[
            ("POP3_PORT", 110),
            ("POP3_SSL_PORT", 995),
            ("CR", 13),
            ("LF", 10),
        ],
        &[("CRLF", "\r\n")],
    );
    super::register_module("poplib", attrs);
}

fn register_imaplib() {
    let mut attrs = build_attrs(
        &[
            "IMAP4",
            "IMAP4_SSL",
            "IMAP4_stream",
            "Internaldate2tuple",
            "Int2AP",
            "ParseFlags",
            "Time2Internaldate",
        ],
        26,
        &[],
        &[
            ("IMAP4_PORT", 143),
            ("IMAP4_SSL_PORT", 993),
            ("AllowedVersions", 1),
        ],
        &[("CRLF", "\r\n"), ("Debug", "")],
    );
    attrs.insert("Idler".into(), make_type_obj("Idler", "imaplib"));
    register_variadic_method_class("Idler", "__exit__", imap_idler_exit as *const () as usize);
    super::register_module("imaplib", attrs);
}

fn register_telnetlib() {
    let mut attrs = build_attrs(
        &["Telnet"],
        33,
        &[],
        &[
            ("DEBUGLEVEL", 0),
            ("TELNET_PORT", 23),
            ("IAC", 255),
            ("DONT", 254),
            ("DO", 253),
            ("WONT", 252),
            ("WILL", 251),
            ("SE", 240),
            ("NOP", 241),
            ("DM", 242),
            ("BRK", 243),
            ("IP", 244),
            ("AO", 245),
            ("AYT", 246),
            ("EC", 247),
            ("EL", 248),
            ("GA", 249),
            ("SB", 250),
        ],
        &[],
    );
    attrs.insert("Telnet".into(), make_type_obj("Telnet", "telnetlib"));
    register_variadic_method_class("Telnet", "write", telnet_write as *const () as usize);
    super::register_module("telnetlib", attrs);
}

fn register_nntplib() {
    let attrs = build_attrs(
        &[
            "NNTP",
            "NNTP_SSL",
            "NNTPError",
            "NNTPReplyError",
            "NNTPTemporaryError",
            "NNTPPermanentError",
            "NNTPProtocolError",
            "NNTPDataError",
            "decode_header",
        ],
        34,
        &[],
        &[("NNTP_PORT", 119), ("NNTP_SSL_PORT", 563)],
        &[],
    );
    super::register_module("nntplib", attrs);
}

/// `mailbox` is provided by the shared vendored Lib/ tree (`vendor_lib`,
/// #867): `py_src/mailbox.py`, adapted from CPython 3.12, materialized once
/// for the whole process. The old stub registered a bare dict whose every
/// class was a `lambda`-style shell and whose `_ProxyFile`/`_PartialFile`/
/// `Mailbox` mapping protocol did nothing, so `mailbox._ProxyFile(...)`,
/// `mailbox.Mailbox('path').add(...)`, the mbox From-delimited store, and the
/// Message/mboxMessage flag machinery all returned `None`/empty. Its only
/// heavy dependency, `email`, is a real Mamba module.

fn register_cgitb() {
    let attrs = build_attrs(
        &["Hook"],
        43,
        &[
            ("enable", dispatch_noop as *const () as usize),
            ("reset", dispatch_empty_str as *const () as usize),
            ("html", dispatch_empty_str as *const () as usize),
            ("text", dispatch_empty_str as *const () as usize),
            ("scanvars", dispatch_empty_list as *const () as usize),
            ("handler", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    super::register_module("cgitb", attrs);
}

fn register_shelve() {
    let mut attrs = build_attrs(
        &["Shelf", "BsdDbShelf"],
        44,
        &[("open", dispatch_empty_dict as *const () as usize)],
        &[],
        &[],
    );
    let dbfilename_shelf_addr = dispatch_shelve_dbfilename_shelf as *const () as usize;
    attrs.insert(
        "DbfilenameShelf".into(),
        MbValue::from_func(dbfilename_shelf_addr),
    );
    register_addrs(&[dbfilename_shelf_addr]);
    super::register_module("shelve", attrs);
}

fn register_pickletools() {
    let attrs = build_attrs(
        &["OpcodeInfo", "StackObject", "ArgumentDescriptor"],
        47,
        &[
            ("dis", dispatch_noop as *const () as usize),
            ("genops", dispatch_empty_list as *const () as usize),
            ("optimize", dispatch_empty_str as *const () as usize),
            ("read_uint1", dispatch_int_zero as *const () as usize),
            ("read_uint2", dispatch_int_zero as *const () as usize),
            ("read_int4", dispatch_int_zero as *const () as usize),
            ("read_string1", dispatch_empty_str as *const () as usize),
            ("read_string4", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    super::register_module("pickletools", attrs);
}

fn register_xdrlib() {
    let attrs = build_attrs(
        &["Packer", "Unpacker", "Error", "ConversionError"],
        50,
        &[],
        &[],
        &[],
    );
    super::register_module("xdrlib", attrs);
}

fn register_marshal() {
    let attrs = build_attrs(
        &[],
        54,
        &[
            ("dump", dispatch_noop as *const () as usize),
            ("dumps", dispatch_empty_str as *const () as usize),
            ("load", dispatch_noop as *const () as usize),
            ("loads", dispatch_noop as *const () as usize),
        ],
        &[("version", 4)],
        &[],
    );
    super::register_module("marshal", attrs);
}

fn register_optparse() {
    let attrs = build_attrs(
        &[
            "OptionParser",
            "Option",
            "OptionGroup",
            "OptionContainer",
            "OptionError",
            "OptionConflictError",
            "OptionValueError",
            "BadOptionError",
            "AmbiguousOptionError",
            "Values",
            "HelpFormatter",
            "IndentedHelpFormatter",
            "TitledHelpFormatter",
            "OptParseError",
            "check_choice",
            "check_builtin",
        ],
        54,
        &[],
        &[
            ("SUPPRESS_HELP", 0),
            ("SUPPRESS_USAGE", 0),
            ("NO_DEFAULT", 0),
        ],
        &[],
    );
    super::register_module("optparse", attrs);
}

fn register_pydoc() {
    let mut attrs = build_attrs(
        &["Helper", "ModuleScanner", "TextDoc", "HTMLDoc", "Doc"],
        70,
        &[
            ("help", dispatch_noop as *const () as usize),
            ("doc", dispatch_noop as *const () as usize),
            ("render_doc", dispatch_empty_str as *const () as usize),
            ("describe", dispatch_empty_str as *const () as usize),
            ("locate", dispatch_noop as *const () as usize),
            ("getdoc", dispatch_empty_str as *const () as usize),
            ("splitdoc", dispatch_empty_list as *const () as usize),
            ("classname", dispatch_empty_str as *const () as usize),
            ("plain", dispatch_empty_str as *const () as usize),
            ("pager", dispatch_noop as *const () as usize),
            ("plainpager", dispatch_noop as *const () as usize),
            ("getpager", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    // `render_doc`/`doc`/`resolve`/`writedoc` take `str | object` in
    // typeshed/CPython, so tightening them here would break real pydoc usage
    // on ordinary instances. `ErrorDuringImport.__init__` is the concrete wall:
    // its `filename` field is a real `str` contract.
    attrs.insert(
        "ErrorDuringImport".into(),
        make_type_obj("ErrorDuringImport", "pydoc"),
    );
    register_variadic_method_class_with_bases(
        "ErrorDuringImport",
        vec!["Exception".to_string()],
        "__init__",
        pydoc_error_during_import_init as *const () as usize,
    );
    super::register_module("pydoc", attrs);
}

fn register_rlcompleter() {
    let attrs = build_attrs(
        &["Completer"],
        75,
        &[(
            "readline_complete",
            dispatch_empty_str as *const () as usize,
        )],
        &[],
        &[],
    );
    super::register_module("rlcompleter", attrs);
}

fn register_thread() {
    let attrs = build_attrs(
        &["LockType", "RLock", "_local", "error"],
        76,
        &[
            ("allocate_lock", shell_addr(80)),
            ("get_ident", dispatch_int_zero as *const () as usize),
            ("get_native_id", dispatch_int_zero as *const () as usize),
            ("start_new_thread", dispatch_int_zero as *const () as usize),
            ("start_new", dispatch_int_zero as *const () as usize),
            ("exit", dispatch_noop as *const () as usize),
            ("exit_thread", dispatch_noop as *const () as usize),
            ("interrupt_main", dispatch_noop as *const () as usize),
            ("stack_size", dispatch_int_zero as *const () as usize),
            ("_count", dispatch_int_zero as *const () as usize),
        ],
        &[("TIMEOUT_MAX", 9223372036), ("_is_main_interpreter", 1)],
        &[],
    );
    super::register_module("_thread", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_exc() {
        super::super::super::exception::clear_current_exception();
    }

    fn raised_type() -> Option<String> {
        super::super::super::exception::current_exception_type()
    }

    #[test]
    fn test_dbfilename_shelf_accepts_str_filename() {
        clear_exc();
        let args = [MbValue::from_ptr(MbObject::new_str("cache.db".to_string()))];
        let result = unsafe { dispatch_shelve_dbfilename_shelf(args.as_ptr(), args.len()) };
        assert!(result.as_ptr().is_some());
        assert!(raised_type().is_none());
    }

    #[test]
    fn test_dbfilename_shelf_rejects_non_str_filename() {
        clear_exc();
        let args = [MbValue::from_int(12345)];
        let result = unsafe { dispatch_shelve_dbfilename_shelf(args.as_ptr(), args.len()) };
        assert!(result.is_none());
        assert_eq!(raised_type().as_deref(), Some("TypeError"));
        clear_exc();
    }

    #[test]
    fn test_dbfilename_shelf_accepts_missing_filename() {
        clear_exc();
        let result = unsafe { dispatch_shelve_dbfilename_shelf(std::ptr::null(), 0) };
        assert!(result.as_ptr().is_some());
        assert!(raised_type().is_none());
    }
}
