use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pkgutil/ImpImporter__init__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_ImpImporter__init__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "ImpImporter__init__path_as_typed_wrong"
# subject = "pkgutil.ImpImporter.__init__(path: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.ImpImporter.__init__(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import ImpImporter
try:
    ImpImporter(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/ImpLoader__init__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_ImpLoader__init__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "ImpLoader__init__fullname_as_str_wrong"
# subject = "pkgutil.ImpLoader.__init__(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.ImpLoader.__init__(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import ImpLoader
try:
    ImpLoader(12345, None, None, None)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/extend_path__path_as__PathT_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_extend_path__path_as__PathT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "extend_path__path_as__PathT_wrong"
# subject = "pkgutil.extend_path(path: _PathT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.extend_path(path: _PathT); call it with the wrong type.

typeshed contract: path is _PathT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import extend_path
try:
    extend_path(_W(), "")  # path: _PathT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/find_loader__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_find_loader__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "find_loader__fullname_as_str_wrong"
# subject = "pkgutil.find_loader(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.find_loader(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import find_loader
try:
    find_loader(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/get_data__package_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_get_data__package_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "get_data__package_as_str_wrong"
# subject = "pkgutil.get_data(package: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.get_data(package: str); call it with the wrong type.

typeshed contract: package is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import get_data
try:
    get_data(12345, "")  # package: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/get_importer__path_item_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_get_importer__path_item_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "get_importer__path_item_as_StrOrBytesPath_wrong"
# subject = "pkgutil.get_importer(path_item: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.get_importer(path_item: StrOrBytesPath); call it with the wrong type.

typeshed contract: path_item is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import get_importer
try:
    get_importer(_W())  # path_item: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/get_loader__module_or_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_get_loader__module_or_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "get_loader__module_or_name_as_str_wrong"
# subject = "pkgutil.get_loader(module_or_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.get_loader(module_or_name: str); call it with the wrong type.

typeshed contract: module_or_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import get_loader
try:
    get_loader(12345)  # module_or_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/iter_importers__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_iter_importers__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "iter_importers__fullname_as_str_wrong"
# subject = "pkgutil.iter_importers(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.iter_importers(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import iter_importers
try:
    iter_importers(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/iter_modules__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_iter_modules__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "iter_modules__path_as_typed_wrong"
# subject = "pkgutil.iter_modules(path: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.iter_modules(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import iter_modules
try:
    iter_modules(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/read_code__stream_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_read_code__stream_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "read_code__stream_as_SupportsRead_wrong"
# subject = "pkgutil.read_code(stream: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.read_code(stream: SupportsRead); call it with the wrong type.

typeshed contract: stream is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import read_code
try:
    read_code(_W())  # stream: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/resolve_name__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_resolve_name__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "resolve_name__name_as_str_wrong"
# subject = "pkgutil.resolve_name(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.resolve_name(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import resolve_name
try:
    resolve_name(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pkgutil/walk_packages__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pkgutil_walk_packages__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "walk_packages__path_as_typed_wrong"
# subject = "pkgutil.walk_packages(path: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.walk_packages(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import walk_packages
try:
    walk_packages(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
