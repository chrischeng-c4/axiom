use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib__abc/Loader__create_module__spec_as_ModuleSpec_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib__abc_Loader__create_module__spec_as_ModuleSpec_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib__abc"
# dimension = "type"
# case = "Loader__create_module__spec_as_ModuleSpec_wrong"
# subject = "importlib._abc.Loader.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib._abc.Loader.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib._abc import Loader
obj = object.__new__(Loader)
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

/// Ported from `tests/cpython/type/std-libs/importlib__abc/Loader__exec_module__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib__abc_Loader__exec_module__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib__abc"
# dimension = "type"
# case = "Loader__exec_module__module_as_ModuleType_wrong"
# subject = "importlib._abc.Loader.exec_module(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib._abc.Loader.exec_module(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib._abc import Loader
obj = object.__new__(Loader)
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

/// Ported from `tests/cpython/type/std-libs/importlib__abc/Loader__load_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib__abc_Loader__load_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib__abc"
# dimension = "type"
# case = "Loader__load_module__fullname_as_str_wrong"
# subject = "importlib._abc.Loader.load_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib._abc.Loader.load_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib._abc import Loader
obj = object.__new__(Loader)
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

/// Ported from `tests/cpython/type/std-libs/importlib__abc/Loader__module_repr__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib__abc_Loader__module_repr__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib__abc"
# dimension = "type"
# case = "Loader__module_repr__module_as_ModuleType_wrong"
# subject = "importlib._abc.Loader.module_repr(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib._abc.Loader.module_repr(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib._abc import Loader
obj = object.__new__(Loader)
try:
    obj.module_repr(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
