use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/getopt/GetoptError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_getopt_GetoptError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "type"
# case = "GetoptError__init__msg_as_str_wrong"
# subject = "getopt.GetoptError.__init__(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: getopt.GetoptError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from getopt import GetoptError
try:
    GetoptError(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/getopt/getopt__args_as__SliceableT_wrong.py`.
#[test]
fn test_gen_type_std_libs_getopt_getopt__args_as__SliceableT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "type"
# case = "getopt__args_as__SliceableT_wrong"
# subject = "getopt.getopt(args: _SliceableT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: getopt.getopt(args: _SliceableT); call it with the wrong type.

typeshed contract: args is _SliceableT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from getopt import getopt
try:
    getopt(_W(), "")  # args: _SliceableT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/getopt/gnu_getopt__args_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_std_libs_getopt_gnu_getopt__args_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "type"
# case = "gnu_getopt__args_as_Sequence_wrong"
# subject = "getopt.gnu_getopt(args: Sequence)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: getopt.gnu_getopt(args: Sequence); call it with the wrong type.

typeshed contract: args is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from getopt import gnu_getopt
try:
    gnu_getopt(_W(), "")  # args: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
