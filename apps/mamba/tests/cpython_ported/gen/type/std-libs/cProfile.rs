use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/cProfile/Profile__dump_stats__file_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_Profile__dump_stats__file_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__dump_stats__file_as_StrOrBytesPath_wrong"
# subject = "cProfile.Profile.dump_stats(file: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.dump_stats(file: StrOrBytesPath); call it with the wrong type.

typeshed contract: file is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.dump_stats(_W())  # file: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/Profile__print_stats__sort_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_Profile__print_stats__sort_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__print_stats__sort_as_typed_wrong"
# subject = "cProfile.Profile.print_stats(sort: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.print_stats(sort: typed); call it with the wrong type.

typeshed contract: sort is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.print_stats(_W())  # sort: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/Profile__run__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_Profile__run__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__run__cmd_as_str_wrong"
# subject = "cProfile.Profile.run(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.run(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.run(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/Profile__runcall__func_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_Profile__runcall__func_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__runcall__func_as_Callable_wrong"
# subject = "cProfile.Profile.runcall(func: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.runcall(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.runcall(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/Profile__runctx__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_Profile__runctx__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__runctx__cmd_as_str_wrong"
# subject = "cProfile.Profile.runctx(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.runctx(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.runctx(12345, None, None)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/label__code_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_label__code_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "label__code_as_typed_wrong"
# subject = "cProfile.label(code: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.label(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import label
try:
    label(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/run__statement_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_run__statement_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "run__statement_as_str_wrong"
# subject = "cProfile.run(statement: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.run(statement: str); call it with the wrong type.

typeshed contract: statement is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cProfile import run
try:
    run(12345)  # statement: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cProfile/runctx__statement_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cProfile_runctx__statement_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "runctx__statement_as_str_wrong"
# subject = "cProfile.runctx(statement: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.runctx(statement: str); call it with the wrong type.

typeshed contract: statement is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cProfile import runctx
try:
    runctx(12345, None, None)  # statement: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
