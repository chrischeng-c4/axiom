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
unsafe extern "C" fn dispatch_false(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}
// importlib.util.find_spec(name) -> spec | None. Routes to the real
// module-registry lookup so a missing module yields None (not an empty
// shell dict), matching CPython's "find_spec returns None when absent".
unsafe extern "C" fn dispatch_importlib_find_spec(a: *const MbValue, n: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(a, n) };
    super::importlib_mod::mb_importlib_find_spec(args.first().copied().unwrap_or_else(MbValue::none))
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
    register_distutils();
    register_ctypes();
    register_html_entities();
    register_xml_subs();
    register_zoneinfo();
    register_unittest_subs();
    register_importlib_subs();
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

fn register_ctypes() {
    register_with(
        "ctypes",
        &[
            "CDLL",
            "PyDLL",
            "WinDLL",
            "OleDLL",
            "LibraryLoader",
            "Structure",
            "Union",
            "Array",
            "BigEndianStructure",
            "LittleEndianStructure",
            "c_byte",
            "c_ubyte",
            "c_char",
            "c_char_p",
            "c_double",
            "c_longdouble",
            "c_float",
            "c_int",
            "c_uint",
            "c_int8",
            "c_uint8",
            "c_int16",
            "c_uint16",
            "c_int32",
            "c_uint32",
            "c_int64",
            "c_uint64",
            "c_long",
            "c_ulong",
            "c_longlong",
            "c_ulonglong",
            "c_short",
            "c_ushort",
            "c_size_t",
            "c_ssize_t",
            "c_void_p",
            "c_wchar",
            "c_wchar_p",
            "c_bool",
            "POINTER",
            "pointer",
            "byref",
            "cast",
            "addressof",
            "alignment",
            "sizeof",
            "string_at",
            "wstring_at",
            "memmove",
            "memset",
            "CFUNCTYPE",
            "WINFUNCTYPE",
            "PYFUNCTYPE",
            "HRESULT",
            "ArgumentError",
            "Error",
            "ARRAY",
            "BigEndianUnion",
            "LittleEndianUnion",
            "SetPointerType",
            "c_buffer",
            "c_time_t",
            "c_voidp",
            "create_string_buffer",
            "create_unicode_buffer",
            "py_object",
            "resize",
            "pythonapi",
        ],
        &[
            ("CDLL", dispatch_class_shell as *const () as usize),
            ("cdll", dispatch_class_shell as *const () as usize),
            ("windll", dispatch_class_shell as *const () as usize),
            ("oledll", dispatch_class_shell as *const () as usize),
            ("pydll", dispatch_class_shell as *const () as usize),
            ("get_errno", dispatch_int_zero as *const () as usize),
            ("set_errno", dispatch_int_zero as *const () as usize),
            ("get_last_error", dispatch_int_zero as *const () as usize),
            ("set_last_error", dispatch_int_zero as *const () as usize),
            ("FormatError", dispatch_empty_str as *const () as usize),
            ("WinError", dispatch_class_shell as *const () as usize),
            ("DllCanUnloadNow", dispatch_int_zero as *const () as usize),
            ("DllGetClassObject", dispatch_int_zero as *const () as usize),
            ("GetLastError", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("DEFAULT_MODE", 0),
            ("RTLD_LOCAL", 0),
            ("RTLD_GLOBAL", 256),
            ("FUNCFLAG_CDECL", 1),
            ("FUNCFLAG_HRESULT", 2),
            ("FUNCFLAG_PYTHONAPI", 4),
            ("FUNCFLAG_USE_ERRNO", 8),
            ("FUNCFLAG_USE_LASTERROR", 16),
            ("SIZEOF_TIME_T", 8),
        ],
        &[],
    );
    register_with(
        "ctypes.util",
        &[],
        &[
            ("find_library", dispatch_noop as *const () as usize),
            ("find_msvcrt", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "ctypes.wintypes",
        &[
            "BOOL",
            "BYTE",
            "WORD",
            "DWORD",
            "UINT",
            "INT",
            "FLOAT",
            "LPVOID",
            "LPCVOID",
            "HANDLE",
            "HWND",
            "HMODULE",
            "HINSTANCE",
            "HKEY",
            "HMENU",
            "HRESULT",
            "LPCWSTR",
            "LPWSTR",
            "LPCSTR",
            "LPSTR",
            "LARGE_INTEGER",
            "ULARGE_INTEGER",
            "SIZE",
            "POINT",
            "RECT",
            "FILETIME",
            "SYSTEMTIME",
            "MSG",
            "BSTR",
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
    register_with("xml.parsers", &[], &[], &[], &[]);
    register_with(
        "xml.parsers.expat",
        &[
            "ExpatError",
            "XMLParserType",
            "ParserCreate",
            "ErrorString",
            "error",
            "model",
            "errors",
        ],
        &[
            ("ParserCreate", dispatch_class_shell as *const () as usize),
            ("ErrorString", dispatch_empty_str as *const () as usize),
        ],
        &[("EXPAT_VERSION_NUMBER", 0)],
        &[("EXPAT_VERSION", "2.0.0"), ("native_encoding", "utf-8")],
    );
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
                map.insert("clear_cache".to_string(), MbValue::from_func(shell));
                map.insert("from_file".to_string(), MbValue::from_func(shell));
                map.insert("no_cache".to_string(), MbValue::from_func(shell));
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
    attrs.insert("InvalidTZPathWarning".to_string(), MbValue::from_func(shell));
    attrs.insert(
        "available_timezones".to_string(),
        MbValue::from_func(dispatch_empty_list as *const () as usize),
    );
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
        "email.charset",
        &["Charset"],
        &[
            ("add_alias", dispatch_noop as *const () as usize),
            ("add_charset", dispatch_noop as *const () as usize),
            ("add_codec", dispatch_noop as *const () as usize),
        ],
        &[("QP", 1), ("BASE64", 2), ("SHORTEST", 3)],
        &[],
    );
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
            "InvalidBase64CharactersDefect",
            "InvalidBase64LengthDefect",
            "InvalidHeaderDefect",
            "HeaderDefect",
            "NonPrintableDefect",
            "ObsoleteHeaderDefect",
        ],
        &[],
        &[],
        &[],
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
        &["WeakSet", "_IterationGuard"],
        &[],
        &[],
        &[],
    );
}
