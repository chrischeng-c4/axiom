use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_util/LazyLoader__exec_module__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_LazyLoader__exec_module__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "LazyLoader__exec_module__module_as_ModuleType_wrong"
# subject = "importlib.util.LazyLoader.exec_module(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.LazyLoader.exec_module(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import LazyLoader
obj = object.__new__(LazyLoader)
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

/// Ported from `tests/cpython/type/std-libs/importlib_util/LazyLoader__factory__loader_as_Loader_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_LazyLoader__factory__loader_as_Loader_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "LazyLoader__factory__loader_as_Loader_wrong"
# subject = "importlib.util.LazyLoader.factory(loader: Loader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.LazyLoader.factory(loader: Loader); call it with the wrong type.

typeshed contract: loader is Loader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import LazyLoader
try:
    LazyLoader.factory(_W())  # loader: Loader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/LazyLoader__init__loader_as_Loader_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_LazyLoader__init__loader_as_Loader_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "LazyLoader__init__loader_as_Loader_wrong"
# subject = "importlib.util.LazyLoader.__init__(loader: Loader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.LazyLoader.__init__(loader: Loader); call it with the wrong type.

typeshed contract: loader is Loader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import LazyLoader
try:
    LazyLoader(_W())  # loader: Loader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/find_spec__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_find_spec__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "find_spec__name_as_str_wrong"
# subject = "importlib.util.find_spec(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.find_spec(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.util import find_spec
try:
    find_spec(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/module_for_loader__fxn_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_module_for_loader__fxn_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "module_for_loader__fxn_as_Callable_wrong"
# subject = "importlib.util.module_for_loader(fxn: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.module_for_loader(fxn: Callable); call it with the wrong type.

typeshed contract: fxn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import module_for_loader
try:
    module_for_loader(_W())  # fxn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/resolve_name__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_resolve_name__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "resolve_name__name_as_str_wrong"
# subject = "importlib.util.resolve_name(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.resolve_name(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.util import resolve_name
try:
    resolve_name(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/set_loader__fxn_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_set_loader__fxn_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "set_loader__fxn_as_Callable_wrong"
# subject = "importlib.util.set_loader(fxn: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.set_loader(fxn: Callable); call it with the wrong type.

typeshed contract: fxn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import set_loader
try:
    set_loader(_W())  # fxn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/set_package__fxn_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_set_package__fxn_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "set_package__fxn_as_Callable_wrong"
# subject = "importlib.util.set_package(fxn: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.set_package(fxn: Callable); call it with the wrong type.

typeshed contract: fxn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import set_package
try:
    set_package(_W())  # fxn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_util/source_hash__source_bytes_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_util_source_hash__source_bytes_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "source_hash__source_bytes_as_ReadableBuffer_wrong"
# subject = "importlib.util.source_hash(source_bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.source_hash(source_bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: source_bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import source_hash
try:
    source_hash(_W())  # source_bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
