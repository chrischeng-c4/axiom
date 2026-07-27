use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/resource/getrlimit__resource_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_resource_getrlimit__resource_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "getrlimit__resource_as_int_wrong"
# subject = "resource.getrlimit(resource: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.getrlimit(resource: int); call it with the wrong type.

typeshed contract: resource is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import getrlimit
try:
    getrlimit("not_an_int")  # resource: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/resource/getrusage__who_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_resource_getrusage__who_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "getrusage__who_as_int_wrong"
# subject = "resource.getrusage(who: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.getrusage(who: int); call it with the wrong type.

typeshed contract: who is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import getrusage
try:
    getrusage("not_an_int")  # who: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/resource/prlimit__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_resource_prlimit__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "prlimit__pid_as_int_wrong"
# subject = "resource.prlimit(pid: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.prlimit(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import prlimit
try:
    prlimit("not_an_int", 0)  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/resource/setrlimit__resource_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_resource_setrlimit__resource_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "setrlimit__resource_as_int_wrong"
# subject = "resource.setrlimit(resource: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.setrlimit(resource: int); call it with the wrong type.

typeshed contract: resource is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import setrlimit
try:
    setrlimit("not_an_int", None)  # resource: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
