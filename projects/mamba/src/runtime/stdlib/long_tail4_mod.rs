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

/// #1040 follow-up: `dispatch_class_shell` is kept as a MARKER address only.
/// Every call site below still writes `dispatch_class_shell as *const () as
/// usize` (for `dispatchers` tuples) or a bare class name (for the `classes`
/// list) exactly as before -- `register_with` (below) detects the marker
/// address / every `classes` entry and transparently substitutes a genuinely
/// distinct `SHELL_POOL` slot before the value is ever inserted into a
/// module's attrs. This file also has THREE hand-rolled registration
/// functions (`register_codec_module`, `register_punycode_module`,
/// `register_xml_sax_package`) that build attrs directly instead of going
/// through `register_with` -- those draw their own fresh `SHELL_POOL` slots
/// per name too, right at their call sites.
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
const SHELL_POOL_SIZE: usize = 320;
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
    shell_315, shell_316, shell_317, shell_318, shell_319,
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
unsafe extern "C" fn dispatch_empty_bytes(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
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

unsafe extern "C" fn dispatch_incremental_decoder_buffer_decode(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(input) = args.first().copied() else {
        return raise_type_error("_buffer_decode() missing required data argument");
    };
    if !is_bytes_like(input) {
        return raise_type_error("_buffer_decode() argument 1 must be bytes-like");
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(String::new())),
        MbValue::from_int(0),
    ]))
}

unsafe extern "C" fn dispatch_decode_generalized_number(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(extended) = args.first().copied() else {
        return raise_type_error("decode_generalized_number() missing required extended argument");
    };
    if !is_bytes_like(extended) {
        return raise_type_error("decode_generalized_number() argument 1 must be bytes-like");
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(0),
        MbValue::from_int(0),
    ]))
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

fn is_bytes_like(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
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
            if name == "IncrementalDecoder" {
                let addr = dispatch_incremental_decoder_buffer_decode as *const () as usize;
                map.insert("_buffer_decode".to_string(), MbValue::from_func(addr));
                super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                    s.borrow_mut().insert(addr as u64);
                });
            }
        }
    }
    MbValue::from_ptr(obj)
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

unsafe extern "C" fn dispatch_io_stream_constructor_body(
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

// #1040 follow-up: `_io.BufferedReader`, `_io.BufferedWriter`, and
// `_io.TextIOWrapper` used to all share ONE function address
// (`dispatch_io_stream_constructor`, now `..._body` above) -- the exact
// #962/#954 FUNC_NAMES/NATIVE_FUNC_ADDRS collision, just via a real
// validating constructor instead of a trivial shell. Each class name below
// gets its own thin wrapper with a distinct `stringify!`-derived fingerprint
// (same ICF-defeating trick as `SHELL_POOL`) before delegating to the shared
// validation body, so behavior is unchanged but each address is unique.
unsafe extern "C" fn dispatch_io_stream_constructor_buffered_reader(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
        module_path!(),
        "::dispatch_io_stream_constructor_buffered_reader"
    )));
    dispatch_io_stream_constructor_body(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_io_stream_constructor_buffered_writer(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
        module_path!(),
        "::dispatch_io_stream_constructor_buffered_writer"
    )));
    dispatch_io_stream_constructor_body(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_io_stream_constructor_text_io_wrapper(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
        module_path!(),
        "::dispatch_io_stream_constructor_text_io_wrapper"
    )));
    dispatch_io_stream_constructor_body(args_ptr, nargs)
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
    // #1040: every `classes` entry used to share ONE `dispatch_class_shell`
    // address across the WHOLE file, and every `dispatchers` tuple using the
    // `dispatch_class_shell` marker address shared that same one address too
    // -- both are the exact FUNC_NAMES/NATIVE_FUNC_ADDRS address-collision
    // bug from #962/#954. Give each a fresh, genuinely distinct SHELL_POOL
    // slot instead; `dispatch_class_shell` itself is never handed out
    // anymore, only used as a marker to detect (and replace) the legacy
    // shared address in `dispatchers` tuples.
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

unsafe extern "C" fn codec_encode(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args);
    let Some(input) = items.first().copied() else {
        return raise_type_error("encode() missing required input argument");
    };
    if !is_str(input) {
        return raise_type_error("encode() argument 1 must be str");
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
        MbValue::from_int(0),
    ]))
}

unsafe extern "C" fn codec_decode(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args);
    let Some(input) = items.first().copied() else {
        return raise_type_error("decode() missing required input argument");
    };
    if !is_bytes_like(input) {
        return raise_type_error("decode() argument 1 must be bytes-like");
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(String::new())),
        MbValue::from_int(0),
    ]))
}

unsafe extern "C" fn incremental_encoder_encode(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args);
    let Some(input) = items.first().copied() else {
        return raise_type_error("encode() missing required input argument");
    };
    if !is_str(input) {
        return raise_type_error("encode() argument 1 must be str");
    }
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

unsafe extern "C" fn incremental_decoder_decode(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args);
    let Some(input) = items.first().copied() else {
        return raise_type_error("decode() missing required input argument");
    };
    if !is_bytes_like(input) {
        return raise_type_error("decode() argument 1 must be bytes-like");
    }
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

fn register_codec_classes() {
    let codec_encode_addr = codec_encode as usize;
    let codec_decode_addr = codec_decode as usize;
    let incremental_encoder_addr = incremental_encoder_encode as usize;
    let incremental_decoder_addr = incremental_decoder_decode as usize;
    for addr in [
        codec_encode_addr,
        codec_decode_addr,
        incremental_encoder_addr,
        incremental_decoder_addr,
    ] {
        super::super::module::register_variadic_func(addr as u64);
    }

    let mut codec_methods = HashMap::new();
    codec_methods.insert("encode".to_string(), MbValue::from_func(codec_encode_addr));
    codec_methods.insert("decode".to_string(), MbValue::from_func(codec_decode_addr));
    super::super::class::mb_class_register("Codec", vec!["object".to_string()], codec_methods);

    let mut encoder_methods = HashMap::new();
    encoder_methods.insert(
        "encode".to_string(),
        MbValue::from_func(incremental_encoder_addr),
    );
    super::super::class::mb_class_register(
        "IncrementalEncoder",
        vec!["object".to_string()],
        encoder_methods,
    );

    let mut decoder_methods = HashMap::new();
    decoder_methods.insert(
        "decode".to_string(),
        MbValue::from_func(incremental_decoder_addr),
    );
    super::super::class::mb_class_register(
        "IncrementalDecoder",
        vec!["object".to_string()],
        decoder_methods,
    );
}

fn register_codec_module(name: &str) {
    // encodings.<codec> follows the same shape as encodings.utf_8 (PR #2373):
    // exposes `getregentry()` plus codec class objects. The class objects are
    // real enough for `object.__new__(Codec)` type-wall fixtures to call the
    // registered methods instead of passing through a setup-only TypeError.
    let mut attrs = HashMap::new();
    for class_name in &[
        "Codec",
        "IncrementalEncoder",
        "IncrementalDecoder",
        "StreamWriter",
        "StreamReader",
    ] {
        attrs.insert((*class_name).to_string(), make_type_obj(class_name, name));
    }
    // #1040: `getregentry` used to share `dispatch_class_shell`'s address
    // with every OTHER class-shell name across the whole file (this function
    // alone is called 81 times, once per `encodings.*` codec module) --
    // give it its own fresh SHELL_POOL slot per call instead.
    let getregentry = shell_addr(next_shell_slot());
    let encode = dispatch_empty_bytes as *const () as usize;
    let decode = dispatch_empty_str as *const () as usize;
    attrs.insert("getregentry".to_string(), MbValue::from_func(getregentry));
    attrs.insert("encode".to_string(), MbValue::from_func(encode));
    attrs.insert("decode".to_string(), MbValue::from_func(decode));
    register_addrs(&[getregentry, encode, decode]);
    super::register_module(name, attrs);
}

fn register_punycode_module() {
    let name = "encodings.punycode";
    let mut attrs = HashMap::new();
    for class_name in &[
        "Codec",
        "IncrementalEncoder",
        "IncrementalDecoder",
        "StreamWriter",
        "StreamReader",
    ] {
        attrs.insert((*class_name).to_string(), make_type_obj(class_name, name));
    }

    // #1040: `getregentry` plus all 11 `function_name` loop entries below
    // used to share ONE address (`dispatch_class_shell`'s) -- a 12-way
    // FUNC_NAMES/NATIVE_FUNC_ADDRS collision entirely within this one
    // function. Give each its own fresh SHELL_POOL slot instead.
    let getregentry = shell_addr(next_shell_slot());
    let encode = dispatch_empty_bytes as *const () as usize;
    let decode = dispatch_empty_str as *const () as usize;
    let decode_generalized_number = dispatch_decode_generalized_number as *const () as usize;
    attrs.insert("getregentry".to_string(), MbValue::from_func(getregentry));
    attrs.insert("encode".to_string(), MbValue::from_func(encode));
    attrs.insert("decode".to_string(), MbValue::from_func(decode));
    attrs.insert(
        "decode_generalized_number".to_string(),
        MbValue::from_func(decode_generalized_number),
    );
    let mut addrs = vec![getregentry, encode, decode, decode_generalized_number];
    for function_name in &[
        "adapt",
        "generate_generalized_integer",
        "generate_integers",
        "insertion_sort",
        "insertion_unsort",
        "punycode_decode",
        "punycode_encode",
        "selective_find",
        "selective_len",
        "segregate",
        "T",
    ] {
        let f = shell_addr(next_shell_slot());
        addrs.push(f);
        attrs.insert((*function_name).to_string(), MbValue::from_func(f));
    }
    register_addrs(&addrs);
    super::register_module(name, attrs);
}

pub fn register() {
    NEXT_SHELL_SLOT.with(|c| c.set(0));

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
    // #1040: all 8 classes + 3 functions below used to share ONE address
    // (`dispatch_class_shell`'s) -- an 11-way FUNC_NAMES/NATIVE_FUNC_ADDRS
    // collision entirely within this one function. Give each its own fresh
    // SHELL_POOL slot instead.
    let mut attrs = HashMap::new();
    let mut addrs = Vec::new();
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
        let f = shell_addr(next_shell_slot());
        addrs.push(f);
        attrs.insert((*cn).into(), MbValue::from_func(f));
    }
    for fn_name in &["make_parser", "parse", "parseString"] {
        let f = shell_addr(next_shell_slot());
        addrs.push(f);
        attrs.insert((*fn_name).into(), MbValue::from_func(f));
    }
    attrs.insert(
        "default_parser_list".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    register_addrs(&addrs);
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
    register_codec_classes();
    for name in &[
        "encodings.ascii",
        "encodings.cp037",
        "encodings.cp1006",
        "encodings.cp1026",
        "encodings.cp1125",
        "encodings.cp1140",
        "encodings.cp1250",
        "encodings.cp1251",
        "encodings.cp1252",
        "encodings.cp1253",
        "encodings.cp1254",
        "encodings.cp1255",
        "encodings.cp1256",
        "encodings.cp1257",
        "encodings.cp1258",
        "encodings.cp273",
        "encodings.cp424",
        "encodings.cp437",
        "encodings.cp500",
        "encodings.cp720",
        "encodings.cp737",
        "encodings.cp775",
        "encodings.cp850",
        "encodings.cp852",
        "encodings.cp855",
        "encodings.cp856",
        "encodings.cp857",
        "encodings.cp858",
        "encodings.cp860",
        "encodings.cp861",
        "encodings.cp862",
        "encodings.cp863",
        "encodings.cp864",
        "encodings.cp865",
        "encodings.cp866",
        "encodings.cp869",
        "encodings.cp874",
        "encodings.cp875",
        "encodings.hp_roman8",
        "encodings.iso8859_1",
        "encodings.iso8859_10",
        "encodings.iso8859_11",
        "encodings.iso8859_13",
        "encodings.iso8859_14",
        "encodings.iso8859_15",
        "encodings.iso8859_16",
        "encodings.iso8859_2",
        "encodings.iso8859_3",
        "encodings.iso8859_4",
        "encodings.iso8859_5",
        "encodings.iso8859_6",
        "encodings.iso8859_7",
        "encodings.iso8859_8",
        "encodings.iso8859_9",
        "encodings.koi8_r",
        "encodings.koi8_t",
        "encodings.koi8_u",
        "encodings.kz1048",
        "encodings.latin_1",
        "encodings.mac_arabic",
        "encodings.mac_croatian",
        "encodings.mac_cyrillic",
        "encodings.mac_farsi",
        "encodings.mac_greek",
        "encodings.mac_iceland",
        "encodings.mac_latin2",
        "encodings.mac_roman",
        "encodings.mac_romanian",
        "encodings.mac_turkish",
        "encodings.mbcs",
        "encodings.oem",
        "encodings.palmos",
        "encodings.ptcp154",
        "encodings.tis_620",
        "encodings.utf_16",
        "encodings.utf_16_be",
        "encodings.utf_16_le",
        "encodings.utf_32_be",
        "encodings.utf_32_le",
        "encodings.utf_7",
        "encodings.utf_8",
    ] {
        register_codec_module(name);
    }
    register_punycode_module();
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
                dispatch_io_stream_constructor_buffered_reader as *const () as usize,
            ),
            (
                "BufferedWriter",
                dispatch_io_stream_constructor_buffered_writer as *const () as usize,
            ),
            ("BufferedRWPair", dispatch_io_rw_pair as *const () as usize),
            (
                "TextIOWrapper",
                dispatch_io_stream_constructor_text_io_wrapper as *const () as usize,
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
