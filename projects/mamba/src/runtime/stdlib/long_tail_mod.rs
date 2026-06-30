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

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
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

fn register_variadic_method_class(class_name: &str, method_name: &str, addr: usize) {
    super::super::module::register_variadic_func(addr as u64);
    let mut methods = HashMap::new();
    methods.insert(method_name.to_string(), MbValue::from_func(addr));
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], methods);
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
    dispatchers: &[(&str, usize)],
    consts_int: &[(&str, i64)],
    consts_str: &[(&str, &str)],
) -> HashMap<String, MbValue> {
    let mut attrs = HashMap::new();
    let shell = dispatch_class_shell as *const () as usize;
    let mut addrs: Vec<usize> = Vec::new();
    addrs.push(shell);
    for name in classes {
        attrs.insert((*name).into(), MbValue::from_func(shell));
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
    register_mailbox();
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
        &[],
        &[("FTP_PORT", 21), ("MSG_OOB", 1), ("MAXLINE", 8192)],
        &[("CRLF", "\r\n"), ("B_CRLF", "\r\n")],
    );
    super::register_module("ftplib", attrs);
}

fn register_poplib() {
    let attrs = build_attrs(
        &["POP3", "POP3_SSL", "error_proto"],
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
    let attrs = build_attrs(
        &[
            "IMAP4",
            "IMAP4_SSL",
            "IMAP4_stream",
            "Internaldate2tuple",
            "Int2AP",
            "ParseFlags",
            "Time2Internaldate",
        ],
        &[],
        &[
            ("IMAP4_PORT", 143),
            ("IMAP4_SSL_PORT", 993),
            ("AllowedVersions", 1),
        ],
        &[("CRLF", "\r\n"), ("Debug", "")],
    );
    super::register_module("imaplib", attrs);
}

fn register_telnetlib() {
    let mut attrs = build_attrs(
        &["Telnet"],
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
        &[],
        &[("NNTP_PORT", 119), ("NNTP_SSL_PORT", 563)],
        &[],
    );
    super::register_module("nntplib", attrs);
}

/// The pure-Python `mailbox` source (CPython 3.12), embedded at compile time.
///
/// The old stub registered a bare dict whose every class was a `lambda`-style
/// shell and whose `_ProxyFile`/`_PartialFile`/`Mailbox` mapping protocol did
/// nothing, so `mailbox._ProxyFile(...)`, `mailbox.Mailbox('path').add(...)`,
/// the mbox From-delimited store, and the Message/mboxMessage flag machinery
/// all returned `None`/empty. Instead we ship the real CPython source and let
/// Mamba's own compiler execute it (same approach as `plistlib_mod`): the file
/// is materialized to a per-build temp directory at startup and that directory
/// is added to the import search path, so `import mailbox` resolves to the real
/// implementation. Its only heavy dependency, `email`, is a real Mamba module.
const MAILBOX_SRC: &str = include_str!("py_src/mailbox.py");

fn register_mailbox() {
    // Materialize the embedded source to a stable temp directory and add that
    // directory to the import search path. We deliberately do NOT register a
    // native stub here: a registered module is pre-seeded into MODULES and wins
    // the import cache before find_module() is ever consulted, which would
    // shadow the real source. With no stub, `import mailbox` falls through to
    // the search path and loads py_src/mailbox.py. A user-supplied mailbox.py in
    // the running script's directory still wins (SCRIPT_DIR precedes SEARCH_PATHS).
    if let Some(dir) = materialize_mailbox_src() {
        super::super::module::mb_insert_search_path(0, &dir.display().to_string());
    }
}

fn materialize_mailbox_src() -> Option<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    use std::io::Write;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    MAILBOX_SRC.hash(&mut hasher);
    let h = hasher.finish();

    let mut dir = std::env::temp_dir();
    dir.push(format!("mamba_mailbox_{h:016x}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let file = dir.join("mailbox.py");
    let needs_write = match std::fs::read_to_string(&file) {
        Ok(existing) => existing != MAILBOX_SRC,
        Err(_) => true,
    };
    if needs_write {
        let tmp = dir.join(format!("mailbox.{}.tmp", std::process::id()));
        if let Ok(mut f) = std::fs::File::create(&tmp) {
            if f.write_all(MAILBOX_SRC.as_bytes()).is_ok() {
                let _ = std::fs::rename(&tmp, &file);
            }
        }
    }
    Some(dir)
}

fn register_cgitb() {
    let attrs = build_attrs(
        &["Hook"],
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
    let attrs = build_attrs(
        &["Shelf", "BsdDbShelf", "DbfilenameShelf"],
        &[("open", dispatch_empty_dict as *const () as usize)],
        &[],
        &[],
    );
    super::register_module("shelve", attrs);
}

fn register_pickletools() {
    let attrs = build_attrs(
        &["OpcodeInfo", "StackObject", "ArgumentDescriptor"],
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
        &[],
        &[],
        &[],
    );
    super::register_module("xdrlib", attrs);
}

fn register_marshal() {
    let attrs = build_attrs(
        &[],
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
    let attrs = build_attrs(
        &["Helper", "ModuleScanner", "TextDoc", "HTMLDoc", "Doc"],
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
    super::register_module("pydoc", attrs);
}

fn register_rlcompleter() {
    let attrs = build_attrs(
        &["Completer"],
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
        &[
            ("allocate_lock", dispatch_class_shell as *const () as usize),
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
