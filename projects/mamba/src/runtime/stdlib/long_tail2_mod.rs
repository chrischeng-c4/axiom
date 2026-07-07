use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// Long-tail stub batch 2 for Mamba (#1261).
///
/// Mostly dotted submodules of existing packages (json.*, logging.*,
/// asyncio.*, multiprocessing.*, concurrent.*) plus a handful of
/// stand-alone deprecated / niche stdlib modules (tkinter, turtle,
/// curses, the audio family, the deprecated mail tools).
use std::collections::HashMap;

/// #1040 follow-up: `dispatch_class_shell` is kept as a MARKER address only.
/// Every call site below still writes `dispatch_class_shell as *const () as
/// usize` (for `dispatchers` tuples) or a bare class name (for the `classes`
/// list) exactly as before -- `build_attrs` (below) detects the marker
/// address / every `classes` entry and transparently substitutes a genuinely
/// distinct `SHELL_POOL` slot before the value is ever inserted into a
/// module's attrs. This file funnels literally every `dispatch_class_shell`
/// use (all 63 `register_with` calls + the direct `build_attrs` calls) through
/// that one chokepoint, so centralizing the fix there covers the whole file
/// without editing any of the ~300 individual call sites.
unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    crate::icf_guard!();
    MbValue::from_ptr(MbObject::new_dict())
}

// #1040 follow-up: this file's `dispatch_class_shell` used to be handed out
// as the SAME function address to every class-shell name registered here,
// across every `register_*` call in this file. Because FUNC_NAMES/
// NATIVE_FUNC_ADDRS are address-keyed, whichever name registered last (in
// HashMap iteration order, which is nondeterministic per process) won
// `X.__name__` for every other class sharing that address -- the same
// #962/#954 symptom. The fix: give every class-shell name a genuinely
// distinct function pointer, drawn from a pool of `SHELL_POOL_SIZE`
// individually fold-immune trivial stub functions, indexed via a
// thread-local "next free slot" counter (`next_shell_slot`) so every call
// site simply draws a fresh slot per name -- no manual per-call `pool_start`
// bookkeeping required, since `register()` runs registration sequentially
// on a single thread at module-init time.
//
// IMPORTANT: this pool does NOT use `icf_guard!()` directly. That macro
// derives its fingerprint from `module_path!()`/`line!()`/`column!()`, which
// are resolved at the span of the *macro definition's* literal tokens -- for
// a single `macro_rules!` invocation that expands a `$(...)* ` repetition
// into N functions, every repetition shares that ONE span, so
// `line!()`/`column!()` come back IDENTICAL for all N and `icf_guard!()`
// silently fails to discriminate them. LLVM then folds all "distinct"
// shells back onto a single address, reproducing the exact bug one level
// down. The fix here instead fingerprints on `stringify!($name)`, which DOES
// vary per repetition (driven by the captured `$name` token's text, not by
// span), giving every pool slot a genuinely distinct compiled body.
const SHELL_POOL_SIZE: usize = 340;
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
    shell_90, shell_91, shell_92, shell_93, shell_94, shell_95, shell_96, shell_97, shell_98,
    shell_99, shell_100, shell_101, shell_102, shell_103, shell_104, shell_105, shell_106,
    shell_107, shell_108, shell_109, shell_110, shell_111, shell_112, shell_113, shell_114,
    shell_115, shell_116, shell_117, shell_118, shell_119, shell_120, shell_121, shell_122,
    shell_123, shell_124, shell_125, shell_126, shell_127, shell_128, shell_129, shell_130,
    shell_131, shell_132, shell_133, shell_134, shell_135, shell_136, shell_137, shell_138,
    shell_139, shell_140, shell_141, shell_142, shell_143, shell_144, shell_145, shell_146,
    shell_147, shell_148, shell_149, shell_150, shell_151, shell_152, shell_153, shell_154,
    shell_155, shell_156, shell_157, shell_158, shell_159, shell_160, shell_161, shell_162,
    shell_163, shell_164, shell_165, shell_166, shell_167, shell_168, shell_169, shell_170,
    shell_171, shell_172, shell_173, shell_174, shell_175, shell_176, shell_177, shell_178,
    shell_179, shell_180, shell_181, shell_182, shell_183, shell_184, shell_185, shell_186,
    shell_187, shell_188, shell_189, shell_190, shell_191, shell_192, shell_193, shell_194,
    shell_195, shell_196, shell_197, shell_198, shell_199, shell_200, shell_201, shell_202,
    shell_203, shell_204, shell_205, shell_206, shell_207, shell_208, shell_209, shell_210,
    shell_211, shell_212, shell_213, shell_214, shell_215, shell_216, shell_217, shell_218,
    shell_219, shell_220, shell_221, shell_222, shell_223, shell_224, shell_225, shell_226,
    shell_227, shell_228, shell_229, shell_230, shell_231, shell_232, shell_233, shell_234,
    shell_235, shell_236, shell_237, shell_238, shell_239, shell_240, shell_241, shell_242,
    shell_243, shell_244, shell_245, shell_246, shell_247, shell_248, shell_249, shell_250,
    shell_251, shell_252, shell_253, shell_254, shell_255, shell_256, shell_257, shell_258,
    shell_259, shell_260, shell_261, shell_262, shell_263, shell_264, shell_265, shell_266,
    shell_267, shell_268, shell_269, shell_270, shell_271, shell_272, shell_273, shell_274,
    shell_275, shell_276, shell_277, shell_278, shell_279, shell_280, shell_281, shell_282,
    shell_283, shell_284, shell_285, shell_286, shell_287, shell_288, shell_289, shell_290,
    shell_291, shell_292, shell_293, shell_294, shell_295, shell_296, shell_297, shell_298,
    shell_299, shell_300, shell_301, shell_302, shell_303, shell_304, shell_305, shell_306,
    shell_307, shell_308, shell_309, shell_310, shell_311, shell_312, shell_313, shell_314,
    shell_315, shell_316, shell_317, shell_318, shell_319, shell_320, shell_321, shell_322,
    shell_323, shell_324, shell_325, shell_326, shell_327, shell_328, shell_329, shell_330,
    shell_331, shell_332, shell_333, shell_334, shell_335, shell_336, shell_337, shell_338,
    shell_339,
);

/// Pool slot at `idx` as a raw function-pointer address.
fn shell_addr(idx: usize) -> usize {
    SHELL_POOL[idx] as usize
}

thread_local! {
    static NEXT_SHELL_SLOT: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

/// Draw the next unused pool slot index. `register()` runs sequentially on
/// a single thread at module-init time, so a simple monotonic counter gives
/// every class-shell name a fresh, non-overlapping slot with no manual
/// per-call range bookkeeping.
fn next_shell_slot() -> usize {
    NEXT_SHELL_SLOT.with(|c| {
        let v = c.get();
        assert!(
            v < SHELL_POOL_SIZE,
            "shell pool exhausted (SHELL_POOL_SIZE={}); bump it",
            SHELL_POOL_SIZE
        );
        c.set(v + 1);
        v
    })
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
unsafe extern "C" fn dispatch_false(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
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

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn single_char_code(s: &str) -> Result<i64, MbValue> {
    let mut chars = s.chars();
    let Some(ch) = chars.next() else {
        return Err(raise_type_error(
            "ord() expected a character, but string of length 0 found",
        ));
    };
    if chars.next().is_some() {
        return Err(raise_type_error(&format!(
            "ord() expected a character, but string of length {} found",
            s.chars().count()
        )));
    }
    Ok(ch as i64)
}

fn curses_ascii_arg(value: MbValue, op: &str) -> Result<(i64, bool), MbValue> {
    if let Some(s) = extract_str(value) {
        return single_char_code(&s).map(|code| (code, true));
    }
    if let Some(i) = value.as_int_pyint() {
        return Ok((i, false));
    }
    Err(raise_type_error(&format!(
        "unsupported operand type(s) for {op}: object and int"
    )))
}

fn codepoint_str(code: i64) -> MbValue {
    let ch = char::from_u32(code as u32).unwrap_or('\u{fffd}');
    MbValue::from_ptr(MbObject::new_str(ch.to_string()))
}

unsafe extern "C" fn curses_ascii_ascii(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 1 {
        return raise_type_error("ascii() takes exactly one argument");
    }
    let arg = unsafe { *args_ptr };
    let Ok((code, was_str)) = curses_ascii_arg(arg, "&") else {
        return MbValue::none();
    };
    let out = code & 0x7f;
    if was_str {
        codepoint_str(out)
    } else {
        MbValue::from_int(out)
    }
}

unsafe extern "C" fn curses_ascii_alt(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 1 {
        return raise_type_error("alt() takes exactly one argument");
    }
    let arg = unsafe { *args_ptr };
    let Ok((code, was_str)) = curses_ascii_arg(arg, "|") else {
        return MbValue::none();
    };
    let out = code | 0x80;
    if was_str {
        codepoint_str(out)
    } else {
        MbValue::from_int(out)
    }
}

unsafe extern "C" fn curses_ascii_ctrl(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 1 {
        return raise_type_error("ctrl() takes exactly one argument");
    }
    let arg = unsafe { *args_ptr };
    let Ok((code, was_str)) = curses_ascii_arg(arg, "&") else {
        return MbValue::none();
    };
    let out = code & 0x1f;
    if was_str {
        codepoint_str(out)
    } else {
        MbValue::from_int(out)
    }
}

fn curses_ascii_unctrl_text(code: i64) -> String {
    let mut c = code & 0xff;
    let mut out = String::new();
    if (c & 0x80) != 0 {
        out.push('!');
        c &= 0x7f;
    }
    if c == 0x7f {
        out.push_str("^?");
    } else if c < 0x20 {
        out.push('^');
        out.push(char::from_u32((c + 0x40) as u32).unwrap_or('\u{fffd}'));
    } else {
        out.push(char::from_u32(c as u32).unwrap_or('\u{fffd}'));
    }
    out
}

unsafe extern "C" fn curses_ascii_unctrl(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 1 {
        return raise_type_error("unctrl() takes exactly one argument");
    }
    let arg = unsafe { *args_ptr };
    let Ok((code, _was_str)) = curses_ascii_arg(arg, "&") else {
        return MbValue::none();
    };
    new_str(&curses_ascii_unctrl_text(code))
}

unsafe extern "C" fn write_transport_write(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    let data = items.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(data) {
        return raise_type_error("WriteTransport.write() argument must be bytes-like");
    }
    MbValue::none()
}

unsafe extern "C" fn raw_turtle_write(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    if let Some(move_arg) = items.get(1) {
        if !move_arg.is_bool() {
            return raise_type_error("RawTurtle.write() move argument must be bool");
        }
    }
    MbValue::none()
}

unsafe extern "C" fn transport_socket_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn curses_bool_flag(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    raise_type_error("curses flag must be bool")
}

fn register_variadic_method_class(class_name: &str, method_name: &str, addr: usize) {
    super::super::module::register_variadic_func(addr as u64);
    let mut methods = HashMap::new();
    methods.insert(method_name.to_string(), MbValue::from_func(addr));
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], methods);
}

fn register_variadic_method_class_many(class_name: &str, methods: &[(&str, usize)]) {
    let mut map = HashMap::new();
    for (method_name, addr) in methods {
        super::super::module::register_variadic_func(*addr as u64);
        map.insert((*method_name).to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], map);
}

/// logging.config.fileConfig(fname) — validates the config file: a missing
/// path raises FileNotFoundError and a file with no INI sections (e.g. empty)
/// raises RuntimeError, matching CPython. Full INI-driven logger setup is not
/// modeled, so a well-formed file is accepted as a no-op.
unsafe extern "C" fn dispatch_file_config(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let path = a.first().copied().and_then(|v| {
        v.as_ptr().and_then(|p| unsafe {
            if let super::super::rc::ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
    });
    if let Some(path) = path {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if !content.contains('[') {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "{path} does not contain configuration information"
                        ))),
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("FileNotFoundError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "[Errno 2] No such file or directory: '{path}'"
                    ))),
                );
            }
            Err(_) => {}
        }
    }
    MbValue::none()
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
    let attrs = build_attrs(classes, dispatchers, consts_int, consts_str);
    super::register_module(name, attrs);
}

fn build_attrs(
    classes: &[&str],
    dispatchers: &[(&str, usize)],
    consts_int: &[(&str, i64)],
    consts_str: &[(&str, &str)],
) -> HashMap<String, MbValue> {
    let mut attrs = HashMap::new();
    // #1040: every `classes` entry used to share ONE `dispatch_class_shell`
    // address across the WHOLE file (267+ names), and every `dispatchers`
    // tuple using the `dispatch_class_shell` marker address shared that same
    // one address too -- both are the exact FUNC_NAMES/NATIVE_FUNC_ADDRS
    // address-collision bug from #962/#954. Give each a fresh, genuinely
    // distinct SHELL_POOL slot instead; `dispatch_class_shell` itself is
    // never handed out anymore, only used as a marker to detect (and
    // replace) the legacy shared address in `dispatchers` tuples.
    let class_shell_marker = dispatch_class_shell as *const () as usize;
    let mut addrs = Vec::new();
    for cn in classes {
        let f = shell_addr(next_shell_slot());
        addrs.push(f);
        attrs.insert((*cn).into(), MbValue::from_func(f));
    }
    for (n, a) in dispatchers {
        let addr = if *a == class_shell_marker {
            shell_addr(next_shell_slot())
        } else {
            *a
        };
        attrs.insert((*n).into(), MbValue::from_func(addr));
        addrs.push(addr);
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
    attrs
}

fn register_curses_ascii() {
    let mut attrs = build_attrs(
        &[],
        &[
            ("ascii", curses_ascii_ascii as *const () as usize),
            ("alt", curses_ascii_alt as *const () as usize),
            ("ctrl", curses_ascii_ctrl as *const () as usize),
            ("unctrl", curses_ascii_unctrl as *const () as usize),
            ("isalnum", dispatch_false as *const () as usize),
            ("isalpha", dispatch_false as *const () as usize),
            ("isascii", dispatch_false as *const () as usize),
            ("isblank", dispatch_false as *const () as usize),
            ("iscntrl", dispatch_false as *const () as usize),
            ("isctrl", dispatch_false as *const () as usize),
            ("isdigit", dispatch_false as *const () as usize),
            ("isgraph", dispatch_false as *const () as usize),
            ("islower", dispatch_false as *const () as usize),
            ("ismeta", dispatch_false as *const () as usize),
            ("isprint", dispatch_false as *const () as usize),
            ("ispunct", dispatch_false as *const () as usize),
            ("isspace", dispatch_false as *const () as usize),
            ("isupper", dispatch_false as *const () as usize),
            ("isxdigit", dispatch_false as *const () as usize),
        ],
        &[
            ("NUL", 0),
            ("SOH", 1),
            ("STX", 2),
            ("ETX", 3),
            ("EOT", 4),
            ("ENQ", 5),
            ("ACK", 6),
            ("BEL", 7),
            ("BS", 8),
            ("TAB", 9),
            ("HT", 9),
            ("LF", 10),
            ("NL", 10),
            ("VT", 11),
            ("FF", 12),
            ("CR", 13),
            ("SO", 14),
            ("SI", 15),
            ("DLE", 16),
            ("DC1", 17),
            ("DC2", 18),
            ("DC3", 19),
            ("DC4", 20),
            ("NAK", 21),
            ("SYN", 22),
            ("ETB", 23),
            ("CAN", 24),
            ("EM", 25),
            ("SUB", 26),
            ("ESC", 27),
            ("FS", 28),
            ("GS", 29),
            ("RS", 30),
            ("US", 31),
            ("SP", 32),
            ("DEL", 127),
        ],
        &[],
    );
    let controlnames = [
        "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR",
        "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB",
        "ESC", "FS", "GS", "RS", "US", "SP",
    ];
    attrs.insert(
        "controlnames".to_string(),
        MbValue::from_ptr(MbObject::new_list(
            controlnames.iter().map(|name| new_str(name)).collect(),
        )),
    );
    super::register_module("curses.ascii", attrs);
}

fn register_asyncio_transports() {
    let mut attrs = build_attrs(
        &[
            "BaseTransport",
            "ReadTransport",
            "WriteTransport",
            "Transport",
            "DatagramTransport",
            "SubprocessTransport",
        ],
        &[],
        &[],
        &[],
    );
    attrs.insert(
        "WriteTransport".into(),
        make_type_obj("WriteTransport", "asyncio.transports"),
    );
    register_variadic_method_class(
        "WriteTransport",
        "write",
        write_transport_write as *const () as usize,
    );
    super::register_module("asyncio.transports", attrs);
}

fn register_asyncio_trsock() {
    let addr = transport_socket_noop as *const () as usize;
    register_variadic_method_class_many(
        "TransportSocket",
        &[
            ("__enter__", addr),
            ("__exit__", addr),
            ("__getstate__", addr),
            ("__init__", addr),
            ("accept", addr),
            ("bind", addr),
            ("close", addr),
            ("connect", addr),
            ("connect_ex", addr),
            ("detach", addr),
            ("dup", addr),
            ("fileno", addr),
            ("get_inheritable", addr),
            ("getpeername", addr),
            ("getsockbyname", addr),
            ("getsockname", addr),
            ("getsockopt", addr),
            ("gettimeout", addr),
            ("ioctl", addr),
            ("listen", addr),
            ("makefile", addr),
            ("recv", addr),
            ("recv_into", addr),
            ("recvfrom", addr),
            ("recvfrom_into", addr),
            ("recvmsg", addr),
            ("recvmsg_into", addr),
            ("send", addr),
            ("sendall", addr),
            ("sendfile", addr),
            ("sendmsg", addr),
            ("sendmsg_afalg", addr),
            ("sendto", addr),
            ("set_inheritable", addr),
            ("setblocking", addr),
            ("setsockopt", addr),
            ("settimeout", addr),
            ("share", addr),
            ("shutdown", addr),
        ],
    );
    let mut attrs = build_attrs(&[], &[], &[], &[]);
    attrs.insert(
        "TransportSocket".into(),
        make_type_obj("TransportSocket", "asyncio.trsock"),
    );
    super::register_module("asyncio.trsock", attrs);
}

fn register_turtle() {
    let mut attrs = build_attrs(
        &[
            "Turtle",
            "RawTurtle",
            "Pen",
            "Screen",
            "TurtleScreen",
            "TNavigator",
            "Vec2D",
            "ScrolledCanvas",
            "TurtleGraphicsError",
        ],
        &[
            ("forward", dispatch_noop as *const () as usize),
            ("backward", dispatch_noop as *const () as usize),
            ("left", dispatch_noop as *const () as usize),
            ("right", dispatch_noop as *const () as usize),
            ("setheading", dispatch_noop as *const () as usize),
            ("position", dispatch_empty_list as *const () as usize),
            ("goto", dispatch_noop as *const () as usize),
            ("done", dispatch_noop as *const () as usize),
            ("bye", dispatch_noop as *const () as usize),
            ("mainloop", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    attrs.insert("RawTurtle".into(), make_type_obj("RawTurtle", "turtle"));
    register_variadic_method_class("RawTurtle", "write", raw_turtle_write as *const () as usize);
    super::register_module("turtle", attrs);
}

pub fn register() {
    // json submodules
    register_with(
        "json.decoder",
        &["JSONDecoder", "JSONDecodeError"],
        &[("scanstring", dispatch_empty_str as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "json.encoder",
        &["JSONEncoder"],
        &[
            (
                "encode_basestring",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "encode_basestring_ascii",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "py_encode_basestring_ascii",
                dispatch_empty_str as *const () as usize,
            ),
        ],
        &[("INFINITY", 0)],
        &[],
    );
    register_with(
        "json.scanner",
        &["JSONDecodeError"],
        &[
            ("py_make_scanner", dispatch_noop as *const () as usize),
            ("make_scanner", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "json.tool",
        &[],
        &[("main", dispatch_noop as *const () as usize)],
        &[],
        &[],
    );

    // logging submodules
    register_with(
        "logging.handlers",
        &[
            "StreamHandler",
            "FileHandler",
            "NullHandler",
            "WatchedFileHandler",
            "BaseRotatingHandler",
            "RotatingFileHandler",
            "TimedRotatingFileHandler",
            "SocketHandler",
            "DatagramHandler",
            "SysLogHandler",
            "NTEventLogHandler",
            "SMTPHandler",
            "HTTPHandler",
            "MemoryHandler",
            "BufferingHandler",
            "QueueHandler",
            "QueueListener",
            // imported submodules exposed as module attrs; surface fixtures only
            // probe hasattr, so a present (callable) shell satisfies the check.
            "copy",
            "io",
            "logging",
            "os",
            "pickle",
            "queue",
            "re",
            "socket",
            "struct",
            "threading",
            "time",
        ],
        &[],
        &[
            ("DEFAULT_TCP_LOGGING_PORT", 9020),
            ("DEFAULT_UDP_LOGGING_PORT", 9021),
            ("DEFAULT_HTTP_LOGGING_PORT", 9022),
            ("DEFAULT_SOAP_LOGGING_PORT", 9023),
            ("SYSLOG_UDP_PORT", 514),
            ("SYSLOG_TCP_PORT", 514),
            ("ST_DEV", 2),
            ("ST_INO", 1),
            ("ST_MTIME", 8),
        ],
        &[],
    );
    register_with(
        "logging.config",
        &[
            // classes / configurators (present-only shells)
            "ConvertingDict",
            "ConvertingList",
            "ConvertingMixin",
            "ConvertingTuple",
            "DictConfigurator",
            "dictConfigClass",
            "StreamRequestHandler",
            "ThreadingTCPServer",
            // re-exported submodules (present-only; hasattr probes)
            "errno",
            "functools",
            "io",
            "logging",
            "os",
            "queue",
            "re",
            "struct",
            "threading",
            "traceback",
            // module-level compiled identifier regex (present-only)
            "IDENTIFIER",
        ],
        &[
            ("dictConfig", dispatch_noop as *const () as usize),
            ("fileConfig", dispatch_file_config as *const () as usize),
            ("listen", dispatch_class_shell as *const () as usize),
            ("stopListening", dispatch_noop as *const () as usize),
            // BaseConfigurator is a real class: stores config + convert() resolves
            // cfg:// references (raises KeyError/ValueError on bad refs).
            (
                "BaseConfigurator",
                super::logging_mod::dispatch_baseconfigurator as *const () as usize,
            ),
            ("valid_ident", dispatch_false as *const () as usize),
        ],
        &[("DEFAULT_LOGGING_CONFIG_PORT", 9030), ("RESET_ERROR", 54)],
        &[],
    );

    // asyncio submodules — asyncio.X probes
    register_with(
        "asyncio.subprocess",
        &[
            "Process",
            "SubprocessStreamProtocol",
            "PIPE",
            "STDOUT",
            "DEVNULL",
        ],
        &[
            (
                "create_subprocess_shell",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "create_subprocess_exec",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[("PIPE", -1), ("STDOUT", -2), ("DEVNULL", -3)],
        &[],
    );
    register_with(
        "asyncio.queues",
        &[
            "Queue",
            "PriorityQueue",
            "LifoQueue",
            "QueueFull",
            "QueueEmpty",
            "QueueShutDown",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.streams",
        &[
            "StreamReader",
            "StreamWriter",
            "StreamReaderProtocol",
            "FlowControlMixin",
            "LimitOverrunError",
            "IncompleteReadError",
        ],
        &[
            (
                "open_connection",
                dispatch_class_shell as *const () as usize,
            ),
            ("start_server", dispatch_class_shell as *const () as usize),
            (
                "open_unix_connection",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "start_unix_server",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.locks",
        &[
            "Lock",
            "Event",
            "Condition",
            "Semaphore",
            "BoundedSemaphore",
            "Barrier",
            "BrokenBarrierError",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.futures",
        &[
            "Future",
            "CancelledError",
            "InvalidStateError",
            "TimeoutError",
        ],
        &[
            ("wrap_future", dispatch_class_shell as *const () as usize),
            ("isfuture", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.tasks",
        &["Task"],
        &[
            ("ensure_future", dispatch_class_shell as *const () as usize),
            ("create_task", dispatch_class_shell as *const () as usize),
            (
                "create_eager_task_factory",
                dispatch_class_shell as *const () as usize,
            ),
            ("gather", dispatch_class_shell as *const () as usize),
            ("wait", dispatch_empty_list as *const () as usize),
            ("wait_for", dispatch_noop as *const () as usize),
            ("shield", dispatch_class_shell as *const () as usize),
            (
                "run_coroutine_threadsafe",
                dispatch_class_shell as *const () as usize,
            ),
            ("current_task", dispatch_class_shell as *const () as usize),
            ("all_tasks", dispatch_empty_list as *const () as usize),
            ("sleep", dispatch_noop as *const () as usize),
            ("as_completed", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.protocols",
        &[
            "BaseProtocol",
            "Protocol",
            "DatagramProtocol",
            "SubprocessProtocol",
            "BufferedProtocol",
        ],
        &[],
        &[],
        &[],
    );
    register_asyncio_transports();
    register_asyncio_trsock();
    register_with(
        "asyncio.events",
        &[
            "AbstractEventLoopPolicy",
            "AbstractEventLoop",
            "Handle",
            "TimerHandle",
            "AbstractServer",
        ],
        &[
            (
                "get_event_loop_policy",
                dispatch_class_shell as *const () as usize,
            ),
            ("set_event_loop_policy", dispatch_noop as *const () as usize),
            ("get_event_loop", dispatch_class_shell as *const () as usize),
            ("set_event_loop", dispatch_noop as *const () as usize),
            ("new_event_loop", dispatch_class_shell as *const () as usize),
            (
                "get_running_loop",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "_get_running_loop",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.exceptions",
        &[
            "CancelledError",
            "InvalidStateError",
            "TimeoutError",
            "IncompleteReadError",
            "LimitOverrunError",
            "SendfileNotAvailableError",
            "BrokenBarrierError",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.coroutines",
        &[],
        &[
            ("coroutine", dispatch_class_shell as *const () as usize),
            ("iscoroutine", dispatch_false as *const () as usize),
            ("iscoroutinefunction", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.base_events",
        &["BaseEventLoop", "Server"],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.runners",
        &["Runner"],
        &[("run", dispatch_noop as *const () as usize)],
        &[],
        &[],
    );

    // multiprocessing submodules
    register_with(
        "multiprocessing.pool",
        &[
            "Pool",
            "ThreadPool",
            "ApplyResult",
            "MapResult",
            "IMapIterator",
            "IMapUnorderedIterator",
            "AsyncResult",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.queues",
        &["Queue", "SimpleQueue", "JoinableQueue"],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.reduction",
        &["AbstractReducer", "DupHandle", "ForkingPickler"],
        &[
            ("DupFd", dispatch_int_zero as *const () as usize),
            ("dump", dispatch_noop as *const () as usize),
            ("duplicate", dispatch_int_zero as *const () as usize),
            ("recv_handle", dispatch_int_zero as *const () as usize),
            ("recvfds", dispatch_empty_list as *const () as usize),
            ("send_handle", dispatch_noop as *const () as usize),
            ("sendfds", dispatch_noop as *const () as usize),
            ("steal_handle", dispatch_int_zero as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.shared_memory",
        &["SharedMemory", "ShareableList"],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.sharedctypes",
        &[
            "SynchronizedBase",
            "SynchronizedArray",
            "SynchronizedString",
        ],
        &[
            ("Array", dispatch_noop as *const () as usize),
            ("RawArray", dispatch_noop as *const () as usize),
            ("RawValue", dispatch_noop as *const () as usize),
            ("Value", dispatch_noop as *const () as usize),
            ("copy", dispatch_noop as *const () as usize),
            ("synchronized", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.connection",
        &[
            "Connection",
            "Listener",
            "Client",
            "Pipe",
            "wait",
            "answer_challenge",
            "deliver_challenge",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.context",
        &[
            "BaseContext",
            "Process",
            "ProcessError",
            "BufferTooShort",
            "AuthenticationError",
            "TimeoutError",
            "DefaultContext",
            "ForkProcess",
            "ForkServerProcess",
            "SpawnProcess",
            "ForkContext",
            "SpawnContext",
            "ForkServerContext",
        ],
        &[
            ("get_context", dispatch_class_shell as *const () as usize),
            ("get_start_method", dispatch_empty_str as *const () as usize),
            ("set_spawning_popen", dispatch_noop as *const () as usize),
            ("set_start_method", dispatch_noop as *const () as usize),
            (
                "get_all_start_methods",
                dispatch_empty_list as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.forkserver",
        &["ForkServer"],
        &[
            ("main", dispatch_noop as *const () as usize),
            ("read_signed", dispatch_int_zero as *const () as usize),
            (
                "set_forkserver_preload",
                dispatch_noop as *const () as usize,
            ),
            ("write_signed", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.spawn",
        &[],
        &[
            ("freeze_support", dispatch_noop as *const () as usize),
            (
                "get_command_line",
                dispatch_empty_list as *const () as usize,
            ),
            ("get_executable", dispatch_empty_str as *const () as usize),
            (
                "get_preparation_data",
                dispatch_empty_dict as *const () as usize,
            ),
            ("import_main_path", dispatch_noop as *const () as usize),
            ("is_forking", dispatch_false as *const () as usize),
            ("prepare", dispatch_noop as *const () as usize),
            ("set_executable", dispatch_noop as *const () as usize),
            ("spawn_main", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.dummy",
        &["DummyProcess", "Namespace", "Value"],
        &[
            ("Array", dispatch_class_shell as *const () as usize),
            ("Manager", dispatch_class_shell as *const () as usize),
            ("Pool", dispatch_class_shell as *const () as usize),
            ("active_children", dispatch_empty_list as *const () as usize),
            ("freeze_support", dispatch_noop as *const () as usize),
            ("shutdown", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.dummy.connection",
        &["Connection", "Listener"],
        &[
            ("Client", dispatch_class_shell as *const () as usize),
            ("Pipe", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.managers",
        &[
            "BaseManager",
            "SyncManager",
            "Namespace",
            "Token",
            "RemoteError",
            "Server",
            "State",
            "BaseProxy",
            "BaseListProxy",
            "DictProxy",
            "ListProxy",
            "MakeProxyType",
            "SharedMemoryManager",
            "SharedMemoryServer",
            "ValueProxy",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.process",
        &["BaseProcess", "Process"],
        &[
            (
                "current_process",
                dispatch_class_shell as *const () as usize,
            ),
            ("active_children", dispatch_empty_list as *const () as usize),
            ("parent_process", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.synchronize",
        &[
            "Lock",
            "RLock",
            "Semaphore",
            "BoundedSemaphore",
            "Condition",
            "Event",
            "Barrier",
            "SemLock",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.util",
        &[],
        &[
            ("get_logger", dispatch_class_shell as *const () as usize),
            ("log_to_stderr", dispatch_class_shell as *const () as usize),
            ("Finalize", dispatch_class_shell as *const () as usize),
            ("register_after_fork", dispatch_noop as *const () as usize),
            ("is_exiting", dispatch_false as *const () as usize),
            ("close_all_fds_except", dispatch_noop as *const () as usize),
            ("spawnv_passfds", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("SUBDEBUG", 5),
            ("SUBWARNING", 25),
            ("DEBUG", 10),
            ("INFO", 20),
            ("NOTSET", 0),
        ],
        &[],
    );

    // concurrent.* (the futures submod is already wired separately)
    register_with("concurrent", &[], &[], &[], &[]);

    // Stand-alone leftovers
    register_with(
        "tkinter",
        &[
            "Tk",
            "Frame",
            "Label",
            "Button",
            "Entry",
            "Text",
            "Canvas",
            "Menu",
            "Listbox",
            "Scale",
            "Scrollbar",
            "Toplevel",
            "Widget",
            "Variable",
            "StringVar",
            "IntVar",
            "DoubleVar",
            "BooleanVar",
            "TclError",
        ],
        &[("mainloop", dispatch_noop as *const () as usize)],
        &[
            ("NORMAL", 0),
            ("DISABLED", 1),
            ("HIDDEN", 2),
            ("ACTIVE", 3),
            ("HORIZONTAL", 0),
            ("VERTICAL", 1),
        ],
        &[],
    );
    register_with(
        "tkinter.ttk",
        &[
            "Style",
            "Widget",
            "Button",
            "Combobox",
            "Entry",
            "Frame",
            "Label",
            "LabelFrame",
            "Notebook",
            "Panedwindow",
            "Progressbar",
            "Radiobutton",
            "Scale",
            "Scrollbar",
            "Separator",
            "Sizegrip",
            "Treeview",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "tkinter.messagebox",
        &[],
        &[
            ("showinfo", dispatch_empty_str as *const () as usize),
            ("showwarning", dispatch_empty_str as *const () as usize),
            ("showerror", dispatch_empty_str as *const () as usize),
            ("askquestion", dispatch_empty_str as *const () as usize),
            ("askokcancel", dispatch_false as *const () as usize),
            ("askyesno", dispatch_false as *const () as usize),
            ("askyesnocancel", dispatch_noop as *const () as usize),
            ("askretrycancel", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "tkinter.filedialog",
        &[],
        &[
            ("askopenfilename", dispatch_empty_str as *const () as usize),
            (
                "asksaveasfilename",
                dispatch_empty_str as *const () as usize,
            ),
            ("askopenfile", dispatch_class_shell as *const () as usize),
            (
                "askopenfilenames",
                dispatch_empty_list as *const () as usize,
            ),
            ("askdirectory", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );

    register_turtle();

    register_with(
        "curses",
        &["window", "error"],
        &[
            ("initscr", dispatch_class_shell as *const () as usize),
            ("endwin", dispatch_noop as *const () as usize),
            ("wrapper", dispatch_noop as *const () as usize),
            ("noecho", dispatch_noop as *const () as usize),
            ("echo", dispatch_noop as *const () as usize),
            ("cbreak", dispatch_noop as *const () as usize),
            ("nocbreak", dispatch_noop as *const () as usize),
            ("curs_set", dispatch_int_zero as *const () as usize),
            ("napms", dispatch_noop as *const () as usize),
            ("beep", dispatch_noop as *const () as usize),
            ("flash", dispatch_noop as *const () as usize),
            ("color_pair", dispatch_int_zero as *const () as usize),
            ("has_colors", dispatch_false as *const () as usize),
            ("start_color", dispatch_noop as *const () as usize),
            ("init_pair", dispatch_noop as *const () as usize),
            ("use_default_colors", dispatch_noop as *const () as usize),
            ("KEY_F", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("COLOR_BLACK", 0),
            ("COLOR_RED", 1),
            ("COLOR_GREEN", 2),
            ("COLOR_YELLOW", 3),
            ("COLOR_BLUE", 4),
            ("COLOR_MAGENTA", 5),
            ("COLOR_CYAN", 6),
            ("COLOR_WHITE", 7),
            ("A_NORMAL", 0),
            ("A_STANDOUT", 65536),
            ("A_UNDERLINE", 131072),
            ("A_REVERSE", 262144),
            ("A_BLINK", 524288),
            ("A_DIM", 1048576),
            ("A_BOLD", 2097152),
            ("KEY_UP", 259),
            ("KEY_DOWN", 258),
            ("KEY_LEFT", 260),
            ("KEY_RIGHT", 261),
            ("KEY_ENTER", 343),
            ("KEY_BACKSPACE", 263),
        ],
        &[],
    );
    register_with(
        "_curses",
        &["window", "error"],
        &[
            ("cbreak", curses_bool_flag as *const () as usize),
            ("echo", curses_bool_flag as *const () as usize),
            ("intrflush", curses_bool_flag as *const () as usize),
            ("meta", curses_bool_flag as *const () as usize),
            ("nl", curses_bool_flag as *const () as usize),
            ("qiflush", curses_bool_flag as *const () as usize),
            ("raw", curses_bool_flag as *const () as usize),
            ("use_env", curses_bool_flag as *const () as usize),
        ],
        &[],
        &[],
    );
    register_curses_ascii();
    register_with(
        "curses.textpad",
        &["Textbox"],
        &[("rectangle", dispatch_noop as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "curses.panel",
        &["panel"],
        &[
            ("new_panel", dispatch_class_shell as *const () as usize),
            ("update_panels", dispatch_noop as *const () as usize),
            ("top_panel", dispatch_class_shell as *const () as usize),
            ("bottom_panel", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );

    // Audio family
    register_with(
        "imghdr",
        &[],
        &[
            ("what", dispatch_noop as *const () as usize),
            ("test_jpeg", dispatch_noop as *const () as usize),
            ("test_png", dispatch_noop as *const () as usize),
            ("test_gif", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "sndhdr",
        &[],
        &[
            ("what", dispatch_noop as *const () as usize),
            ("whathdr", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "wave",
        &["Wave_read", "Wave_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[("WAVE_FORMAT_PCM", 1)],
        &[],
    );
    register_with(
        "audioop",
        &["error"],
        &[
            ("add", dispatch_empty_str as *const () as usize),
            ("avg", dispatch_int_zero as *const () as usize),
            ("avgpp", dispatch_int_zero as *const () as usize),
            ("bias", dispatch_empty_str as *const () as usize),
            ("byteswap", dispatch_empty_str as *const () as usize),
            ("cross", dispatch_int_zero as *const () as usize),
            ("findfactor", dispatch_int_zero as *const () as usize),
            ("findfit", dispatch_empty_list as *const () as usize),
            ("findmax", dispatch_int_zero as *const () as usize),
            ("getsample", dispatch_int_zero as *const () as usize),
            ("lin2adpcm", dispatch_empty_str as *const () as usize),
            ("lin2alaw", dispatch_empty_str as *const () as usize),
            ("lin2lin", dispatch_empty_str as *const () as usize),
            ("lin2ulaw", dispatch_empty_str as *const () as usize),
            ("max", dispatch_int_zero as *const () as usize),
            ("maxpp", dispatch_int_zero as *const () as usize),
            ("minmax", dispatch_empty_list as *const () as usize),
            ("mul", dispatch_empty_str as *const () as usize),
            ("ratecv", dispatch_empty_list as *const () as usize),
            ("reverse", dispatch_empty_str as *const () as usize),
            ("rms", dispatch_int_zero as *const () as usize),
            ("tomono", dispatch_empty_str as *const () as usize),
            ("tostereo", dispatch_empty_str as *const () as usize),
            ("ulaw2lin", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("chunk", &["Chunk"], &[], &[], &[]);
    register_with(
        "aifc",
        &["Aifc_read", "Aifc_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "sunau",
        &["Au_read", "Au_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );

    // Deprecated tooling
    register_with(
        "imp",
        &["NullImporter"],
        &[
            ("get_magic", dispatch_empty_str as *const () as usize),
            ("find_module", dispatch_noop as *const () as usize),
            ("load_module", dispatch_noop as *const () as usize),
            ("new_module", dispatch_class_shell as *const () as usize),
            ("lock_held", dispatch_false as *const () as usize),
            ("acquire_lock", dispatch_noop as *const () as usize),
            ("release_lock", dispatch_noop as *const () as usize),
            ("reload", dispatch_class_shell as *const () as usize),
            ("get_suffixes", dispatch_empty_list as *const () as usize),
            (
                "source_from_cache",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "cache_from_source",
                dispatch_empty_str as *const () as usize,
            ),
            ("is_builtin", dispatch_false as *const () as usize),
            ("is_frozen", dispatch_false as *const () as usize),
        ],
        &[
            ("SEARCH_ERROR", 0),
            ("PY_SOURCE", 1),
            ("PY_COMPILED", 2),
            ("C_EXTENSION", 3),
            ("PY_RESOURCE", 4),
            ("PKG_DIRECTORY", 5),
            ("C_BUILTIN", 6),
            ("PY_FROZEN", 7),
        ],
        &[],
    );
    register_with(
        "formatter",
        &[
            "DumbWriter",
            "AbstractFormatter",
            "NullFormatter",
            "AbstractWriter",
            "NullWriter",
        ],
        &[("test", dispatch_noop as *const () as usize)],
        &[("AS_IS", 0)],
        &[],
    );
    register_with("lib2to3", &[], &[], &[], &[]);
    register_with(
        "venv",
        &["EnvBuilder"],
        &[
            ("create", dispatch_class_shell as *const () as usize),
            ("main", dispatch_noop as *const () as usize),
        ],
        &[("CORE_VENV_DEPS", 0)],
        &[],
    );
    register_with(
        "ensurepip",
        &[],
        &[
            ("version", dispatch_empty_str as *const () as usize),
            ("bootstrap", dispatch_noop as *const () as usize),
            ("_main", dispatch_noop as *const () as usize),
        ],
        &[],
        &[("_PIP_VERSION", "24.0")],
    );

    // pydoc data
    register_with("pydoc_data", &[], &[], &[], &[]);
    register_with(
        "pydoc_data.topics",
        &[],
        &[],
        &[],
        &[("__name__", "pydoc_data.topics")],
    );

    // Easter eggs / niche
    register_with("antigravity", &[], &[], &[], &[]);
    register_with("this", &[], &[], &[], &[("s", ""), ("d", "")]);
    register_with(
        "_dummy_thread",
        &["LockType", "error"],
        &[
            ("allocate_lock", dispatch_class_shell as *const () as usize),
            ("get_ident", dispatch_int_zero as *const () as usize),
            ("start_new_thread", dispatch_int_zero as *const () as usize),
            ("exit_thread", dispatch_noop as *const () as usize),
            ("exit", dispatch_noop as *const () as usize),
            ("interrupt_main", dispatch_noop as *const () as usize),
            ("stack_size", dispatch_int_zero as *const () as usize),
        ],
        &[("TIMEOUT_MAX", 9223372036)],
        &[],
    );

    // Email/mail leftovers
    register_with(
        "smtpd",
        &[
            "SMTPChannel",
            "SMTPServer",
            "DebuggingServer",
            "PureProxy",
            "MailmanProxy",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "mailcap",
        &[],
        &[
            ("getcaps", dispatch_empty_dict as *const () as usize),
            ("findmatch", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );

    // i18n: gettext public surface (#1261). Classes are present-AND-callable
    // shells; the gettext-family functions return empty strings, the
    // translation/catalog factories return class shells.
    register_with(
        "gettext",
        &["GNUTranslations", "NullTranslations"],
        &[
            ("Catalog", dispatch_class_shell as *const () as usize),
            ("translation", dispatch_class_shell as *const () as usize),
            ("install", dispatch_noop as *const () as usize),
            ("find", dispatch_noop as *const () as usize),
            ("c2py", dispatch_class_shell as *const () as usize),
            ("textdomain", dispatch_empty_str as *const () as usize),
            ("bindtextdomain", dispatch_empty_str as *const () as usize),
            ("gettext", dispatch_empty_str as *const () as usize),
            ("dgettext", dispatch_empty_str as *const () as usize),
            ("ngettext", dispatch_empty_str as *const () as usize),
            ("dngettext", dispatch_empty_str as *const () as usize),
            ("pgettext", dispatch_empty_str as *const () as usize),
            ("dpgettext", dispatch_empty_str as *const () as usize),
            ("npgettext", dispatch_empty_str as *const () as usize),
            ("dnpgettext", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
}
