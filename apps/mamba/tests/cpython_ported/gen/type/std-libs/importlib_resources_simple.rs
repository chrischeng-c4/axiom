use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/ResourceContainer__init__reader_as_SimpleReader_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_ResourceContainer__init__reader_as_SimpleReader_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceContainer__init__reader_as_SimpleReader_wrong"
# subject = "importlib.resources.simple.ResourceContainer.__init__(reader: SimpleReader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceContainer.__init__(reader: SimpleReader); call it with the wrong type.

typeshed contract: reader is SimpleReader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.simple import ResourceContainer
try:
    ResourceContainer(_W())  # reader: SimpleReader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/ResourceHandle__init__parent_as_ResourceContainer_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_ResourceHandle__init__parent_as_ResourceContainer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__init__parent_as_ResourceContainer_wrong"
# subject = "importlib.resources.simple.ResourceHandle.__init__(parent: ResourceContainer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.__init__(parent: ResourceContainer); call it with the wrong type.

typeshed contract: parent is ResourceContainer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.simple import ResourceHandle
try:
    ResourceHandle(_W(), "")  # parent: ResourceContainer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/ResourceHandle__joinpath__name_as_Never_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_ResourceHandle__joinpath__name_as_Never_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__joinpath__name_as_Never_wrong"
# subject = "importlib.resources.simple.ResourceHandle.joinpath(name: Never)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.joinpath(name: Never); call it with the wrong type.

typeshed contract: name is Never. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.simple import ResourceHandle
obj = object.__new__(ResourceHandle)
try:
    obj.joinpath(_W())  # name: Never <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/ResourceHandle__open__mode_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_ResourceHandle__open__mode_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__open__mode_as_Literal_wrong"
# subject = "importlib.resources.simple.ResourceHandle.open(mode: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.open(mode: Literal); call it with the wrong type.

typeshed contract: mode is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.simple import ResourceHandle
obj = object.__new__(ResourceHandle)
try:
    obj.open(_W())  # mode: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/ResourceHandle__open__mode_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_ResourceHandle__open__mode_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__open__mode_as_str_wrong"
# subject = "importlib.resources.simple.ResourceHandle.open(mode: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.open(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.resources.simple import ResourceHandle
obj = object.__new__(ResourceHandle)
try:
    obj.open(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources_simple/SimpleReader__open_binary__resource_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources_simple_SimpleReader__open_binary__resource_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "SimpleReader__open_binary__resource_as_str_wrong"
# subject = "importlib.resources.simple.SimpleReader.open_binary(resource: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.SimpleReader.open_binary(resource: str); call it with the wrong type.

typeshed contract: resource is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.resources.simple import SimpleReader
obj = object.__new__(SimpleReader)
try:
    obj.open_binary(12345)  # resource: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
