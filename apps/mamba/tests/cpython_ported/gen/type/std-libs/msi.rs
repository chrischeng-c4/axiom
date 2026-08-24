use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_msi/CreateRecord__count_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__msi_CreateRecord__count_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_msi"
# dimension = "type"
# case = "CreateRecord__count_as_int_wrong"
# subject = "_msi.CreateRecord(count: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _msi.CreateRecord(count: int); call it with the wrong type.

typeshed contract: count is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _msi import CreateRecord
try:
    CreateRecord("not_an_int")  # count: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_msi/FCICreate__cabname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__msi_FCICreate__cabname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_msi"
# dimension = "type"
# case = "FCICreate__cabname_as_str_wrong"
# subject = "_msi.FCICreate(cabname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _msi.FCICreate(cabname: str); call it with the wrong type.

typeshed contract: cabname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _msi import FCICreate
try:
    FCICreate(12345, None)  # cabname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_msi/OpenDatabase__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__msi_OpenDatabase__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_msi"
# dimension = "type"
# case = "OpenDatabase__path_as_str_wrong"
# subject = "_msi.OpenDatabase(path: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _msi.OpenDatabase(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _msi import OpenDatabase
try:
    OpenDatabase(12345, 0)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
