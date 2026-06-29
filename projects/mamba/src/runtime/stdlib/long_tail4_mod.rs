use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// Long-tail stub batch 4 for Mamba (#1261).
///
/// Final sweep: remaining xml/email/asyncio/concurrent dotted internals,
/// codec shims for the encoded-form modules (encodings.ascii etc.),
/// CPython `_*` C-extension shims (_io, _socket, _pickle, ...), and a
/// minimal probe-time shell for the third-party heavyweights legacy
/// code touches at import time (numpy, pandas, scipy, torch, tensorflow,
/// matplotlib, yaml, sklearn). The third-party shells are zero-machinery
/// — they exist purely so `import numpy` returns a dict instead of
/// crashing; any attribute lookup beyond that still fails normally.
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
unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}
unsafe extern "C" fn dispatch_false(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_str(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
}

fn is_io_stream_like(v: MbValue) -> bool {
    if v.is_none() || v.is_bool() || v.is_int() || v.is_float() {
        return false;
    }
    v.as_ptr().is_some_and(|p| unsafe {
        match &(*p).data {
            ObjData::Dict(_) => true,
            ObjData::Instance { class_name, .. } => matches!(
                class_name.as_str(),
                "IOBase"
                    | "RawIOBase"
                    | "BufferedIOBase"
                    | "TextIOBase"
                    | "FileIO"
                    | "BytesIO"
                    | "StringIO"
                    | "BufferedReader"
                    | "BufferedWriter"
                    | "BufferedRWPair"
                    | "BufferedRandom"
                    | "TextIOWrapper"
                    | "SpooledTemporaryFile"
                    | "NamedTemporaryFile"
            ),
            _ => false,
        }
    })
}

unsafe extern "C" fn dispatch_io_stream_constructor(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs == 0 {
        return raise_type_error("missing required stream argument");
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if !is_io_stream_like(a[0]) {
        return raise_type_error("expected an IO stream object");
    }
    dispatch_class_shell(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_io_rw_pair(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 2 {
        return raise_type_error("missing required stream argument");
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if !is_io_stream_like(a[0]) || !is_io_stream_like(a[1]) {
        return raise_type_error("expected IO stream objects");
    }
    dispatch_class_shell(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_io_text_encoding(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs > 0 {
        let encoding = unsafe { *args_ptr };
        if !encoding.is_none() && !is_str(encoding) {
            return raise_type_error("encoding must be str or None");
        }
    }
    dispatch_empty_str(args_ptr, nargs)
}

fn register_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

fn register_with(
    name: &str,
    classes: &[&str],
    dispatchers: &[(&str, usize)],
    consts_int: &[(&str, i64)],
    consts_str: &[(&str, &str)],
) {
    let mut attrs = HashMap::new();
    let shell = dispatch_class_shell as *const () as usize;
    let mut addrs = vec![shell];
    for cn in classes {
        attrs.insert((*cn).into(), MbValue::from_func(shell));
    }
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
        addrs.push(*a);
    }
    for (n, v) in consts_int {
        attrs.insert((*n).into(), MbValue::from_int(*v));
    }
    for (n, v) in consts_str {
        attrs.insert(
            (*n).into(),
            MbValue::from_ptr(MbObject::new_str((*v).to_string())),
        );
    }
    register_addrs(&addrs);
    super::register_module(name, attrs);
}

fn register_marker(name: &str) {
    // Marker module — a 1-key dict with __name__ so `import X` succeeds.
    let mut attrs = HashMap::new();
    attrs.insert(
        "__name__".into(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    super::register_module(name, attrs);
}

fn register_codec_module(name: &str) {
    // encodings.<codec> follows the same shape as encodings.utf_8 (PR #2373):
    // exposes `getregentry()` and decode/encode dispatchers. Sentinel-only.
    register_with(
        name,
        &[
            "Codec",
            "IncrementalEncoder",
            "IncrementalDecoder",
            "StreamWriter",
            "StreamReader",
        ],
        &[
            ("getregentry", dispatch_class_shell as *const () as usize),
            ("encode", dispatch_empty_str as *const () as usize),
            ("decode", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
}

pub fn register() {
    register_xml_remainder();
    register_email_internals();
    register_asyncio_remainder();
    register_concurrent_futures_subs();
    register_collections_underscore();
    register_codec_shims();
    register_msilib_subs();
    register_third_party_probe_shells();
    register_c_extensions();
    register_sched();
}

fn register_sched() {
    // `sched` top-level stdlib module (CPython 3.12). `scheduler` is the
    // event-scheduler class; `Event` is the per-event namedtuple. Both are
    // callable shells — surface only checks existence/callability.
    register_with("sched", &["scheduler", "Event"], &[], &[], &[]);
}

fn register_xml_sax_package() {
    // `xml.sax` public surface (CPython 3.12). Classes/functions are callable
    // shells; `default_parser_list` is a real list; `handler` and `xmlreader`
    // are submodules re-attached below so they survive this parent overwrite.
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    for cn in &[
        "ContentHandler",
        "ErrorHandler",
        "InputSource",
        "SAXException",
        "SAXParseException",
        "SAXNotRecognizedException",
        "SAXNotSupportedException",
        "SAXReaderNotAvailable",
    ] {
        attrs.insert((*cn).into(), MbValue::from_func(shell));
    }
    for fn_name in &["make_parser", "parse", "parseString"] {
        attrs.insert((*fn_name).into(), MbValue::from_func(shell));
    }
    attrs.insert(
        "default_parser_list".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    register_addrs(&[shell]);
    super::register_module("xml.sax", attrs);

    // Re-register the `xml.sax.*` submodules (originally defined in
    // long_tail3) so submodule-to-parent propagation re-attaches `handler`
    // and `xmlreader` as module-valued attributes on the `xml.sax` we just
    // overwrote. Attrs are reproduced identically — no information is lost.
    register_with(
        "xml.sax.handler",
        &[
            "ContentHandler",
            "DTDHandler",
            "EntityResolver",
            "ErrorHandler",
            "LexicalHandler",
        ],
        &[],
        &[
            ("feature_namespaces", 0),
            ("feature_namespace_prefixes", 0),
            ("feature_string_interning", 0),
            ("feature_validation", 0),
            ("feature_external_ges", 0),
            ("feature_external_pes", 0),
            ("property_lexical_handler", 0),
            ("property_declaration_handler", 0),
            ("property_dom_node", 0),
            ("property_xml_string", 0),
            ("property_encoding", 0),
            ("property_interning_dict", 0),
        ],
        &[],
    );
    register_with(
        "xml.sax.xmlreader",
        &[
            "XMLReader",
            "IncrementalParser",
            "Locator",
            "InputSource",
            "AttributesImpl",
            "AttributesNSImpl",
        ],
        &[],
        &[],
        &[],
    );
}

fn register_xml_remainder() {
    register_xml_sax_package();
    register_with(
        "xml.etree.ElementPath",
        &[],
        &[
            ("find", dispatch_class_shell as *const () as usize),
            ("findall", dispatch_empty_list as *const () as usize),
            ("findtext", dispatch_empty_str as *const () as usize),
            ("iterfind", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "xml.etree.ElementInclude",
        &["FatalIncludeError", "LimitedRecursiveIncludeError"],
        &[
            ("include", dispatch_noop as *const () as usize),
            ("default_loader", dispatch_class_shell as *const () as usize),
        ],
        &[("DEFAULT_MAX_INCLUSION_DEPTH", 6)],
        &[
            ("XINCLUDE", "{http://www.w3.org/2001/XInclude}"),
            (
                "XINCLUDE_INCLUDE",
                "{http://www.w3.org/2001/XInclude}include",
            ),
            (
                "XINCLUDE_FALLBACK",
                "{http://www.w3.org/2001/XInclude}fallback",
            ),
        ],
    );
    register_with(
        "xml.etree.cElementTree",
        &[
            "Element",
            "ElementTree",
            "SubElement",
            "Comment",
            "ProcessingInstruction",
            "QName",
            "XMLParser",
            "TreeBuilder",
            "iselement",
        ],
        &[
            ("parse", dispatch_class_shell as *const () as usize),
            ("fromstring", dispatch_class_shell as *const () as usize),
            ("tostring", dispatch_empty_str as *const () as usize),
            ("dump", dispatch_noop as *const () as usize),
            ("XML", dispatch_class_shell as *const () as usize),
            ("register_namespace", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
}

fn register_email_internals() {
    register_with(
        "email.base64mime",
        &[],
        &[
            ("body_encode", dispatch_empty_str as *const () as usize),
            ("body_decode", dispatch_empty_str as *const () as usize),
            ("decode", dispatch_empty_str as *const () as usize),
            ("decodestring", dispatch_empty_str as *const () as usize),
            ("header_length", dispatch_int_zero as *const () as usize),
            ("header_encode", dispatch_empty_str as *const () as usize),
        ],
        &[("CRLF", 0)],
        &[],
    );
    register_with(
        "email.quoprimime",
        &[],
        &[
            ("body_check", dispatch_false as *const () as usize),
            ("body_decode", dispatch_empty_str as *const () as usize),
            ("body_encode", dispatch_empty_str as *const () as usize),
            ("body_length", dispatch_int_zero as *const () as usize),
            ("decode", dispatch_empty_str as *const () as usize),
            ("decodestring", dispatch_empty_str as *const () as usize),
            ("header_check", dispatch_false as *const () as usize),
            ("header_decode", dispatch_empty_str as *const () as usize),
            ("header_encode", dispatch_empty_str as *const () as usize),
            ("header_length", dispatch_int_zero as *const () as usize),
            ("quote", dispatch_empty_str as *const () as usize),
            ("unquote", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("email._policybase", &["Policy", "Compat32"], &[], &[], &[]);
    register_with(
        "email._encoded_words",
        &[],
        &[
            ("decode", dispatch_empty_str as *const () as usize),
            ("encode", dispatch_empty_str as *const () as usize),
            ("decode_b", dispatch_empty_str as *const () as usize),
            ("decode_q", dispatch_empty_str as *const () as usize),
            ("encode_b", dispatch_empty_str as *const () as usize),
            ("encode_q", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "email._header_value_parser",
        &[
            "TokenList",
            "WhiteSpaceTerminal",
            "UnstructuredTokenList",
            "Phrase",
            "Word",
            "CFWSList",
            "Atom",
            "Token",
            "EncodedWord",
            "DotAtomText",
            "DotAtom",
            "AddrSpec",
            "LocalPart",
            "DomainLiteral",
            "Domain",
            "Address",
            "AddressList",
            "MailboxList",
            "Mailbox",
            "NameAddr",
            "AngleAddr",
            "GroupList",
            "Group",
            "DisplayName",
            "Identifier",
            "HeaderLabel",
            "Header",
            "ParameterizedHeaderValue",
            "Parameter",
            "MimeParameters",
            "MIMEVersion",
            "ContentType",
            "ContentDisposition",
            "ContentTransferEncoding",
            "BareQuotedString",
            "QuotedString",
            "Comment",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "email._parseaddr",
        &["AddrlistClass", "AddressList"],
        &[
            ("parseaddr", dispatch_empty_list as *const () as usize),
            ("quote", dispatch_empty_str as *const () as usize),
            ("mktime_tz", dispatch_int_zero as *const () as usize),
            ("parsedate", dispatch_empty_list as *const () as usize),
            ("parsedate_tz", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
}

fn register_asyncio_remainder() {
    register_with(
        "asyncio.format_helpers",
        &[],
        &[
            ("extract_stack", dispatch_empty_list as *const () as usize),
            ("format_helpers", dispatch_empty_str as *const () as usize),
            ("_get_function_source", dispatch_noop as *const () as usize),
            ("_format_callback", dispatch_empty_str as *const () as usize),
            (
                "_format_callback_source",
                dispatch_empty_str as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with("asyncio.log", &[], &[], &[], &[]);
    register_with(
        "asyncio.windows_events",
        &[
            "ProactorEventLoop",
            "WindowsSelectorEventLoopPolicy",
            "WindowsProactorEventLoopPolicy",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.windows_utils",
        &["PipeHandle"],
        &[("pipe", dispatch_empty_list as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "asyncio.unix_events",
        &[
            "SelectorEventLoop",
            "AbstractChildWatcher",
            "SafeChildWatcher",
            "FastChildWatcher",
            "PidfdChildWatcher",
            "MultiLoopChildWatcher",
            "ThreadedChildWatcher",
            "DefaultEventLoopPolicy",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.selector_events",
        &["BaseSelectorEventLoop"],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.proactor_events",
        &["BaseProactorEventLoop"],
        &[],
        &[],
        &[],
    );
    register_with("asyncio.taskgroups", &["TaskGroup"], &[], &[], &[]);
    register_with(
        "asyncio.timeouts",
        &["Timeout", "_State"],
        &[
            ("timeout", dispatch_class_shell as *const () as usize),
            ("timeout_at", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.staggered",
        &[],
        &[("staggered_race", dispatch_class_shell as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "asyncio.threads",
        &[],
        &[("to_thread", dispatch_class_shell as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "asyncio.constants",
        &[],
        &[],
        &[
            ("LOG_THRESHOLD_FOR_CONNLOST_WRITES", 5),
            ("ACCEPT_RETRY_DELAY", 1),
            ("DEBUG_STACK_DEPTH", 10),
            ("SSL_HANDSHAKE_TIMEOUT", 60),
            ("SENDFILE_FALLBACK_READBUFFER_SIZE", 262144),
            ("FLOW_CONTROL_HIGH_WATER_SSL_READ", 262144),
            ("FLOW_CONTROL_HIGH_WATER_SSL_WRITE", 524288),
            ("THREAD_JOIN_TIMEOUT", 300),
        ],
        &[],
    );
    register_with("asyncio.mixins", &["_LoopBoundMixin"], &[], &[], &[]);
}

fn register_concurrent_futures_subs() {
    register_with(
        "concurrent.futures.thread",
        &["ThreadPoolExecutor", "BrokenThreadPool", "_WorkItem"],
        &[],
        &[],
        &[],
    );
    register_with(
        "concurrent.futures.process",
        &[
            "ProcessPoolExecutor",
            "BrokenProcessPool",
            "_ResultItem",
            "_CallItem",
        ],
        &[(
            "EXTRA_QUEUED_CALLS",
            dispatch_int_zero as *const () as usize,
        )],
        &[],
        &[],
    );
    register_with(
        "concurrent.futures._base",
        &[
            "Future",
            "Executor",
            "CancelledError",
            "TimeoutError",
            "InvalidStateError",
            "BrokenExecutor",
            "FIRST_COMPLETED",
            "FIRST_EXCEPTION",
            "ALL_COMPLETED",
        ],
        &[
            ("wait", dispatch_class_shell as *const () as usize),
            ("as_completed", dispatch_empty_list as *const () as usize),
        ],
        &[
            ("PENDING", 1),
            ("RUNNING", 2),
            ("CANCELLED", 3),
            ("CANCELLED_AND_NOTIFIED", 4),
            ("FINISHED", 5),
            ("LOG_LEVEL", 30),
        ],
        &[
            ("FIRST_COMPLETED", "FIRST_COMPLETED"),
            ("FIRST_EXCEPTION", "FIRST_EXCEPTION"),
            ("ALL_COMPLETED", "ALL_COMPLETED"),
        ],
    );
}

fn register_collections_underscore() {
    // Some libraries write `from collections._collections_abc import X` instead
    // of the public `collections.abc` form. Mirror the surface.
    register_with(
        "collections._collections_abc",
        &[
            "Container",
            "Hashable",
            "Iterable",
            "Iterator",
            "Reversible",
            "Generator",
            "Sized",
            "Callable",
            "Collection",
            "Sequence",
            "MutableSequence",
            "ByteString",
            "Set",
            "MutableSet",
            "Mapping",
            "MutableMapping",
            "MappingView",
            "KeysView",
            "ItemsView",
            "ValuesView",
            "Awaitable",
            "Coroutine",
            "AsyncIterable",
            "AsyncIterator",
            "AsyncGenerator",
        ],
        &[],
        &[],
        &[],
    );
}

fn register_codec_shims() {
    for name in &[
        "encodings.ascii",
        "encodings.utf_16",
        "encodings.latin_1",
        "encodings.cp1252",
        "encodings.cp437",
        "encodings.cp850",
        "encodings.mbcs",
    ] {
        register_codec_module(name);
    }
}

fn register_msilib_subs() {
    for name in &["msilib.schema", "msilib.sequence", "msilib.text"] {
        register_marker(name);
    }
}

fn register_third_party_probe_shells() {
    // Zero-machinery shells for third-party packages legacy probe code
    // touches at import time. Goal: `import numpy` succeeds; any attribute
    // lookup beyond that still fails normally (which is the right behaviour
    // — the package's real functionality is not available).
    for name in &[
        "yaml",
        "numpy",
        "pandas",
        "matplotlib",
        "scipy",
        "tensorflow",
        "torch",
        "sklearn",
    ] {
        register_marker(name);
    }
}

fn register_c_extensions() {
    // CPython `_*` C-extension internals. Probe code occasionally falls
    // back to `import _io` etc. for low-level access. Provide minimum-viable
    // sentinels for the ones that are not already wired elsewhere.
    register_with(
        "_string",
        &[],
        &[
            (
                "formatter_field_name_split",
                dispatch_empty_list as *const () as usize,
            ),
            (
                "formatter_parser",
                dispatch_empty_list as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "_decimal",
        &[
            "Decimal",
            "Context",
            "DecimalException",
            "Clamped",
            "DivisionByZero",
            "InvalidOperation",
            "Overflow",
            "Rounded",
            "Subnormal",
            "Underflow",
            "Inexact",
            "FloatOperation",
            "DefaultContext",
            "BasicContext",
            "ExtendedContext",
        ],
        &[
            ("getcontext", dispatch_class_shell as *const () as usize),
            ("setcontext", dispatch_noop as *const () as usize),
            ("localcontext", dispatch_class_shell as *const () as usize),
        ],
        &[
            ("ROUND_HALF_EVEN", 0),
            ("ROUND_HALF_DOWN", 1),
            ("ROUND_HALF_UP", 2),
            ("ROUND_FLOOR", 3),
            ("ROUND_CEILING", 4),
            ("ROUND_DOWN", 5),
            ("ROUND_UP", 6),
            ("ROUND_05UP", 7),
            ("MAX_PREC", 425000000),
            ("MAX_EMAX", 425000000),
            ("MIN_EMIN", -425000000),
        ],
        &[],
    );
    register_with(
        "_json",
        &["make_encoder", "make_scanner"],
        &[
            (
                "encode_basestring",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "encode_basestring_ascii",
                dispatch_empty_str as *const () as usize,
            ),
            ("scanstring", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "_pickle",
        &[
            "Pickler",
            "Unpickler",
            "PicklingError",
            "UnpicklingError",
            "PickleError",
        ],
        &[
            ("dump", dispatch_noop as *const () as usize),
            ("dumps", dispatch_empty_str as *const () as usize),
            ("load", dispatch_noop as *const () as usize),
            ("loads", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("_random", &["Random"], &[], &[], &[]);
    register_with(
        "_socket",
        &[
            "socket", "gaierror", "herror", "error", "timeout", "SocketIO",
        ],
        &[
            ("gethostname", dispatch_empty_str as *const () as usize),
            ("gethostbyname", dispatch_empty_str as *const () as usize),
            ("getaddrinfo", dispatch_empty_list as *const () as usize),
            ("getservbyname", dispatch_int_zero as *const () as usize),
            ("getservbyport", dispatch_empty_str as *const () as usize),
            ("inet_aton", dispatch_empty_str as *const () as usize),
            ("inet_ntoa", dispatch_empty_str as *const () as usize),
            ("inet_pton", dispatch_empty_str as *const () as usize),
            ("inet_ntop", dispatch_empty_str as *const () as usize),
            ("htons", dispatch_int_zero as *const () as usize),
            ("htonl", dispatch_int_zero as *const () as usize),
            ("ntohs", dispatch_int_zero as *const () as usize),
            ("ntohl", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("AF_INET", 2),
            ("AF_INET6", 10),
            ("AF_UNIX", 1),
            ("AF_UNSPEC", 0),
            ("SOCK_STREAM", 1),
            ("SOCK_DGRAM", 2),
            ("SOCK_RAW", 3),
            ("SOL_SOCKET", 1),
            ("SO_REUSEADDR", 2),
            ("SO_KEEPALIVE", 9),
            ("IPPROTO_TCP", 6),
            ("IPPROTO_UDP", 17),
            ("IPPROTO_IP", 0),
            ("INADDR_ANY", 0),
            ("INADDR_BROADCAST", -1),
        ],
        &[],
    );
    register_with(
        "_signal",
        &[],
        &[
            ("signal", dispatch_noop as *const () as usize),
            ("getsignal", dispatch_int_zero as *const () as usize),
            ("alarm", dispatch_int_zero as *const () as usize),
            ("pause", dispatch_noop as *const () as usize),
            ("set_wakeup_fd", dispatch_int_zero as *const () as usize),
            ("default_int_handler", dispatch_noop as *const () as usize),
            ("raise_signal", dispatch_noop as *const () as usize),
            ("strsignal", dispatch_empty_str as *const () as usize),
        ],
        &[
            ("SIG_DFL", 0),
            ("SIG_IGN", 1),
            ("SIGHUP", 1),
            ("SIGINT", 2),
            ("SIGQUIT", 3),
            ("SIGILL", 4),
            ("SIGTRAP", 5),
            ("SIGABRT", 6),
            ("SIGBUS", 7),
            ("SIGFPE", 8),
            ("SIGKILL", 9),
            ("SIGSEGV", 11),
            ("SIGPIPE", 13),
            ("SIGALRM", 14),
            ("SIGTERM", 15),
            ("SIGUSR1", 10),
            ("SIGUSR2", 12),
            ("SIGCHLD", 17),
            ("SIGCONT", 18),
            ("SIGSTOP", 19),
            ("SIGTSTP", 20),
            ("SIGTTIN", 21),
            ("SIGTTOU", 22),
            ("NSIG", 65),
        ],
        &[],
    );
    register_with(
        "_io",
        &[
            "IOBase",
            "RawIOBase",
            "BufferedIOBase",
            "TextIOBase",
            "FileIO",
            "BytesIO",
            "StringIO",
            "BufferedReader",
            "BufferedWriter",
            "BufferedRWPair",
            "BufferedRandom",
            "TextIOWrapper",
            "UnsupportedOperation",
            "BlockingIOError",
            "IncrementalNewlineDecoder",
        ],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("open_code", dispatch_class_shell as *const () as usize),
            (
                "BufferedReader",
                dispatch_io_stream_constructor as *const () as usize,
            ),
            (
                "BufferedWriter",
                dispatch_io_stream_constructor as *const () as usize,
            ),
            ("BufferedRWPair", dispatch_io_rw_pair as *const () as usize),
            (
                "TextIOWrapper",
                dispatch_io_stream_constructor as *const () as usize,
            ),
            (
                "text_encoding",
                dispatch_io_text_encoding as *const () as usize,
            ),
        ],
        &[("DEFAULT_BUFFER_SIZE", 8192)],
        &[],
    );
    register_with(
        "_struct",
        &["Struct", "error"],
        &[
            ("calcsize", dispatch_int_zero as *const () as usize),
            ("pack", dispatch_empty_str as *const () as usize),
            ("pack_into", dispatch_noop as *const () as usize),
            ("unpack", dispatch_empty_list as *const () as usize),
            ("unpack_from", dispatch_empty_list as *const () as usize),
            ("iter_unpack", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "_warnings",
        &[],
        &[
            ("warn", dispatch_noop as *const () as usize),
            ("warn_explicit", dispatch_noop as *const () as usize),
            ("_filters_mutated", dispatch_noop as *const () as usize),
            ("_acquire_lock", dispatch_noop as *const () as usize),
            ("_release_lock", dispatch_noop as *const () as usize),
        ],
        &[("_defaultaction", 0), ("_onceregistry", 0), ("filters", 0)],
        &[],
    );
    register_with(
        "_csv",
        &["Dialect", "Error", "__doc__"],
        &[
            ("reader", dispatch_class_shell as *const () as usize),
            ("writer", dispatch_class_shell as *const () as usize),
            ("register_dialect", dispatch_noop as *const () as usize),
            ("unregister_dialect", dispatch_noop as *const () as usize),
            ("get_dialect", dispatch_class_shell as *const () as usize),
            ("list_dialects", dispatch_empty_list as *const () as usize),
            ("field_size_limit", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("QUOTE_MINIMAL", 0),
            ("QUOTE_ALL", 1),
            ("QUOTE_NONNUMERIC", 2),
            ("QUOTE_NONE", 3),
            ("QUOTE_STRINGS", 4),
            ("QUOTE_NOTNULL", 5),
        ],
        &[("__version__", "1.0")],
    );
}
