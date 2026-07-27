use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_readers/FileReader__init__loader_as_FileLoader_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_FileReader__init__loader_as_FileLoader_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "FileReader__init__loader_as_FileLoader_wrong"
# subject = "importlib.readers.FileReader.__init__(loader: FileLoader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.FileReader.__init__(loader: FileLoader); call it with the wrong type.

typeshed contract: loader is FileLoader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import FileReader
try:
    FileReader(_W())  # loader: FileLoader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/FileReader__resource_path__resource_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_FileReader__resource_path__resource_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "FileReader__resource_path__resource_as_StrPath_wrong"
# subject = "importlib.readers.FileReader.resource_path(resource: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.FileReader.resource_path(resource: StrPath); call it with the wrong type.

typeshed contract: resource is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import FileReader
obj = object.__new__(FileReader)
try:
    obj.resource_path(_W())  # resource: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/NamespaceReader__init__namespace_path_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_NamespaceReader__init__namespace_path_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "NamespaceReader__init__namespace_path_as_Iterable_wrong"
# subject = "importlib.readers.NamespaceReader.__init__(namespace_path: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.NamespaceReader.__init__(namespace_path: Iterable); call it with the wrong type.

typeshed contract: namespace_path is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import NamespaceReader
try:
    NamespaceReader(_W())  # namespace_path: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/NamespaceReader__resource_path__resource_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_NamespaceReader__resource_path__resource_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "NamespaceReader__resource_path__resource_as_str_wrong"
# subject = "importlib.readers.NamespaceReader.resource_path(resource: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.NamespaceReader.resource_path(resource: str); call it with the wrong type.

typeshed contract: resource is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.readers import NamespaceReader
obj = object.__new__(NamespaceReader)
try:
    obj.resource_path(12345)  # resource: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/ZipReader__init__loader_as_zipimporter_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_ZipReader__init__loader_as_zipimporter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "ZipReader__init__loader_as_zipimporter_wrong"
# subject = "importlib.readers.ZipReader.__init__(loader: zipimporter)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.ZipReader.__init__(loader: zipimporter); call it with the wrong type.

typeshed contract: loader is zipimporter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import ZipReader
try:
    ZipReader(_W(), "")  # loader: zipimporter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/ZipReader__is_resource__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_ZipReader__is_resource__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "ZipReader__is_resource__path_as_StrPath_wrong"
# subject = "importlib.readers.ZipReader.is_resource(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.ZipReader.is_resource(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import ZipReader
obj = object.__new__(ZipReader)
try:
    obj.is_resource(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/ZipReader__open_resource__resource_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_ZipReader__open_resource__resource_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "ZipReader__open_resource__resource_as_str_wrong"
# subject = "importlib.readers.ZipReader.open_resource(resource: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.ZipReader.open_resource(resource: str); call it with the wrong type.

typeshed contract: resource is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.readers import ZipReader
obj = object.__new__(ZipReader)
try:
    obj.open_resource(12345)  # resource: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_readers/remove_duplicates__items_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_readers_remove_duplicates__items_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "remove_duplicates__items_as_Iterable_wrong"
# subject = "importlib.readers.remove_duplicates(items: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.remove_duplicates(items: Iterable); call it with the wrong type.

typeshed contract: items is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import remove_duplicates
try:
    remove_duplicates(_W())  # items: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
