use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__create_module__spec_as_ModuleSpec_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__create_module__spec_as_ModuleSpec_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__create_module__spec_as_ModuleSpec_wrong"
# subject = "zipimport.zipimporter.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.create_module(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__exec_module__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__exec_module__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__exec_module__module_as_ModuleType_wrong"
# subject = "zipimport.zipimporter.exec_module(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.exec_module(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.exec_module(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__find_loader__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__find_loader__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__find_loader__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.find_loader(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.find_loader(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.find_loader(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__find_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__find_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__find_module__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.find_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.find_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.find_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__find_spec__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__find_spec__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__find_spec__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.find_spec(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.find_spec(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.find_spec(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__get_code__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__get_code__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_code__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.get_code(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_code(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_code(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__get_data__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__get_data__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_data__pathname_as_str_wrong"
# subject = "zipimport.zipimporter.get_data(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_data(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_data(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__get_filename__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__get_filename__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_filename__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.get_filename(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_filename(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_filename(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__get_resource_reader__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__get_resource_reader__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_resource_reader__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.get_resource_reader(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_resource_reader(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_resource_reader(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__get_source__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__get_source__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_source__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.get_source(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_source(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_source(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__init__path_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__init__path_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__init__path_as_StrOrBytesPath_wrong"
# subject = "zipimport.zipimporter.__init__(path: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.__init__(path: StrOrBytesPath); call it with the wrong type.

typeshed contract: path is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipimport import zipimporter
try:
    zipimporter(_W())  # path: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__init__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__init__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__init__path_as_str_wrong"
# subject = "zipimport.zipimporter.__init__(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.__init__(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
try:
    zipimporter(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__is_package__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__is_package__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__is_package__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.is_package(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.is_package(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.is_package(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipimport/zipimporter__load_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipimport_zipimporter__load_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__load_module__fullname_as_str_wrong"
# subject = "zipimport.zipimporter.load_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.load_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.load_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
