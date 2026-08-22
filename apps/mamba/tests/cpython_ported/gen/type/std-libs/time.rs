use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/time/asctime__time_tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_asctime__time_tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "asctime__time_tuple_as_typed_wrong"
# subject = "time.asctime(time_tuple: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.asctime(time_tuple: typed); call it with the wrong type.

typeshed contract: time_tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import asctime
try:
    asctime(_W())  # time_tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/clock_getres__clk_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_clock_getres__clk_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "clock_getres__clk_id_as_int_wrong"
# subject = "time.clock_getres(clk_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.clock_getres(clk_id: int); call it with the wrong type.

typeshed contract: clk_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import clock_getres
try:
    clock_getres("not_an_int")  # clk_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/clock_gettime__clk_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_clock_gettime__clk_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "clock_gettime__clk_id_as_int_wrong"
# subject = "time.clock_gettime(clk_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.clock_gettime(clk_id: int); call it with the wrong type.

typeshed contract: clk_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import clock_gettime
try:
    clock_gettime("not_an_int")  # clk_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/clock_gettime_ns__clk_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_clock_gettime_ns__clk_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "clock_gettime_ns__clk_id_as_int_wrong"
# subject = "time.clock_gettime_ns(clk_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.clock_gettime_ns(clk_id: int); call it with the wrong type.

typeshed contract: clk_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import clock_gettime_ns
try:
    clock_gettime_ns("not_an_int")  # clk_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/clock_settime__clk_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_clock_settime__clk_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "clock_settime__clk_id_as_int_wrong"
# subject = "time.clock_settime(clk_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.clock_settime(clk_id: int); call it with the wrong type.

typeshed contract: clk_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import clock_settime
try:
    clock_settime("not_an_int", 0.0)  # clk_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/clock_settime_ns__clock_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_clock_settime_ns__clock_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "clock_settime_ns__clock_id_as_int_wrong"
# subject = "time.clock_settime_ns(clock_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.clock_settime_ns(clock_id: int); call it with the wrong type.

typeshed contract: clock_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import clock_settime_ns
try:
    clock_settime_ns("not_an_int", 0)  # clock_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/ctime__seconds_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_ctime__seconds_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "ctime__seconds_as_typed_wrong"
# subject = "time.ctime(seconds: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.ctime(seconds: typed); call it with the wrong type.

typeshed contract: seconds is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import ctime
try:
    ctime(_W())  # seconds: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/get_clock_info__name_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_get_clock_info__name_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "get_clock_info__name_as_Literal_wrong"
# subject = "time.get_clock_info(name: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.get_clock_info(name: Literal); call it with the wrong type.

typeshed contract: name is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import get_clock_info
try:
    get_clock_info(_W())  # name: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/gmtime__seconds_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_gmtime__seconds_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "gmtime__seconds_as_typed_wrong"
# subject = "time.gmtime(seconds: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.gmtime(seconds: typed); call it with the wrong type.

typeshed contract: seconds is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import gmtime
try:
    gmtime(_W())  # seconds: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/localtime__seconds_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_localtime__seconds_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "localtime__seconds_as_typed_wrong"
# subject = "time.localtime(seconds: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.localtime(seconds: typed); call it with the wrong type.

typeshed contract: seconds is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import localtime
try:
    localtime(_W())  # seconds: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/mktime__time_tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_mktime__time_tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "mktime__time_tuple_as_typed_wrong"
# subject = "time.mktime(time_tuple: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.mktime(time_tuple: typed); call it with the wrong type.

typeshed contract: time_tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import mktime
try:
    mktime(_W())  # time_tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/pthread_getcpuclockid__thread_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_pthread_getcpuclockid__thread_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "pthread_getcpuclockid__thread_id_as_int_wrong"
# subject = "time.pthread_getcpuclockid(thread_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.pthread_getcpuclockid(thread_id: int); call it with the wrong type.

typeshed contract: thread_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import pthread_getcpuclockid
try:
    pthread_getcpuclockid("not_an_int")  # thread_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/sleep__seconds_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_sleep__seconds_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "sleep__seconds_as__SupportsFloatOrIndex_wrong"
# subject = "time.sleep(seconds: _SupportsFloatOrIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.sleep(seconds: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: seconds is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import sleep
try:
    sleep(_W())  # seconds: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/strftime__format_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_strftime__format_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "strftime__format_as_str_wrong"
# subject = "time.strftime(format: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.strftime(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import strftime
try:
    strftime(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/time/strptime__data_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_time_strptime__data_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "strptime__data_string_as_str_wrong"
# subject = "time.strptime(data_string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.strptime(data_string: str); call it with the wrong type.

typeshed contract: data_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from time import strptime
try:
    strptime(12345)  # data_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
