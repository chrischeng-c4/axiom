use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/time/ctime_asctime_mktime_consistent.py`.
#[test]
fn test_gen_behavior_std_libs_time_ctime_asctime_mktime_consistent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_asctime_mktime_consistent"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: for 'now', ctime(t) == asctime(localtime(t)) and int(mktime(localtime(t))) round-trips to int(t)"""
import time

_now = time.time()
assert time.ctime(_now) == time.asctime(time.localtime(_now)), \
    "ctime == asctime(localtime)"
assert int(time.mktime(time.localtime(_now))) == int(_now), \
    "mktime(localtime) round-trips to the same integer second"
print("ctime_asctime_mktime_consistent OK")
"###);
    assert_output(&out, r###"ctime_asctime_mktime_consistent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/ctime_canonical_form.py`.
#[test]
fn test_gen_behavior_std_libs_time_ctime_canonical_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_canonical_form"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: ctime renders a fixed timestamp in the 24-char canonical form: 1973-09-16 01:03:52 -> 'Sun Sep 16 01:03:52 1973' and 2000-01-01 -> 'Sat Jan  1 00:00:00 2000' (struct built with tm_isdst=-1 so mktime resolves DST)"""
import time

# Use tm_isdst=-1 so mktime picks the right DST for the local zone.
_c1 = time.ctime(time.mktime((1973, 9, 16, 1, 3, 52, 0, 0, -1)))
assert _c1 == "Sun Sep 16 01:03:52 1973", f"ctime 1973 = {_c1!r}"
_c2 = time.ctime(time.mktime((2000, 1, 1, 0, 0, 0, 0, 0, -1)))
assert _c2 == "Sat Jan  1 00:00:00 2000", f"ctime 2000 = {_c2!r}"
print("ctime_canonical_form OK")
"###);
    assert_output(&out, r###"ctime_canonical_form OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/ctime_now_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_time_ctime_now_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_now_returns_str"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: ctime() and ctime(None) both use the current time and return a str without raising"""
import time

assert isinstance(time.ctime(), str), "ctime() returns str"
assert isinstance(time.ctime(None), str), "ctime(None) returns str"
print("ctime_now_returns_str OK")
"###);
    assert_output(&out, r###"ctime_now_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/get_clock_info_flags.py`.
#[test]
fn test_gen_behavior_std_libs_time_get_clock_info_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "get_clock_info_flags"
# subject = "time.get_clock_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.get_clock_info: get_clock_info reports the right monotonic/adjustable flags: 'monotonic' is monotonic & not adjustable, 'time' is not monotonic & adjustable, 'process_time' is monotonic & not adjustable"""
import time

_mono = time.get_clock_info("monotonic")
assert _mono.monotonic is True, "monotonic clock is monotonic"
assert _mono.adjustable is False, "monotonic clock is not adjustable"
_wall = time.get_clock_info("time")
assert _wall.monotonic is False, "wall clock is not monotonic"
assert _wall.adjustable is True, "wall clock is adjustable"
_proc = time.get_clock_info("process_time")
assert _proc.monotonic is True, "process_time is monotonic"
assert _proc.adjustable is False, "process_time is not adjustable"
print("get_clock_info_flags OK")
"###);
    assert_output(&out, r###"get_clock_info_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/gmtime_epoch_components.py`.
#[test]
fn test_gen_behavior_std_libs_time_gmtime_epoch_components() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_epoch_components"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: time.gmtime(0) is the unix epoch 1970-01-01 00:00:00 UTC: year/mon/mday/hour/min/sec and tm_wday=3 (Thu) tm_yday=1"""
import time

_epoch = time.gmtime(0)
assert _epoch.tm_year == 1970, f"epoch year = {_epoch.tm_year!r}"
assert _epoch.tm_mon == 1, f"epoch month = {_epoch.tm_mon!r}"
assert _epoch.tm_mday == 1, f"epoch mday = {_epoch.tm_mday!r}"
assert _epoch.tm_hour == 0, f"epoch hour = {_epoch.tm_hour!r}"
assert _epoch.tm_min == 0, f"epoch min = {_epoch.tm_min!r}"
assert _epoch.tm_sec == 0, f"epoch sec = {_epoch.tm_sec!r}"
assert _epoch.tm_wday == 3, f"epoch wday (Thu=3) = {_epoch.tm_wday!r}"
assert _epoch.tm_yday == 1, f"epoch yday = {_epoch.tm_yday!r}"
print("gmtime_epoch_components OK")
"###);
    assert_output(&out, r###"gmtime_epoch_components OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/gmtime_epoch_tuple_slice.py`.
#[test]
fn test_gen_behavior_std_libs_time_gmtime_epoch_tuple_slice() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_epoch_tuple_slice"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: tuple(time.gmtime(0))[:6] equals (1970, 1, 1, 0, 0, 0) — struct_time is tuple-like by index and by slice"""
import time

assert tuple(time.gmtime(0))[:6] == (1970, 1, 1, 0, 0, 0), "epoch tuple slice"
print("gmtime_epoch_tuple_slice OK")
"###);
    assert_output(&out, r###"gmtime_epoch_tuple_slice OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/gmtime_none_means_now.py`.
#[test]
fn test_gen_behavior_std_libs_time_gmtime_none_means_now() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_none_means_now"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: gmtime() and gmtime(None) both mean 'now': their mktime values differ by less than one second"""
import time

assert abs(time.mktime(time.gmtime()) - time.mktime(time.gmtime(None))) < 1.0, \
    "gmtime() == gmtime(None)"
print("gmtime_none_means_now OK")
"###);
    assert_output(&out, r###"gmtime_none_means_now OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/gmtime_struct_fields_in_range.py`.
#[test]
fn test_gen_behavior_std_libs_time_gmtime_struct_fields_in_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_struct_fields_in_range"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.gmtime: gmtime of an arbitrary timestamp (1_700_000_000) yields a struct_time whose mon is 1..12, mday 1..31, hour 0..23"""
import time

_st = time.gmtime(1_700_000_000)
assert 1 <= _st.tm_mon <= 12, f"month in range: {_st.tm_mon!r}"
assert 1 <= _st.tm_mday <= 31, f"mday in range: {_st.tm_mday!r}"
assert 0 <= _st.tm_hour <= 23, f"hour in range: {_st.tm_hour!r}"
print("gmtime_struct_fields_in_range OK")
"###);
    assert_output(&out, r###"gmtime_struct_fields_in_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/localtime_mktime_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_time_localtime_mktime_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "localtime_mktime_roundtrip"
# subject = "time.localtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.localtime: localtime exposes tm_gmtoff and tm_zone; localtime(mktime(localtime(now))) round-trips equal and preserves tm_gmtoff and tm_zone"""
import time

_now = time.time()
_lt = time.localtime(_now)
assert hasattr(_lt, "tm_gmtoff"), "struct_time has tm_gmtoff"
assert hasattr(_lt, "tm_zone"), "struct_time has tm_zone"
_back = time.localtime(time.mktime(_lt))
assert _back == _lt, "localtime(mktime(localtime)) == localtime"
assert _back.tm_gmtoff == _lt.tm_gmtoff, "tm_gmtoff preserved"
assert _back.tm_zone == _lt.tm_zone, "tm_zone preserved"
print("localtime_mktime_roundtrip OK")
"###);
    assert_output(&out, r###"localtime_mktime_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/localtime_none_means_now.py`.
#[test]
fn test_gen_behavior_std_libs_time_localtime_none_means_now() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "localtime_none_means_now"
# subject = "time.localtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.localtime: localtime() and localtime(None) both mean 'now': their mktime values differ by less than one second"""
import time

assert abs(time.mktime(time.localtime()) - time.mktime(time.localtime(None))) < 1.0, \
    "localtime() == localtime(None)"
print("localtime_none_means_now OK")
"###);
    assert_output(&out, r###"localtime_none_means_now OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/monotonic_non_decreasing.py`.
#[test]
fn test_gen_behavior_std_libs_time_monotonic_non_decreasing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "monotonic_non_decreasing"
# subject = "time.monotonic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.monotonic: ten successive time.monotonic() reads are float and never go backward (each >= the previous)"""
import time

_readings = [time.monotonic() for _ in range(10)]
for r in _readings:
    assert isinstance(r, float), f"monotonic type = {type(r)!r}"
for outer in range(len(_readings)):
    for inner in range(outer + 1, len(_readings)):
        assert _readings[inner] >= _readings[outer], \
            f"monotonic went backward at {outer},{inner}"
print("monotonic_non_decreasing OK")
"###);
    assert_output(&out, r###"monotonic_non_decreasing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/perf_counter_advances.py`.
#[test]
fn test_gen_behavior_std_libs_time_perf_counter_advances() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "perf_counter_advances"
# subject = "time.perf_counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.perf_counter: time.perf_counter() is a float and strictly advances across a busy interval (two reads bracketing a 1000-iteration loop)"""
import time

_p1 = time.perf_counter()
assert isinstance(_p1, float), f"perf_counter type = {type(_p1)!r}"
for _ in range(1000):
    pass
_p2 = time.perf_counter()
assert _p2 > _p1, f"perf_counter advances: {_p1} {_p2}"
print("perf_counter_advances OK")
"###);
    assert_output(&out, r###"perf_counter_advances OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/process_time_non_negative_float.py`.
#[test]
fn test_gen_behavior_std_libs_time_process_time_non_negative_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "process_time_non_negative_float"
# subject = "time.process_time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.process_time: time.process_time() returns a non-negative float"""
import time

_pt = time.process_time()
assert isinstance(_pt, float), f"process_time type = {type(_pt)!r}"
assert _pt >= 0, f"process_time >= 0: {_pt!r}"
print("process_time_non_negative_float OK")
"###);
    assert_output(&out, r###"process_time_non_negative_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/strftime_epoch_datetime.py`.
#[test]
fn test_gen_behavior_std_libs_time_strftime_epoch_datetime() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_epoch_datetime"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strftime: strftime('%Y-%m-%d %H:%M:%S', gmtime(0)) renders the epoch as '1970-01-01 00:00:00'"""
import time

_fmt = time.strftime("%Y-%m-%d %H:%M:%S", time.gmtime(0))
assert _fmt == "1970-01-01 00:00:00", f"strftime = {_fmt!r}"
print("strftime_epoch_datetime OK")
"###);
    assert_output(&out, r###"strftime_epoch_datetime OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/strftime_julian_day.py`.
#[test]
fn test_gen_behavior_std_libs_time_strftime_julian_day() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_julian_day"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.strftime: %j is the zero-padded day-of-year: nine days after the epoch is Jan 10, tm_yday==10, strftime('%j') == '010'"""
import time

_jan10 = time.gmtime(9 * 86400)  # 9 days after epoch = Jan 10
assert _jan10.tm_yday == 10, f"yday = {_jan10.tm_yday!r}"
_jfmt = time.strftime("%j", _jan10)
assert _jfmt == "010", f"strftime %%j = {_jfmt!r}"
print("strftime_julian_day OK")
"###);
    assert_output(&out, r###"strftime_julian_day OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/strftime_zero_field_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_time_strftime_zero_field_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_zero_field_defaults"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strftime: strftime substitutes documented defaults for zero-valued fields: (2000,)+(0,)*8 with '%Y %m %d %H %M %S %w %j' yields '2000 01 01 00 00 00 1 001'"""
import time

# (2000,) + nine zeros -> year 2000, Jan 1, weekday Sat (%w=1), yday 001.
_zero = time.strftime("%Y %m %d %H %M %S %w %j", (2000,) + (0,) * 8)
assert _zero == "2000 01 01 00 00 00 1 001", f"zero-default strftime = {_zero!r}"
print("strftime_zero_field_defaults OK")
"###);
    assert_output(&out, r###"strftime_zero_field_defaults OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/strptime_parses_datetime.py`.
#[test]
fn test_gen_behavior_std_libs_time_strptime_parses_datetime() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strptime_parses_datetime"
# subject = "time.strptime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime('2023-06-15 12:30:00', '%Y-%m-%d %H:%M:%S') returns a struct_time with year=2023, mon=6, mday=15, hour=12"""
import time

_parsed = time.strptime("2023-06-15 12:30:00", "%Y-%m-%d %H:%M:%S")
assert isinstance(_parsed, time.struct_time), f"strptime type = {type(_parsed)!r}"
assert _parsed.tm_year == 2023, f"parsed year = {_parsed.tm_year!r}"
assert _parsed.tm_mon == 6, f"parsed month = {_parsed.tm_mon!r}"
assert _parsed.tm_mday == 15, f"parsed day = {_parsed.tm_mday!r}"
assert _parsed.tm_hour == 12, f"parsed hour = {_parsed.tm_hour!r}"
print("strptime_parses_datetime OK")
"###);
    assert_output(&out, r###"strptime_parses_datetime OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/strptime_zone_directives.py`.
#[test]
fn test_gen_behavior_std_libs_time_strptime_zone_directives() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strptime_zone_directives"
# subject = "time.strptime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime understands %Z (zone name -> tm_zone == 'UTC') and %z (offset -> tm_gmtoff == 5*3600)"""
import time

assert time.strptime("UTC", "%Z").tm_zone == "UTC", "strptime %Z -> tm_zone"
assert time.strptime("+0500", "%z").tm_gmtoff == 5 * 3600, "strptime %z -> tm_gmtoff"
print("strptime_zone_directives OK")
"###);
    assert_output(&out, r###"strptime_zone_directives OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/struct_time_index_access.py`.
#[test]
fn test_gen_behavior_std_libs_time_struct_time_index_access() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "struct_time_index_access"
# subject = "time.struct_time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.struct_time: struct_time supports integer index access mirroring the named fields: gmtime(0)[0]==1970, [1]==1, [5]==0"""
import time

_st = time.gmtime(0)
assert _st[0] == 1970, f"struct_time[0] = {_st[0]!r}"
assert _st[1] == 1, f"struct_time[1] = {_st[1]!r}"
assert _st[5] == 0, f"struct_time[5] = {_st[5]!r}"
print("struct_time_index_access OK")
"###);
    assert_output(&out, r###"struct_time_index_access OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/time_ns_returns_positive_int.py`.
#[test]
fn test_gen_behavior_std_libs_time_time_ns_returns_positive_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_ns_returns_positive_int"
# subject = "time.time_ns"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.time_ns: time.time_ns() returns an int of nanoseconds since the epoch, greater than zero, consistent with time.time() to within one second"""
import time

_ns = time.time_ns()
assert isinstance(_ns, int), f"time_ns type = {type(_ns)!r}"
assert _ns > 0, f"time_ns > 0: {_ns!r}"

# time_ns is close to time() * 1e9.
_t = time.time()
_diff = abs(_ns - _t * 1_000_000_000)
assert _diff < 1_000_000_000, f"time_ns ~ time*1e9, diff={_diff}"
print("time_ns_returns_positive_int OK")
"###);
    assert_output(&out, r###"time_ns_returns_positive_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/time_returns_positive_float.py`.
#[test]
fn test_gen_behavior_std_libs_time_time_returns_positive_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_returns_positive_float"
# subject = "time.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.time: time.time() returns a float of seconds since the unix epoch, greater than 1e9"""
import time

_t = time.time()
assert isinstance(_t, float), f"time() type = {type(_t)!r}"
assert _t > 1_000_000_000.0, f"time() > 1e9: {_t!r}"
print("time_returns_positive_float OK")
"###);
    assert_output(&out, r###"time_returns_positive_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/time/tzset_rereads_tz_env.py`.
#[test]
fn test_gen_behavior_std_libs_time_tzset_rereads_tz_env() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "tzset_rereads_tz_env"
# subject = "time.tzset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.tzset: tzset re-reads the TZ env var: UTC+0 makes gmtime==localtime with daylight/timezone 0 and tm_isdst 0; EST+05EDT makes gmtime!=localtime with tzname ('EST','EDT'), daylight 1, timezone 18000, altzone 14400, December tm_isdst 0; original TZ restored in finally"""
import os
import time

if not hasattr(time, "tzset"):
    # tzset is Unix-only; nothing to assert on platforms without it.
    print("tzset_rereads_tz_env OK (skipped: not available)")
    raise SystemExit(0)

# 2002-12-25 00:00:00 UTC — a fixed instant for reproducible checks.
XMAS_2002 = 1040774400.0
EASTERN = "EST+05EDT,M4.1.0,M10.5.0"
UTC = "UTC+0"

_saved_tz = os.environ.get("TZ")
try:
    # UTC: local time equals UTC, no DST, zero offset.
    os.environ["TZ"] = UTC
    time.tzset()
    assert time.gmtime(XMAS_2002) == time.localtime(XMAS_2002), "UTC: gmtime == localtime"
    assert time.daylight == 0, f"UTC daylight = {time.daylight!r}"
    assert time.timezone == 0, f"UTC timezone = {time.timezone!r}"
    assert time.localtime(XMAS_2002).tm_isdst == 0, "UTC: not in DST"

    # US Eastern: local time diverges from UTC, DST rules active.
    os.environ["TZ"] = EASTERN
    time.tzset()
    assert time.gmtime(XMAS_2002) != time.localtime(XMAS_2002), "EST: gmtime != localtime"
    assert time.tzname == ("EST", "EDT"), f"EST tzname = {time.tzname!r}"
    assert len(time.tzname) == 2, "tzname has 2 entries"
    assert time.daylight == 1, f"EST daylight = {time.daylight!r}"
    assert time.timezone == 18000, f"EST timezone = {time.timezone!r}"  # +5h
    assert time.altzone == 14400, f"EST altzone = {time.altzone!r}"     # +4h (DST)
    # December is standard time, not DST, in the northern hemisphere.
    assert time.localtime(XMAS_2002).tm_isdst == 0, "EST: December not in DST"
finally:
    if _saved_tz is not None:
        os.environ["TZ"] = _saved_tz
    elif "TZ" in os.environ:
        del os.environ["TZ"]
    time.tzset()
print("tzset_rereads_tz_env OK")
"###);
    assert_output(&out, r###"tzset_rereads_tz_env OK
"###);
}
