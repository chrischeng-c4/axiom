use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/sqlite3_dbapi2/DateFromTicks__ticks_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sqlite3_dbapi2_DateFromTicks__ticks_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "DateFromTicks__ticks_as_float_wrong"
# subject = "sqlite3.dbapi2.DateFromTicks(ticks: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.DateFromTicks(ticks: float); call it with the wrong type.

typeshed contract: ticks is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import DateFromTicks
try:
    DateFromTicks("not_a_float")  # ticks: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sqlite3_dbapi2/TimeFromTicks__ticks_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sqlite3_dbapi2_TimeFromTicks__ticks_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "TimeFromTicks__ticks_as_float_wrong"
# subject = "sqlite3.dbapi2.TimeFromTicks(ticks: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.TimeFromTicks(ticks: float); call it with the wrong type.

typeshed contract: ticks is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import TimeFromTicks
try:
    TimeFromTicks("not_a_float")  # ticks: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sqlite3_dbapi2/TimestampFromTicks__ticks_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sqlite3_dbapi2_TimestampFromTicks__ticks_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "TimestampFromTicks__ticks_as_float_wrong"
# subject = "sqlite3.dbapi2.TimestampFromTicks(ticks: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.TimestampFromTicks(ticks: float); call it with the wrong type.

typeshed contract: ticks is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import TimestampFromTicks
try:
    TimestampFromTicks("not_a_float")  # ticks: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sqlite3_dbapi2/enable_shared_cache__enable_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sqlite3_dbapi2_enable_shared_cache__enable_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "enable_shared_cache__enable_as_int_wrong"
# subject = "sqlite3.dbapi2.enable_shared_cache(enable: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.enable_shared_cache(enable: int); call it with the wrong type.

typeshed contract: enable is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import enable_shared_cache
try:
    enable_shared_cache("not_an_int")  # enable: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
