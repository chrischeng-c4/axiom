use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// Long-tail stub batch 3 for Mamba (#1261).
///
/// Covers the remaining empty-NoneType holes in the stdlib surface:
/// distutils family (19), ctypes (3), html.entities, xml dotted
/// submodules, zoneinfo, unittest dotted submodules, importlib dotted
/// submodules, collections.abc, email dotted submodules, and the
/// `_*` internal helper modules that legacy probe code touches.
use std::collections::HashMap;

/// #1040 follow-up: `dispatch_class_shell` is kept as a MARKER address only.
/// Every call site below still writes `dispatch_class_shell as *const () as
/// usize` (for `dispatchers` tuples) or a bare class name (for the `classes`
/// list) exactly as before -- `register_with` (below) detects the marker
/// address / every `classes` entry and transparently substitutes a genuinely
/// distinct `SHELL_POOL` slot before the value is ever inserted into a
/// module's attrs. This file funnels nearly every `dispatch_class_shell` use
/// (all 66 `register_with` calls) through that one chokepoint, so
/// centralizing the fix there covers the file without editing the individual
/// call sites.
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
const SHELL_POOL_SIZE: usize = 460;
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
    shell_339, shell_340, shell_341, shell_342, shell_343, shell_344, shell_345, shell_346,
    shell_347, shell_348, shell_349, shell_350, shell_351, shell_352, shell_353, shell_354,
    shell_355, shell_356, shell_357, shell_358, shell_359, shell_360, shell_361, shell_362,
    shell_363, shell_364, shell_365, shell_366, shell_367, shell_368, shell_369, shell_370,
    shell_371, shell_372, shell_373, shell_374, shell_375, shell_376, shell_377, shell_378,
    shell_379, shell_380, shell_381, shell_382, shell_383, shell_384, shell_385, shell_386,
    shell_387, shell_388, shell_389, shell_390, shell_391, shell_392, shell_393, shell_394,
    shell_395, shell_396, shell_397, shell_398, shell_399, shell_400, shell_401, shell_402,
    shell_403, shell_404, shell_405, shell_406, shell_407, shell_408, shell_409, shell_410,
    shell_411, shell_412, shell_413, shell_414, shell_415, shell_416, shell_417, shell_418,
    shell_419, shell_420, shell_421, shell_422, shell_423, shell_424, shell_425, shell_426,
    shell_427, shell_428, shell_429, shell_430, shell_431, shell_432, shell_433, shell_434,
    shell_435, shell_436, shell_437, shell_438, shell_439, shell_440, shell_441, shell_442,
    shell_443, shell_444, shell_445, shell_446, shell_447, shell_448, shell_449, shell_450,
    shell_451, shell_452, shell_453, shell_454, shell_455, shell_456, shell_457, shell_458,
    shell_459,
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

unsafe extern "C" fn dispatch_weakrefset_weak_set(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs == 0 {
        return super::weakref_mod::mb_weakref_weak_set_from(MbValue::none());
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    super::weakref_mod::mb_weakref_weak_set_from(a.first().copied().unwrap_or_else(MbValue::none))
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

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_str_or_bytes_path(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_) | ObjData::Bytes(_)) })
}

fn is_str(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
}

fn is_str_or_bytes_like(v: MbValue) -> bool {
    v.as_ptr().is_some_and(|p| unsafe {
        matches!(
            (*p).data,
            ObjData::Str(_) | ObjData::Bytes(_) | ObjData::ByteArray(_)
        )
    })
}

fn is_tuple(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Tuple(_)) })
}

unsafe extern "C" fn dispatch_dbm_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.first().copied().unwrap_or_else(MbValue::none);
    if !is_str_or_bytes_path(filename) {
        return raise_type_error("dbm filename must be str or bytes path");
    }
    dispatch_empty_dict(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_importlib_file_finder(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let path = a.first().copied().unwrap_or_else(MbValue::none);
    if !is_str(path) {
        return raise_type_error("FileFinder path must be str");
    }
    dispatch_empty_dict(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_importlib_cache_from_source(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let path = a.first().copied().unwrap_or_else(MbValue::none);
    if !is_str(path) {
        return raise_type_error("cache_from_source path must be str");
    }
    dispatch_empty_str(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_sre_parse_parse_template(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(source) = args.first().copied() else {
        return raise_type_error("parse_template() missing required argument: 'source'");
    };
    if !is_str_or_bytes_like(source) {
        return raise_type_error("parse_template() argument 'source' must be str or bytes-like");
    }
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

unsafe extern "C" fn multibyte_decoder_setstate(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    let state = items.first().copied().unwrap_or_else(MbValue::none);
    if !is_tuple(state) {
        return raise_type_error("_multibytecodec.MultibyteIncrementalDecoder state must be tuple");
    }
    MbValue::none()
}

unsafe extern "C" fn package_metadata_get(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    if let Some(value) = items.first().copied() {
        if extract_str(value).is_none() {
            return raise_type_error("PackageMetadata.get() argument 'name' must be str");
        }
    }
    items.get(1).copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn package_metadata_get_all(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    if let Some(value) = items.first().copied() {
        if extract_str(value).is_none() {
            return raise_type_error("PackageMetadata.get_all() argument 'name' must be str");
        }
    }
    items.get(1).copied().unwrap_or_else(MbValue::none)
}

fn register_variadic_method_class(class_name: &str, methods: &[(&str, usize)]) {
    let mut map = HashMap::new();
    for (name, addr) in methods {
        super::super::module::register_variadic_func(*addr as u64);
        map.insert((*name).to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], map);
}

// importlib.util.find_spec(name) -> spec | None. Routes to the real
// module-registry lookup so a missing module yields None (not an empty
// shell dict), matching CPython's "find_spec returns None when absent".
unsafe extern "C" fn dispatch_importlib_find_spec(a: *const MbValue, n: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(a, n) };
    super::importlib_mod::mb_importlib_find_spec(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
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
    // address across the WHOLE file (400+ names), and every `dispatchers`
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
    super::register_module(name, attrs);
}

fn make_type_obj(name: &str, module: &str) -> MbValue {
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            map.insert(
                "__qualname__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            map.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str(module.to_string())),
            );
        }
    }
    MbValue::from_ptr(obj)
}

fn register_type_module(name: &str, classes: &[&str]) {
    let mut attrs = HashMap::new();
    for cn in classes {
        attrs.insert((*cn).into(), make_type_obj(*cn, name));
    }
    super::register_module(name, attrs);
}

pub fn register() {
    NEXT_SHELL_SLOT.with(|c| c.set(0));

    register_distutils();
    register_html_entities();
    register_xml_subs();
    register_zoneinfo();
    register_unittest_subs();
    register_importlib_subs();
    register_sre_parse();
    register_collections_abc();
    register_email_subs();
    register_internals();
}

fn register_distutils() {
    // root + standard submodules
    register_with(
        "distutils",
        &[],
        &[],
        &[("__version__", 0)],
        &[("__version__", "3.12.0")],
    );
    register_with(
        "distutils.core",
        &["Distribution", "Command", "Extension", "DEBUG"],
        &[
            ("setup", dispatch_class_shell as *const () as usize),
            ("run_setup", dispatch_class_shell as *const () as usize),
        ],
        &[("DEBUG", 0)],
        &[],
    );
    register_with("distutils.cmd", &["Command"], &[], &[], &[]);
    register_with("distutils.command", &[], &[], &[], &[]);
    register_with(
        "distutils.command.build_py",
        &["build_py", "build_py_2to3"],
        &[],
        &[],
        &[],
    );
    register_with(
        "distutils.command.check",
        &["SilentReporter", "check"],
        &[],
        &[],
        &[],
    );
    register_with("distutils.command.config", &["config"], &[], &[], &[]);
    register_with(
        "distutils.errors",
        &[
            "DistutilsError",
            "DistutilsModuleError",
            "DistutilsClassError",
            "DistutilsGetoptError",
            "DistutilsArgError",
            "DistutilsFileError",
            "DistutilsOptionError",
            "DistutilsSetupError",
            "DistutilsPlatformError",
            "DistutilsExecError",
            "DistutilsInternalError",
            "DistutilsTemplateError",
            "DistutilsByteCompileError",
            "CCompilerError",
            "PreprocessError",
            "CompileError",
            "LibError",
            "LinkError",
            "UnknownFileError",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "distutils.util",
        &[],
        &[
            ("get_platform", dispatch_empty_str as *const () as usize),
            ("convert_path", dispatch_empty_str as *const () as usize),
            ("change_root", dispatch_empty_str as *const () as usize),
            ("check_environ", dispatch_noop as *const () as usize),
            ("subst_vars", dispatch_empty_str as *const () as usize),
            ("split_quoted", dispatch_empty_list as *const () as usize),
            ("execute", dispatch_noop as *const () as usize),
            ("strtobool", dispatch_int_zero as *const () as usize),
            ("byte_compile", dispatch_noop as *const () as usize),
            ("rfc822_escape", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.dir_util",
        &[],
        &[
            ("mkpath", dispatch_empty_list as *const () as usize),
            ("create_tree", dispatch_noop as *const () as usize),
            ("copy_tree", dispatch_empty_list as *const () as usize),
            ("remove_tree", dispatch_noop as *const () as usize),
            ("ensure_relative", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.file_util",
        &[],
        &[
            ("copy_file", dispatch_empty_list as *const () as usize),
            ("move_file", dispatch_empty_str as *const () as usize),
            ("write_file", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.filelist",
        &["FileList"],
        &[
            ("findall", dispatch_empty_list as *const () as usize),
            ("glob_to_re", dispatch_empty_str as *const () as usize),
            (
                "translate_pattern",
                dispatch_empty_str as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.fancy_getopt",
        &["FancyGetopt", "OptionDummy"],
        &[
            ("fancy_getopt", dispatch_empty_dict as *const () as usize),
            (
                "translate_longopt",
                dispatch_empty_str as *const () as usize,
            ),
            ("wrap_text", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.archive_util",
        &[],
        &[
            ("make_archive", dispatch_empty_str as *const () as usize),
            ("make_tarball", dispatch_empty_str as *const () as usize),
            ("make_zipfile", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.sysconfig",
        &[],
        &[
            (
                "get_python_version",
                dispatch_empty_str as *const () as usize,
            ),
            ("get_python_inc", dispatch_empty_str as *const () as usize),
            ("get_python_lib", dispatch_empty_str as *const () as usize),
            ("get_config_vars", dispatch_empty_dict as *const () as usize),
            ("get_config_var", dispatch_noop as *const () as usize),
            (
                "get_config_h_filename",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "get_makefile_filename",
                dispatch_empty_str as *const () as usize,
            ),
            ("parse_config_h", dispatch_empty_dict as *const () as usize),
            ("parse_makefile", dispatch_empty_dict as *const () as usize),
            ("customize_compiler", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.log",
        &["Log"],
        &[
            ("debug", dispatch_noop as *const () as usize),
            ("info", dispatch_noop as *const () as usize),
            ("warn", dispatch_noop as *const () as usize),
            ("error", dispatch_noop as *const () as usize),
            ("fatal", dispatch_noop as *const () as usize),
            ("log", dispatch_noop as *const () as usize),
            ("set_threshold", dispatch_int_zero as *const () as usize),
            ("set_verbosity", dispatch_noop as *const () as usize),
        ],
        &[
            ("DEBUG", 1),
            ("INFO", 2),
            ("WARN", 3),
            ("ERROR", 4),
            ("FATAL", 5),
        ],
        &[],
    );
    register_with(
        "distutils.spawn",
        &[],
        &[
            ("spawn", dispatch_noop as *const () as usize),
            ("find_executable", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.version",
        &["Version", "StrictVersion", "LooseVersion"],
        &[],
        &[],
        &[],
    );
    register_with(
        "distutils.dep_util",
        &[],
        &[
            ("newer", dispatch_false as *const () as usize),
            ("newer_pairwise", dispatch_empty_list as *const () as usize),
            ("newer_group", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "distutils.dist",
        &["Distribution", "DistributionMetadata"],
        &[(
            "fix_help_options",
            dispatch_empty_list as *const () as usize,
        )],
        &[],
        &[],
    );
    register_with(
        "distutils.extension",
        &["Extension", "read_setup_file"],
        &[],
        &[],
        &[],
    );
    register_with(
        "distutils.ccompiler",
        &["CCompiler"],
        &[
            (
                "get_default_compiler",
                dispatch_empty_str as *const () as usize,
            ),
            ("new_compiler", dispatch_class_shell as *const () as usize),
            ("show_compilers", dispatch_noop as *const () as usize),
            ("gen_lib_options", dispatch_empty_list as *const () as usize),
            (
                "gen_preprocess_options",
                dispatch_empty_list as *const () as usize,
            ),
            ("get_versions", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("distutils.unixccompiler", &["UnixCCompiler"], &[], &[], &[]);
    register_with(
        "distutils.msvccompiler",
        &[
            "MSVCCompiler",
            "get_build_version",
            "get_build_architecture",
        ],
        &[],
        &[],
        &[],
    );
}

fn register_html_entities() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "name2codepoint".into(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    attrs.insert("html5".into(), MbValue::from_ptr(MbObject::new_dict()));
    attrs.insert(
        "codepoint2name".into(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    attrs.insert("entitydefs".into(), MbValue::from_ptr(MbObject::new_dict()));
    super::register_module("html.entities", attrs);
}

fn register_xml_subs() {
    register_with(
        "xml.dom",
        &[
            "DOMException",
            "DomstringSizeErr",
            "HierarchyRequestErr",
            "IndexSizeErr",
            "InuseAttributeErr",
            "InvalidAccessErr",
            "InvalidCharacterErr",
            "InvalidModificationErr",
            "InvalidStateErr",
            "NamespaceErr",
            "NoDataAllowedErr",
            "NoModificationAllowedErr",
            "NotFoundErr",
            "NotSupportedErr",
            "SyntaxErr",
            "TypeMismatchErr",
            "UnspecifiedEventTypeErr",
            "ValidationErr",
            "WrongDocumentErr",
            "Node",
            "NodeList",
            "Document",
            "Element",
            "Attr",
            "Text",
            "Comment",
            "CDATASection",
            "ProcessingInstruction",
            "DocumentFragment",
            "DocumentType",
            "EmptyNodeList",
            "UserDataHandler",
        ],
        &[
            (
                "getDOMImplementation",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "registerDOMImplementation",
                dispatch_noop as *const () as usize,
            ),
        ],
        &[
            ("INDEX_SIZE_ERR", 1),
            ("DOMSTRING_SIZE_ERR", 2),
            ("HIERARCHY_REQUEST_ERR", 3),
            ("WRONG_DOCUMENT_ERR", 4),
            ("INVALID_CHARACTER_ERR", 5),
            ("NO_DATA_ALLOWED_ERR", 6),
            ("NO_MODIFICATION_ALLOWED_ERR", 7),
            ("NOT_FOUND_ERR", 8),
            ("NOT_SUPPORTED_ERR", 9),
            ("INUSE_ATTRIBUTE_ERR", 10),
            ("INVALID_STATE_ERR", 11),
            ("SYNTAX_ERR", 12),
            ("INVALID_MODIFICATION_ERR", 13),
            ("NAMESPACE_ERR", 14),
            ("INVALID_ACCESS_ERR", 15),
            ("VALIDATION_ERR", 16),
        ],
        &[
            ("XML_NAMESPACE", "http://www.w3.org/XML/1998/namespace"),
            ("XMLNS_NAMESPACE", "http://www.w3.org/2000/xmlns/"),
            ("XHTML_NAMESPACE", "http://www.w3.org/1999/xhtml"),
            ("EMPTY_NAMESPACE", ""),
            ("EMPTY_PREFIX", ""),
        ],
    );
    // xml.dom.domreg is a real submodule in CPython 3.12. Registering it as a
    // dotted module makes `import xml.dom.domreg` resolve and (via
    // propagate_submodule_to_parents) wires `domreg` as an attribute on the
    // `xml.dom` parent package, so `hasattr(xml.dom, "domreg")` is True.
    register_with(
        "xml.dom.domreg",
        &[],
        &[
            (
                "getDOMImplementation",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "registerDOMImplementation",
                dispatch_noop as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "xml.dom.minidom",
        &[
            "Node",
            "Document",
            "Element",
            "Attr",
            "Text",
            "Comment",
            "CDATASection",
            "ProcessingInstruction",
            "DocumentFragment",
            "DocumentType",
            "DOMImplementation",
            "AttributeList",
            "CharacterData",
            "Childless",
            "DOMImplementationLS",
            "DocumentLS",
            "ElementInfo",
            "EmptyNodeList",
            "Entity",
            "Identified",
            "NamedNodeMap",
            "NodeList",
            "Notation",
            "ReadOnlySequentialNamedNodeMap",
            "TypeInfo",
        ],
        &[
            ("parse", dispatch_class_shell as *const () as usize),
            ("parseString", dispatch_class_shell as *const () as usize),
            (
                "getDOMImplementation",
                dispatch_class_shell as *const () as usize,
            ),
            ("defproperty", dispatch_noop as *const () as usize),
        ],
        &[],
        &[
            ("XMLNS_NAMESPACE", "http://www.w3.org/2000/xmlns/"),
            ("EMPTY_NAMESPACE", ""),
            ("EMPTY_PREFIX", ""),
            ("StringTypes", ""),
            ("domreg", ""),
            ("io", ""),
            ("xml", ""),
        ],
    );
    register_with(
        "xml.dom.pulldom",
        &["PullDOM", "DOMEventStream", "SAX2DOM", "ErrorHandler"],
        &[
            ("parse", dispatch_class_shell as *const () as usize),
            ("parseString", dispatch_class_shell as *const () as usize),
        ],
        &[
            ("START_ELEMENT", 1),
            ("END_ELEMENT", 2),
            ("COMMENT", 3),
            ("START_DOCUMENT", 4),
            ("END_DOCUMENT", 5),
            ("PROCESSING_INSTRUCTION", 6),
            ("IGNORABLE_WHITESPACE", 7),
            ("CHARACTERS", 8),
        ],
        &[],
    );
    // xml.parsers / xml.parsers.expat / pyexpat: registered by
    // xml_mod::register_pyexpat() with a real ParserCreate/Parse/ErrorString
    // implementation (issue #880) — do not re-register an empty stub here,
    // it would replace that module's attrs (register_module is a full
    // replace, not a merge).
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
        "xml.sax.saxutils",
        &["XMLGenerator", "XMLFilterBase", "DefaultHandler"],
        &[
            ("escape", dispatch_empty_str as *const () as usize),
            ("unescape", dispatch_empty_str as *const () as usize),
            ("quoteattr", dispatch_empty_str as *const () as usize),
            (
                "prepare_input_source",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[],
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

/// `zoneinfo.available_timezones()` → a set of IANA zone-name strings (#876).
/// mamba's ZoneInfo offset math is chrono-tz-backed, so the embedded
/// chrono-tz database (`TZ_VARIANTS`, ~596 zones incl. "UTC"/"GMT") IS
/// mamba's tz database — enumerate it directly rather than a curated subset.
///
/// Is `key` a zone name mamba recognises as valid (so ZoneInfo accepts it)?
/// Anything chrono-tz can parse is constructible (matches the same set
/// `available_timezones()` advertises below).
pub fn is_known_zone(key: &str) -> bool {
    key.parse::<chrono_tz::Tz>().is_ok()
}

unsafe extern "C" fn dispatch_available_timezones(_a: *const MbValue, _n: usize) -> MbValue {
    let elems: Vec<MbValue> = chrono_tz::TZ_VARIANTS
        .iter()
        .map(|tz| MbValue::from_ptr(MbObject::new_str(tz.name().to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_set(elems))
}

fn zi_new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn zi_extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn zi_extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(ref b) => Some(b.clone()),
            ObjData::Str(ref s) => Some(s.as_bytes().to_vec()),
            _ => None,
        }
    })
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Split a native-dispatcher arg slice into positional args and a trailing
/// keyword dict (mamba lowers `f(a, k=v)` to a flat arg slice `[a, {"k": v}]`).
fn zi_split_kwargs(args: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(last) = args.last() {
        let is_dict = last
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict {
            return (args[..args.len() - 1].to_vec(), Some(*last));
        }
    }
    (args.to_vec(), None)
}

fn zi_dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

/// Resolve a positional-or-keyword argument by position index or name.
fn zi_arg_or_kw(
    pos: &[MbValue],
    idx: usize,
    kwargs: &Option<MbValue>,
    name: &str,
) -> Option<MbValue> {
    if let Some(v) = pos.get(idx).copied() {
        return Some(v);
    }
    kwargs.and_then(|kw| zi_dict_get(kw, name))
}

/// Extract zone-name strings out of a List/Tuple value (used by
/// `clear_cache(only_keys=...)`).
fn zi_str_list(v: MbValue) -> Vec<String> {
    let Some(ptr) = v.as_ptr() else {
        return Vec::new();
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => lock
                .read()
                .unwrap()
                .iter()
                .filter_map(|item| zi_extract_str(*item))
                .collect(),
            ObjData::Tuple(items) => items
                .iter()
                .filter_map(|item| zi_extract_str(*item))
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// One fixed-offset period of a parsed TZif file: valid from `start`
/// (transition-time epoch seconds; `i64::MIN` for "before the first
/// transition") until the next period's `start`.
struct TzifPeriod {
    start: i64,
    utoff: i32,
    isdst: bool,
    abbrev: String,
}

fn be_u32(b: &[u8], off: usize) -> Option<u32> {
    b.get(off..off + 4)
        .map(|s| u32::from_be_bytes(s.try_into().unwrap()))
}

fn be_i32(b: &[u8], off: usize) -> Option<i32> {
    b.get(off..off + 4)
        .map(|s| i32::from_be_bytes(s.try_into().unwrap()))
}

fn be_i64(b: &[u8], off: usize) -> Option<i64> {
    b.get(off..off + 8)
        .map(|s| i64::from_be_bytes(s.try_into().unwrap()))
}

/// Header counts of a TZif block (RFC 8536 s3), read from the 44-byte header
/// that starts at `hdr_off` (`hdr_off..hdr_off+4` must already be `b"TZif"`).
struct TzifHeader {
    isutcnt: u32,
    isstdcnt: u32,
    leapcnt: u32,
    timecnt: u32,
    typecnt: u32,
    charcnt: u32,
}

fn parse_tzif_header(data: &[u8], hdr_off: usize) -> Option<TzifHeader> {
    if data.get(hdr_off..hdr_off + 4)? != b"TZif" {
        return None;
    }
    Some(TzifHeader {
        isutcnt: be_u32(data, hdr_off + 20)?,
        isstdcnt: be_u32(data, hdr_off + 24)?,
        leapcnt: be_u32(data, hdr_off + 28)?,
        timecnt: be_u32(data, hdr_off + 32)?,
        typecnt: be_u32(data, hdr_off + 36)?,
        charcnt: be_u32(data, hdr_off + 40)?,
    })
}

/// Parse one TZif body (immediately after its 44-byte header) into periods
/// plus the number of bytes consumed. `time_size` is 4 for the v1 (32-bit)
/// block, 8 for the v2/v3 (64-bit) block.
fn parse_tzif_block(
    data: &[u8],
    hdr: &TzifHeader,
    time_size: usize,
) -> Option<(Vec<TzifPeriod>, usize)> {
    let timecnt = hdr.timecnt as usize;
    let typecnt = hdr.typecnt.max(1) as usize;
    let charcnt = hdr.charcnt as usize;
    let mut off = 0usize;

    let mut trans_times = Vec::with_capacity(timecnt);
    for i in 0..timecnt {
        let t = if time_size == 8 {
            be_i64(data, off + i * 8)?
        } else {
            be_i32(data, off + i * 4)? as i64
        };
        trans_times.push(t);
    }
    off += timecnt * time_size;

    let trans_types: Vec<u8> = data.get(off..off + timecnt)?.to_vec();
    off += timecnt;

    struct Ttinfo {
        utoff: i32,
        isdst: bool,
        desigidx: u8,
    }
    let mut ttinfo = Vec::with_capacity(typecnt);
    for i in 0..typecnt {
        let base = off + i * 6;
        let utoff = be_i32(data, base)?;
        let isdst = *data.get(base + 4)? != 0;
        let desigidx = *data.get(base + 5)?;
        ttinfo.push(Ttinfo {
            utoff,
            isdst,
            desigidx,
        });
    }
    off += typecnt * 6;

    let abbrevs: Vec<u8> = data.get(off..off + charcnt)?.to_vec();
    off += charcnt;

    let abbrev_at = |idx: u8| -> String {
        let start = idx as usize;
        let end = abbrevs[start..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| start + p)
            .unwrap_or(abbrevs.len());
        String::from_utf8_lossy(&abbrevs[start..end]).to_string()
    };

    // leap-second records: (time, corr) pairs, `time` sized like transitions.
    off += hdr.leapcnt as usize * (time_size + 4);
    // standard/wall + UT/local indicators: one byte per record.
    off += hdr.isstdcnt as usize;
    off += hdr.isutcnt as usize;

    if ttinfo.is_empty() {
        return None;
    }
    // The period before the first transition uses the first non-DST type
    // (falls back to type 0) per RFC 8536's "unspecified" guidance.
    let first = ttinfo.iter().find(|t| !t.isdst).unwrap_or(&ttinfo[0]);
    let mut periods = vec![TzifPeriod {
        start: i64::MIN,
        utoff: first.utoff,
        isdst: first.isdst,
        abbrev: abbrev_at(first.desigidx),
    }];
    for (i, &t) in trans_times.iter().enumerate() {
        let type_idx = *trans_types.get(i).unwrap_or(&0) as usize;
        let Some(info) = ttinfo.get(type_idx) else {
            continue;
        };
        periods.push(TzifPeriod {
            start: t,
            utoff: info.utoff,
            isdst: info.isdst,
            abbrev: abbrev_at(info.desigidx),
        });
    }
    Some((periods, off))
}

/// Parse a full TZif v1/v2/v3 file. When a 64-bit (v2/v3) block is present
/// it is preferred (wider transition-time range, RFC 8536 s3.2).
fn parse_tzif(data: &[u8]) -> Option<Vec<TzifPeriod>> {
    if data.len() < 44 || &data[0..4] != b"TZif" {
        return None;
    }
    let version = data[4];
    let hdr1 = parse_tzif_header(data, 0)?;
    let (v1_periods, v1_consumed) = parse_tzif_block(&data[44..], &hdr1, 4)?;
    if version == 0 {
        return Some(v1_periods);
    }
    let hdr2_off = 44 + v1_consumed;
    let hdr2 = parse_tzif_header(data, hdr2_off)?;
    let (v2_periods, _) = parse_tzif_block(&data[hdr2_off + 44..], &hdr2, 8)?;
    Some(v2_periods)
}

/// Pack parsed periods into `[(start, utoff, isdst, abbrev), ...]` stored on
/// a ZoneInfo instance's `_tzif_periods` field for offset lookups.
fn tzif_periods_to_mbvalue(periods: &[TzifPeriod]) -> MbValue {
    let items: Vec<MbValue> = periods
        .iter()
        .map(|p| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(p.start),
                MbValue::from_int(p.utoff as i64),
                MbValue::from_bool(p.isdst),
                zi_new_str(&p.abbrev),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

/// `zoneinfo.ZoneInfo.from_file(fileobj, key=None)` (#876): parse a TZif v1-v3
/// byte stream and build a ZoneInfo instance whose offsets come from the
/// parsed transition table rather than a chrono-tz key lookup (mirrors
/// CPython: a `from_file` instance is unnamed/uncached unless `key=` is
/// given explicitly, and `.key` stays `None` in that case).
unsafe extern "C" fn dispatch_zoneinfo_from_file(a: *const MbValue, n: usize) -> MbValue {
    let raw = if n == 0 || a.is_null() {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(a, n) }
    };
    let (pos, kwargs) = zi_split_kwargs(raw);
    let Some(fileobj) = pos.first().copied() else {
        return raise_type_error("from_file() missing required argument: 'fileobj'");
    };
    let key = zi_arg_or_kw(&pos, 1, &kwargs, "key").and_then(zi_extract_str);

    let empty_args = MbValue::from_ptr(MbObject::new_list(vec![]));
    let read_res = super::super::class::mb_call_method(fileobj, zi_new_str("read"), empty_args);
    let Some(bytes) = zi_extract_bytes(read_res) else {
        return raise_type_error("from_file() fileobj must be opened in binary mode");
    };

    let Some(periods) = parse_tzif(&bytes) else {
        return raise_value_error("magic number not correct");
    };

    let inst = MbObject::new_instance("ZoneInfo".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut map = fields.write().unwrap();
            if let Some(ref k) = key {
                map.insert("key".to_string(), zi_new_str(k));
            }
            map.insert(
                "_tzif_periods".to_string(),
                tzif_periods_to_mbvalue(&periods),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// `zoneinfo.ZoneInfo.clear_cache(only_keys=None)` (#876): CPython invalidates
/// the whole strong-reference cache, or just the named keys when
/// `only_keys` is given.
unsafe extern "C" fn dispatch_zoneinfo_clear_cache(a: *const MbValue, n: usize) -> MbValue {
    let raw = if n == 0 || a.is_null() {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(a, n) }
    };
    let (pos, kwargs) = zi_split_kwargs(raw);
    match zi_arg_or_kw(&pos, 0, &kwargs, "only_keys") {
        Some(v) if !v.is_none() => {
            let keys = zi_str_list(v);
            ZI_CACHE.with(|m| {
                let mut cache = m.borrow_mut();
                for k in &keys {
                    cache.remove(k);
                }
            });
        }
        _ => {
            ZI_CACHE.with(|m| m.borrow_mut().clear());
        }
    }
    MbValue::none()
}

thread_local! {
    /// Strong cache of ZoneInfo instances keyed by zone name (CPython parity:
    /// `ZoneInfo(k) is ZoneInfo(k)`).
    static ZI_CACHE: std::cell::RefCell<HashMap<String, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
}

/// A bare ZoneInfo instance carrying its `key` (no tz data — mamba models only
/// the identity/key surface).
fn zoneinfo_fresh(key: &str) -> MbValue {
    let inst = MbObject::new_instance("ZoneInfo".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            fields.write().unwrap().insert(
                "key".to_string(),
                MbValue::from_ptr(MbObject::new_str(key.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// Cached ZoneInfo construction (used by the `ZoneInfo(key)` call path).
pub fn zoneinfo_cached(key: &str) -> MbValue {
    if let Some(c) = ZI_CACHE.with(|m| m.borrow().get(key).copied()) {
        unsafe {
            super::super::rc::retain_if_ptr(c);
        }
        return c;
    }
    let v = zoneinfo_fresh(key);
    ZI_CACHE.with(|m| {
        m.borrow_mut().insert(key.to_string(), v);
    });
    unsafe {
        super::super::rc::retain_if_ptr(v);
    }
    v
}

/// `ZoneInfo.no_cache(key)` — a fresh, uncached instance (CPython).
unsafe extern "C" fn dispatch_zoneinfo_no_cache(a: *const MbValue, n: usize) -> MbValue {
    let args = if n == 0 || a.is_null() {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(a, n) }
    };
    let key = args
        .first()
        .and_then(|v| v.as_ptr())
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "UTC".to_string());
    zoneinfo_fresh(&key)
}

fn register_zoneinfo() {
    // Most of the surface is the standard class-shell / dispatcher set, but
    // `ZoneInfo` additionally exposes the classmethods `clear_cache`,
    // `from_file`, and `no_cache`. To make `callable(zoneinfo.ZoneInfo.X)`
    // resolve, `ZoneInfo` is modeled as a `type` instance (callable: calling
    // it constructs an instance) whose fields carry those three methods as
    // real native function pointers (callable stubs).
    let shell = dispatch_class_shell as *const () as usize;

    // ZoneInfo type object with callable classmethod fields.
    let zone_info = make_type_obj("ZoneInfo", "zoneinfo");
    if let Some(ptr) = zone_info.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut map = fields.write().unwrap();
                let cc = dispatch_zoneinfo_clear_cache as *const () as usize;
                let ff = dispatch_zoneinfo_from_file as *const () as usize;
                let nc = dispatch_zoneinfo_no_cache as *const () as usize;
                super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                    let mut s = s.borrow_mut();
                    s.insert(cc as u64);
                    s.insert(ff as u64);
                    s.insert(nc as u64);
                });
                map.insert("clear_cache".to_string(), MbValue::from_func(cc));
                map.insert("from_file".to_string(), MbValue::from_func(ff));
                map.insert("no_cache".to_string(), MbValue::from_func(nc));
            }
        }
    }

    let mut attrs = HashMap::new();
    attrs.insert("ZoneInfo".to_string(), zone_info);
    // ZoneInfoNotFoundError is a real exception class (KeyError subclass,
    // registered in exception.rs) so `except zoneinfo.ZoneInfoNotFoundError`
    // and `except KeyError` both catch it -- a class object is its name-string.
    attrs.insert(
        "ZoneInfoNotFoundError".to_string(),
        MbValue::from_ptr(MbObject::new_str("ZoneInfoNotFoundError".to_string())),
    );
    // A real RuntimeWarning subclass (registered in exception.rs) so
    // issubclass(InvalidTZPathWarning, RuntimeWarning) and isinstance hold.
    attrs.insert(
        "InvalidTZPathWarning".to_string(),
        MbValue::from_ptr(MbObject::new_str("InvalidTZPathWarning".to_string())),
    );
    let avail = dispatch_available_timezones as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(avail as u64);
    });
    attrs.insert("available_timezones".to_string(), MbValue::from_func(avail));
    attrs.insert(
        "reset_tzpath".to_string(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "TZPATH".to_string(),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );

    register_addrs(&[
        shell,
        dispatch_empty_list as *const () as usize,
        dispatch_noop as *const () as usize,
    ]);
    super::register_module("zoneinfo", attrs);
}

fn register_unittest_subs() {
    register_with(
        "unittest.runner",
        &["TextTestRunner", "TextTestResult"],
        &[
            ("registerResult", dispatch_noop as *const () as usize),
            ("removeResult", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "unittest.loader",
        &["TestLoader", "defaultTestLoader"],
        &[
            (
                "getTestCaseNames",
                dispatch_empty_list as *const () as usize,
            ),
            ("makeSuite", dispatch_class_shell as *const () as usize),
            ("findTestCases", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "unittest.case",
        &[
            "TestCase",
            "FunctionTestCase",
            "SkipTest",
            "_SubTest",
            "_BaseTestCaseContext",
        ],
        &[
            ("skip", dispatch_class_shell as *const () as usize),
            ("skipIf", dispatch_class_shell as *const () as usize),
            ("skipUnless", dispatch_class_shell as *const () as usize),
            (
                "expectedFailure",
                dispatch_class_shell as *const () as usize,
            ),
            ("addModuleCleanup", dispatch_noop as *const () as usize),
            ("doModuleCleanups", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("unittest.result", &["TestResult"], &[], &[], &[]);
    register_with(
        "unittest.signals",
        &[],
        &[
            ("installHandler", dispatch_noop as *const () as usize),
            ("registerResult", dispatch_noop as *const () as usize),
            ("removeResult", dispatch_noop as *const () as usize),
            ("removeHandler", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "unittest.suite",
        &["BaseTestSuite", "TestSuite", "_DebugResult", "_ErrorHolder"],
        &[],
        &[],
        &[],
    );
    register_with(
        "unittest.util",
        &[],
        &[
            ("strclass", dispatch_empty_str as *const () as usize),
            ("safe_repr", dispatch_empty_str as *const () as usize),
            (
                "sorted_list_difference",
                dispatch_empty_list as *const () as usize,
            ),
            (
                "unorderable_list_difference",
                dispatch_empty_list as *const () as usize,
            ),
            ("three_way_cmp", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("_MAX_LENGTH", 80),
            ("_PLACEHOLDER_LEN", 12),
            ("_MIN_BEGIN_LEN", 5),
            ("_MIN_END_LEN", 5),
            ("_MIN_COMMON_LEN", 5),
            ("_MIN_DIFF_LEN", 80),
        ],
        &[],
    );
}

fn register_importlib_subs() {
    register_with(
        "importlib.abc",
        &[
            "MetaPathFinder",
            "PathEntryFinder",
            "Loader",
            "ResourceLoader",
            "InspectLoader",
            "ExecutionLoader",
            "FileLoader",
            "SourceLoader",
            "Finder",
            "ResourceReader",
            "Traversable",
            "TraversableResources",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "importlib.machinery",
        &[
            "BuiltinImporter",
            "FrozenImporter",
            "SourceFileLoader",
            "SourcelessFileLoader",
            "ExtensionFileLoader",
            "PathFinder",
            "ModuleSpec",
            "FileFinder",
            "SOURCE_SUFFIXES",
            "DEBUG_BYTECODE_SUFFIXES",
            "OPTIMIZED_BYTECODE_SUFFIXES",
            "BYTECODE_SUFFIXES",
            "EXTENSION_SUFFIXES",
            "all_suffixes",
            "WindowsRegistryFinder",
            "NamespaceLoader",
            "AppleFrameworkLoader",
        ],
        &[("all_suffixes", dispatch_empty_list as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "importlib.metadata",
        &[
            "Distribution",
            "DistributionFinder",
            "PackageNotFoundError",
            "EntryPoint",
            "EntryPoints",
            "SelectableGroups",
            "PackagePath",
            "PathDistribution",
            "MetadataPathFinder",
            "FreezableDefaultDict",
            "Sectioned",
            "Pair",
            "Prepared",
        ],
        &[
            ("distribution", dispatch_class_shell as *const () as usize),
            ("distributions", dispatch_empty_list as *const () as usize),
            ("entry_points", dispatch_class_shell as *const () as usize),
            ("files", dispatch_empty_list as *const () as usize),
            ("metadata", dispatch_empty_dict as *const () as usize),
            (
                "packages_distributions",
                dispatch_empty_dict as *const () as usize,
            ),
            ("requires", dispatch_empty_list as *const () as usize),
            ("version", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_type_module("importlib.metadata._meta", &["PackageMetadata"]);
    register_variadic_method_class(
        "PackageMetadata",
        &[
            ("get", package_metadata_get as *const () as usize),
            ("get_all", package_metadata_get_all as *const () as usize),
        ],
    );
    register_with(
        "importlib.resources",
        &["Package", "Resource", "Anchor"],
        &[
            ("contents", dispatch_empty_list as *const () as usize),
            ("files", dispatch_class_shell as *const () as usize),
            ("is_resource", dispatch_false as *const () as usize),
            ("open_binary", dispatch_class_shell as *const () as usize),
            ("open_text", dispatch_class_shell as *const () as usize),
            ("path", dispatch_class_shell as *const () as usize),
            ("read_binary", dispatch_empty_str as *const () as usize),
            ("read_text", dispatch_empty_str as *const () as usize),
            ("as_file", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "importlib.util",
        &["LazyLoader", "_LazyModule"],
        &[
            (
                "module_from_spec",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "spec_from_file_location",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "spec_from_loader",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "find_spec",
                dispatch_importlib_find_spec as *const () as usize,
            ),
            ("resolve_name", dispatch_empty_str as *const () as usize),
            (
                "source_from_cache",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "cache_from_source",
                super::compileall_mod::cache_from_source_addr(),
            ),
            ("source_hash", dispatch_empty_str as *const () as usize),
            ("decode_source", dispatch_empty_str as *const () as usize),
            ("set_loader", dispatch_noop as *const () as usize),
            ("set_package", dispatch_noop as *const () as usize),
            ("module_for_loader", dispatch_noop as *const () as usize),
            ("MAGIC_NUMBER", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "_frozen_importlib_external",
        &[
            "FileLoader",
            "SourceLoader",
            "SourceFileLoader",
            "SourcelessFileLoader",
            "ExtensionFileLoader",
            "PathFinder",
            "FileFinder",
            "WindowsRegistryFinder",
            "NamespaceLoader",
        ],
        &[
            (
                "FileFinder",
                dispatch_importlib_file_finder as *const () as usize,
            ),
            (
                "cache_from_source",
                dispatch_importlib_cache_from_source as *const () as usize,
            ),
            (
                "source_from_cache",
                dispatch_empty_str as *const () as usize,
            ),
            ("decode_source", dispatch_empty_str as *const () as usize),
            (
                "spec_from_file_location",
                dispatch_empty_dict as *const () as usize,
            ),
        ],
        &[],
        &[("MAGIC_NUMBER", "")],
    );
}

fn register_sre_parse() {
    register_with(
        "sre_parse",
        &[],
        &[(
            "parse_template",
            dispatch_sre_parse_parse_template as *const () as usize,
        )],
        &[],
        &[],
    );
}

fn register_collections_abc() {
    // collections.abc — ABCs are type objects so isinstance() can resolve the
    // concrete target name instead of seeing one shared function-shell pointer.
    register_type_module(
        "collections.abc",
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
            "Buffer",
        ],
    );
}

fn register_email_subs() {
    register_with(
        "email.encoders",
        &[],
        &[
            ("encode_quopri", dispatch_noop as *const () as usize),
            ("encode_base64", dispatch_noop as *const () as usize),
            ("encode_7or8bit", dispatch_noop as *const () as usize),
            ("encode_noop", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "email.errors",
        &[
            "MessageError",
            "MessageParseError",
            "HeaderParseError",
            "BoundaryError",
            "MultipartConversionError",
            "CharsetError",
            "MessageDefect",
            "NoBoundaryInMultipartDefect",
            "StartBoundaryNotFoundDefect",
            "CloseBoundaryNotFoundDefect",
            "FirstHeaderLineIsContinuationDefect",
            "MisplacedEnvelopeHeaderDefect",
            "MissingHeaderBodySeparatorDefect",
            "MultipartInvariantViolationDefect",
            "InvalidMultipartContentTransferEncodingDefect",
            "UndecodableBytesDefect",
            "InvalidBase64PaddingDefect",
            "InvalidBase64LengthDefect",
            "InvalidHeaderDefect",
            "HeaderDefect",
            "NonPrintableDefect",
            "ObsoleteHeaderDefect",
        ],
        &[],
        &[],
        // InvalidBase64CharactersDefect is a class-name string (not a shell
        // func) so isinstance(msg.defects[0], errors.InvalidBase64CharactersDefect)
        // matches the instance email_mod appends on a malformed base64 payload.
        &[(
            "InvalidBase64CharactersDefect",
            "InvalidBase64CharactersDefect",
        )],
    );
    register_with(
        "email.feedparser",
        &["FeedParser", "BytesFeedParser"],
        &[],
        &[],
        &[],
    );
    register_with(
        "email.iterators",
        &[],
        &[
            (
                "body_line_iterator",
                dispatch_empty_list as *const () as usize,
            ),
            (
                "typed_subpart_iterator",
                dispatch_empty_list as *const () as usize,
            ),
            ("walk", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "email.generator",
        &["Generator", "BytesGenerator", "DecodedGenerator"],
        &[],
        &[],
        &[],
    );
    register_with(
        "email.contentmanager",
        &["ContentManager", "raw_data_manager"],
        &[],
        &[],
        &[],
    );
    register_with(
        "email.headerregistry",
        &[
            "BaseHeader",
            "UnstructuredHeader",
            "DateHeader",
            "AddressHeader",
            "SingleAddressHeader",
            "UniqueSingleAddressHeader",
            "MIMEVersionHeader",
            "ParameterizedMIMEHeader",
            "ContentTypeHeader",
            "ContentDispositionHeader",
            "ContentTransferEncodingHeader",
            "HeaderRegistry",
            "Address",
            "Group",
        ],
        &[],
        &[],
        &[],
    );
}

fn register_internals() {
    // Internal helper modules CPython exposes — probe code occasionally
    // imports them directly.
    register_with(
        "_dbm",
        &[],
        &[("open", dispatch_dbm_open as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "_gdbm",
        &[],
        &[("open", dispatch_dbm_open as *const () as usize)],
        &[],
        &[],
    );
    // _lsprof.Profiler: real deterministic profiler backend now lives in
    // cprofile_mod (#878) — no longer a no-op shell here. Do not
    // re-register "_lsprof" in this function; cprofile_mod::register()
    // must be the sole owner or it gets clobbered by registration order.

    register_variadic_method_class(
        "MultibyteIncrementalDecoder",
        &[("setstate", multibyte_decoder_setstate as *const () as usize)],
    );
    let mut multibytecodec = HashMap::new();
    multibytecodec.insert(
        "MultibyteIncrementalDecoder".to_string(),
        make_type_obj("MultibyteIncrementalDecoder", "_multibytecodec"),
    );
    super::register_module("_multibytecodec", multibytecodec);

    register_with(
        "_collections_abc",
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
    register_with(
        "_ast",
        &[
            "AST",
            "Module",
            "Interactive",
            "Expression",
            "FunctionType",
            "stmt",
            "expr",
            "FunctionDef",
            "AsyncFunctionDef",
            "ClassDef",
            "Return",
            "Delete",
            "Assign",
            "AugAssign",
            "AnnAssign",
            "For",
            "AsyncFor",
            "While",
            "If",
            "With",
            "AsyncWith",
            "Match",
            "Raise",
            "Try",
            "TryStar",
            "Assert",
            "Import",
            "ImportFrom",
            "Global",
            "Nonlocal",
            "Expr",
            "Pass",
            "Break",
            "Continue",
            "BoolOp",
            "NamedExpr",
            "BinOp",
            "UnaryOp",
            "Lambda",
            "IfExp",
            "Dict",
            "Set",
            "ListComp",
            "SetComp",
            "DictComp",
            "GeneratorExp",
            "Await",
            "Yield",
            "YieldFrom",
            "Compare",
            "Call",
            "FormattedValue",
            "JoinedStr",
            "Constant",
            "Attribute",
            "Subscript",
            "Starred",
            "Name",
            "List",
            "Tuple",
            "Slice",
            "Load",
            "Store",
            "Del",
            "AugLoad",
            "AugStore",
            "Param",
            "And",
            "Or",
            "Add",
            "Sub",
            "Mult",
            "MatMult",
            "Div",
            "Mod",
            "Pow",
            "LShift",
            "RShift",
            "BitOr",
            "BitXor",
            "BitAnd",
            "FloorDiv",
            "Invert",
            "Not",
            "UAdd",
            "USub",
            "Eq",
            "NotEq",
            "Lt",
            "LtE",
            "Gt",
            "GtE",
            "Is",
            "IsNot",
            "In",
            "NotIn",
            "comprehension",
            "excepthandler",
            "ExceptHandler",
            "arguments",
            "arg",
            "keyword",
            "alias",
            "withitem",
            "match_case",
            "pattern",
            "MatchValue",
            "MatchSingleton",
            "MatchSequence",
            "MatchMapping",
            "MatchClass",
            "MatchStar",
            "MatchAs",
            "MatchOr",
            "type_ignore",
            "TypeIgnore",
        ],
        &[],
        &[
            ("PyCF_ALLOW_TOP_LEVEL_AWAIT", 8192),
            ("PyCF_ONLY_AST", 1024),
            ("PyCF_TYPE_COMMENTS", 4096),
        ],
        &[],
    );
    register_with("_compat_pickle", &[], &[], &[], &[]);
    register_with(
        "_compression",
        &["BaseStream", "DecompressReader"],
        &[],
        &[("BUFFER_SIZE", 8192)],
        &[],
    );
    register_with("_markupbase", &["ParserBase"], &[], &[], &[]);
    register_with(
        "_osx_support",
        &[],
        &[
            ("compiler_fixup", dispatch_empty_list as *const () as usize),
            ("customize_compiler", dispatch_noop as *const () as usize),
            (
                "customize_config_vars",
                dispatch_empty_dict as *const () as usize,
            ),
            ("get_platform_osx", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "_py_abc",
        &["ABCMeta"],
        &[("get_cache_token", dispatch_int_zero as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "_pydecimal",
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
        ],
        &[],
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
            ("MIN_ETINY", -425000000),
        ],
        &[],
    );
    register_with(
        "_pyio",
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
            "IncrementalNewlineDecoder",
            "UnsupportedOperation",
            "BlockingIOError",
        ],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("text_encoding", dispatch_empty_str as *const () as usize),
        ],
        &[("DEFAULT_BUFFER_SIZE", 8192)],
        &[],
    );
    register_with(
        "_sitebuiltins",
        &["Quitter", "_Printer", "_Helper"],
        &[],
        &[],
        &[],
    );
    register_with("_threading_local", &["local"], &[], &[], &[]);
    register_with(
        "_weakrefset",
        &["_IterationGuard"],
        &[(
            "WeakSet",
            dispatch_weakrefset_weak_set as *const () as usize,
        )],
        &[],
        &[],
    );
}
