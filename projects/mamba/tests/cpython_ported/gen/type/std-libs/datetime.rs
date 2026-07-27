use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/datetime/date____add____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____add____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____add____value_as_timedelta_wrong"
# subject = "datetime.date.__add__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__add__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__add__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____format____fmt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____format____fmt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____format____fmt_as_str_wrong"
# subject = "datetime.date.__format__(fmt: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__format__(fmt: str); call it with the wrong type.

typeshed contract: fmt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
obj = object.__new__(date)
try:
    obj.__format__(12345)  # fmt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____ge____value_as_date_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____ge____value_as_date_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____ge____value_as_date_wrong"
# subject = "datetime.date.__ge__(value: date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__ge__(value: date); call it with the wrong type.

typeshed contract: value is date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__ge__(_W())  # value: date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____gt____value_as_date_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____gt____value_as_date_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____gt____value_as_date_wrong"
# subject = "datetime.date.__gt__(value: date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__gt__(value: date); call it with the wrong type.

typeshed contract: value is date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__gt__(_W())  # value: date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____le____value_as_date_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____le____value_as_date_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____le____value_as_date_wrong"
# subject = "datetime.date.__le__(value: date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__le__(value: date); call it with the wrong type.

typeshed contract: value is date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__le__(_W())  # value: date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____lt____value_as_date_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____lt____value_as_date_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____lt____value_as_date_wrong"
# subject = "datetime.date.__lt__(value: date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__lt__(value: date); call it with the wrong type.

typeshed contract: value is date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__lt__(_W())  # value: date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____new____year_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____new____year_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____new____year_as_SupportsIndex_wrong"
# subject = "datetime.date.__new__(year: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__new__(year: SupportsIndex); call it with the wrong type.

typeshed contract: year is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__new__(_W(), None, None)  # year: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date____radd____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date____radd____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____radd____value_as_timedelta_wrong"
# subject = "datetime.date.__radd__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__radd__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__radd__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__fromisocalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__fromisocalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__fromisocalendar__year_as_int_wrong"
# subject = "datetime.date.fromisocalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.fromisocalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
try:
    date.fromisocalendar("not_an_int", 0, 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__fromisoformat__date_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__fromisoformat__date_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__fromisoformat__date_string_as_str_wrong"
# subject = "datetime.date.fromisoformat(date_string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.fromisoformat(date_string: str); call it with the wrong type.

typeshed contract: date_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
try:
    date.fromisoformat(12345)  # date_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__fromordinal__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__fromordinal__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__fromordinal__n_as_int_wrong"
# subject = "datetime.date.fromordinal(n: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.fromordinal(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
try:
    date.fromordinal("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__fromtimestamp__timestamp_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__fromtimestamp__timestamp_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__fromtimestamp__timestamp_as_float_wrong"
# subject = "datetime.date.fromtimestamp(timestamp: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.fromtimestamp(timestamp: float); call it with the wrong type.

typeshed contract: timestamp is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
try:
    date.fromtimestamp("not_a_float")  # timestamp: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__replace__year_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__replace__year_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__replace__year_as_SupportsIndex_wrong"
# subject = "datetime.date.replace(year: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.replace(year: SupportsIndex); call it with the wrong type.

typeshed contract: year is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.replace(_W())  # year: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/date__strftime__format_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_date__strftime__format_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date__strftime__format_as_str_wrong"
# subject = "datetime.date.strftime(format: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.strftime(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import date
obj = object.__new__(date)
try:
    obj.strftime(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime____ge____value_as_datetime_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime____ge____value_as_datetime_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime____ge____value_as_datetime_wrong"
# subject = "datetime.datetime.__ge__(value: datetime)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.__ge__(value: datetime); call it with the wrong type.

typeshed contract: value is datetime. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.__ge__(_W())  # value: datetime <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime____gt____value_as_datetime_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime____gt____value_as_datetime_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime____gt____value_as_datetime_wrong"
# subject = "datetime.datetime.__gt__(value: datetime)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.__gt__(value: datetime); call it with the wrong type.

typeshed contract: value is datetime. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.__gt__(_W())  # value: datetime <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime____le____value_as_datetime_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime____le____value_as_datetime_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime____le____value_as_datetime_wrong"
# subject = "datetime.datetime.__le__(value: datetime)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.__le__(value: datetime); call it with the wrong type.

typeshed contract: value is datetime. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.__le__(_W())  # value: datetime <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime____lt____value_as_datetime_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime____lt____value_as_datetime_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime____lt____value_as_datetime_wrong"
# subject = "datetime.datetime.__lt__(value: datetime)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.__lt__(value: datetime); call it with the wrong type.

typeshed contract: value is datetime. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.__lt__(_W())  # value: datetime <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime____new____year_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime____new____year_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime____new____year_as_SupportsIndex_wrong"
# subject = "datetime.datetime.__new__(year: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.__new__(year: SupportsIndex); call it with the wrong type.

typeshed contract: year is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.__new__(_W(), None, None)  # year: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__astimezone__tz_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__astimezone__tz_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__astimezone__tz_as_typed_wrong"
# subject = "datetime.datetime.astimezone(tz: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.astimezone(tz: typed); call it with the wrong type.

typeshed contract: tz is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.astimezone(_W())  # tz: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__combine__date_as__Date_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__combine__date_as__Date_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__combine__date_as__Date_wrong"
# subject = "datetime.datetime.combine(date: _Date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.combine(date: _Date); call it with the wrong type.

typeshed contract: date is _Date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
try:
    datetime.combine(_W(), None)  # date: _Date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__fromisoformat__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__fromisoformat__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__fromisoformat__string_as_str_wrong"
# subject = "datetime.datetime.fromisoformat(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.fromisoformat(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import datetime
try:
    datetime.fromisoformat(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__fromtimestamp__timestamp_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__fromtimestamp__timestamp_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__fromtimestamp__timestamp_as_float_wrong"
# subject = "datetime.datetime.fromtimestamp(timestamp: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.fromtimestamp(timestamp: float); call it with the wrong type.

typeshed contract: timestamp is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import datetime
try:
    datetime.fromtimestamp("not_a_float")  # timestamp: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__isoformat__sep_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__isoformat__sep_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__isoformat__sep_as_str_wrong"
# subject = "datetime.datetime.isoformat(sep: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.isoformat(sep: str); call it with the wrong type.

typeshed contract: sep is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.isoformat(12345)  # sep: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__now__tz_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__now__tz_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__now__tz_as_typed_wrong"
# subject = "datetime.datetime.now(tz: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.now(tz: typed); call it with the wrong type.

typeshed contract: tz is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
try:
    datetime.now(_W())  # tz: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__replace__year_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__replace__year_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__replace__year_as_SupportsIndex_wrong"
# subject = "datetime.datetime.replace(year: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.replace(year: SupportsIndex); call it with the wrong type.

typeshed contract: year is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import datetime
obj = object.__new__(datetime)
try:
    obj.replace(_W())  # year: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__strptime__date_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__strptime__date_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__strptime__date_string_as_str_wrong"
# subject = "datetime.datetime.strptime(date_string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.strptime(date_string: str); call it with the wrong type.

typeshed contract: date_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import datetime
try:
    datetime.strptime(12345, "")  # date_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/datetime__utcfromtimestamp__t_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_datetime__utcfromtimestamp__t_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "datetime__utcfromtimestamp__t_as_float_wrong"
# subject = "datetime.datetime.utcfromtimestamp(t: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.datetime.utcfromtimestamp(t: float); call it with the wrong type.

typeshed contract: t is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import datetime
try:
    datetime.utcfromtimestamp("not_a_float")  # t: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____format____fmt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____format____fmt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____format____fmt_as_str_wrong"
# subject = "datetime.time.__format__(fmt: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__format__(fmt: str); call it with the wrong type.

typeshed contract: fmt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import time
obj = object.__new__(time)
try:
    obj.__format__(12345)  # fmt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____ge____value_as_time_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____ge____value_as_time_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____ge____value_as_time_wrong"
# subject = "datetime.time.__ge__(value: time)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__ge__(value: time); call it with the wrong type.

typeshed contract: value is time. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.__ge__(_W())  # value: time <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____gt____value_as_time_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____gt____value_as_time_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____gt____value_as_time_wrong"
# subject = "datetime.time.__gt__(value: time)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__gt__(value: time); call it with the wrong type.

typeshed contract: value is time. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.__gt__(_W())  # value: time <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____le____value_as_time_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____le____value_as_time_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____le____value_as_time_wrong"
# subject = "datetime.time.__le__(value: time)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__le__(value: time); call it with the wrong type.

typeshed contract: value is time. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.__le__(_W())  # value: time <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____lt____value_as_time_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____lt____value_as_time_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____lt____value_as_time_wrong"
# subject = "datetime.time.__lt__(value: time)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__lt__(value: time); call it with the wrong type.

typeshed contract: value is time. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.__lt__(_W())  # value: time <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time____new____hour_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time____new____hour_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time____new____hour_as_SupportsIndex_wrong"
# subject = "datetime.time.__new__(hour: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.__new__(hour: SupportsIndex); call it with the wrong type.

typeshed contract: hour is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.__new__(_W())  # hour: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time__fromisoformat__time_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time__fromisoformat__time_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time__fromisoformat__time_string_as_str_wrong"
# subject = "datetime.time.fromisoformat(time_string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.fromisoformat(time_string: str); call it with the wrong type.

typeshed contract: time_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import time
try:
    time.fromisoformat(12345)  # time_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time__isoformat__timespec_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time__isoformat__timespec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time__isoformat__timespec_as_str_wrong"
# subject = "datetime.time.isoformat(timespec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.isoformat(timespec: str); call it with the wrong type.

typeshed contract: timespec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import time
obj = object.__new__(time)
try:
    obj.isoformat(12345)  # timespec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time__replace__hour_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time__replace__hour_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time__replace__hour_as_SupportsIndex_wrong"
# subject = "datetime.time.replace(hour: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.replace(hour: SupportsIndex); call it with the wrong type.

typeshed contract: hour is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import time
obj = object.__new__(time)
try:
    obj.replace(_W())  # hour: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/time__strftime__format_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_time__strftime__format_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "time__strftime__format_as_str_wrong"
# subject = "datetime.time.strftime(format: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.time.strftime(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import time
obj = object.__new__(time)
try:
    obj.strftime(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____add____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____add____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____add____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__add__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__add__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__add__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____divmod____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____divmod____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____divmod____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__divmod__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__divmod__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__divmod__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____floordiv____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____floordiv____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____floordiv____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__floordiv__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__floordiv__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__floordiv__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____ge____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____ge____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____ge____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__ge__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__ge__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__ge__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____gt____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____gt____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____gt____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__gt__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__gt__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__gt__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____le____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____le____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____le____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__le__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__le__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__le__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____lt____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____lt____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____lt____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__lt__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__lt__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__lt__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____mod____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____mod____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____mod____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__mod__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__mod__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__mod__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____mul____value_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____mul____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____mul____value_as_float_wrong"
# subject = "datetime.timedelta.__mul__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__mul__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__mul__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____new____days_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____new____days_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____new____days_as_float_wrong"
# subject = "datetime.timedelta.__new__(days: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__new__(days: float); call it with the wrong type.

typeshed contract: days is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__new__("not_a_float")  # days: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____radd____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____radd____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____radd____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__radd__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__radd__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__radd__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____rmul____value_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____rmul____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____rmul____value_as_float_wrong"
# subject = "datetime.timedelta.__rmul__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__rmul__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__rmul__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____rsub____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____rsub____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____rsub____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__rsub__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__rsub__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__rsub__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____sub____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____sub____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____sub____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__sub__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__sub__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__sub__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timedelta____truediv____value_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timedelta____truediv____value_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timedelta____truediv____value_as_timedelta_wrong"
# subject = "datetime.timedelta.__truediv__(value: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timedelta.__truediv__(value: timedelta); call it with the wrong type.

typeshed contract: value is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timedelta
obj = object.__new__(timedelta)
try:
    obj.__truediv__(_W())  # value: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timezone____new____offset_as_timedelta_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timezone____new____offset_as_timedelta_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timezone____new____offset_as_timedelta_wrong"
# subject = "datetime.timezone.__new__(offset: timedelta)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timezone.__new__(offset: timedelta); call it with the wrong type.

typeshed contract: offset is timedelta. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timezone
obj = object.__new__(timezone)
try:
    obj.__new__(_W())  # offset: timedelta <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timezone__dst__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timezone__dst__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timezone__dst__dt_as_typed_wrong"
# subject = "datetime.timezone.dst(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timezone.dst(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timezone
obj = object.__new__(timezone)
try:
    obj.dst(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timezone__tzname__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timezone__tzname__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timezone__tzname__dt_as_typed_wrong"
# subject = "datetime.timezone.tzname(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timezone.tzname(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timezone
obj = object.__new__(timezone)
try:
    obj.tzname(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/timezone__utcoffset__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_timezone__utcoffset__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "timezone__utcoffset__dt_as_typed_wrong"
# subject = "datetime.timezone.utcoffset(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.timezone.utcoffset(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import timezone
obj = object.__new__(timezone)
try:
    obj.utcoffset(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/tzinfo__dst__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_tzinfo__dst__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "tzinfo__dst__dt_as_typed_wrong"
# subject = "datetime.tzinfo.dst(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.tzinfo.dst(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import tzinfo
obj = object.__new__(tzinfo)
try:
    obj.dst(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/tzinfo__fromutc__dt_as_datetime_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_tzinfo__fromutc__dt_as_datetime_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "tzinfo__fromutc__dt_as_datetime_wrong"
# subject = "datetime.tzinfo.fromutc(dt: datetime)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.tzinfo.fromutc(dt: datetime); call it with the wrong type.

typeshed contract: dt is datetime. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import tzinfo
obj = object.__new__(tzinfo)
try:
    obj.fromutc(_W())  # dt: datetime <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/tzinfo__tzname__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_tzinfo__tzname__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "tzinfo__tzname__dt_as_typed_wrong"
# subject = "datetime.tzinfo.tzname(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.tzinfo.tzname(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import tzinfo
obj = object.__new__(tzinfo)
try:
    obj.tzname(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/datetime/tzinfo__utcoffset__dt_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_datetime_tzinfo__utcoffset__dt_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "tzinfo__utcoffset__dt_as_typed_wrong"
# subject = "datetime.tzinfo.utcoffset(dt: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.tzinfo.utcoffset(dt: typed); call it with the wrong type.

typeshed contract: dt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import tzinfo
obj = object.__new__(tzinfo)
try:
    obj.utcoffset(_W())  # dt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
