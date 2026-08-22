use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/dbm_dumb/open__file_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_dumb_open__file_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "type"
# case = "open__file_as_StrOrBytesPath_wrong"
# subject = "dbm.dumb.open(file: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm/dumb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.dumb.open(file: StrOrBytesPath); call it with the wrong type.

typeshed contract: file is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dbm.dumb import open
try:
    open(_W())  # file: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dbm_dumb/open__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_dbm_dumb_open__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "type"
# case = "open__file_as_str_wrong"
# subject = "dbm.dumb.open(file: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dbm/dumb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dbm.dumb.open(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dbm.dumb import open
try:
    open(12345)  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
