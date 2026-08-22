use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pyclbr/Class__init__module_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pyclbr_Class__init__module_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "type"
# case = "Class__init__module_as_str_wrong"
# subject = "pyclbr.Class.__init__(module: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyclbr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyclbr.Class.__init__(module: str); call it with the wrong type.

typeshed contract: module is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyclbr import Class
try:
    Class(12345, "", None, "", 0)  # module: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pyclbr/Function__init__module_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pyclbr_Function__init__module_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "type"
# case = "Function__init__module_as_str_wrong"
# subject = "pyclbr.Function.__init__(module: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyclbr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyclbr.Function.__init__(module: str); call it with the wrong type.

typeshed contract: module is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyclbr import Function
try:
    Function(12345, "", "", 0)  # module: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pyclbr/readmodule__module_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pyclbr_readmodule__module_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "type"
# case = "readmodule__module_as_str_wrong"
# subject = "pyclbr.readmodule(module: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyclbr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyclbr.readmodule(module: str); call it with the wrong type.

typeshed contract: module is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyclbr import readmodule
try:
    readmodule(12345)  # module: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pyclbr/readmodule_ex__module_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pyclbr_readmodule_ex__module_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "type"
# case = "readmodule_ex__module_as_str_wrong"
# subject = "pyclbr.readmodule_ex(module: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyclbr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyclbr.readmodule_ex(module: str); call it with the wrong type.

typeshed contract: module is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyclbr import readmodule_ex
try:
    readmodule_ex(12345)  # module: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
