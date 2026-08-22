use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/datetime/abstract_tzinfo_methods_raise.py`.
#[test]
fn test_gen_errors_std_libs_datetime_abstract_tzinfo_methods_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "abstract_tzinfo_methods_raise"
# subject = "datetime.tzinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.tzinfo: a bare tzinfo() is abstract: its tzname/utcoffset/dst query methods each raise NotImplementedError"""
import datetime

useless = datetime.tzinfo()
sample = datetime.datetime(2010, 1, 1)
for method in ("tzname", "utcoffset", "dst"):
    _raised = False
    try:
        getattr(useless, method)(sample)
    except NotImplementedError:
        _raised = True
    assert _raised, f"tzinfo.{method}: expected NotImplementedError"
print("abstract_tzinfo_methods_raise OK")
"###);
    assert_output(&out, r###"abstract_tzinfo_methods_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/bad_timespec_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_bad_timespec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "bad_timespec_raises"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: time.isoformat with an unknown timespec keyword raises ValueError"""
import datetime

full = datetime.time(12, 34, 56, 123456)
_raised = False
try:
    full.isoformat(timespec="monkey")
except ValueError:
    _raised = True
assert _raised, "bad_timespec: expected ValueError"
print("bad_timespec_raises OK")
"###);
    assert_output(&out, r###"bad_timespec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/date_day_30_february_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_date_day_30_february_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "date_day_30_february_raises"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date_day_30_february_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date(2024, 2, 30)
except ValueError:
    _raised = True
assert _raised, "date_day_30_february_raises: expected ValueError"
print("date_day_30_february_raises OK")
"###);
    assert_output(&out, r###"date_day_30_february_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/date_month_13_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_date_month_13_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "date_month_13_raises"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date_month_13_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date(2024, 13, 1)
except ValueError:
    _raised = True
assert _raised, "date_month_13_raises: expected ValueError"
print("date_month_13_raises OK")
"###);
    assert_output(&out, r###"date_month_13_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/datetime_plus_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_datetime_plus_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "datetime_plus_int_raises"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime + int, int + datetime, datetime * timedelta, and datetime + datetime are each rejected with TypeError"""
import datetime

a = datetime.datetime(2002, 3, 2, 17, 6)
for expr in (lambda: a + 1, lambda: 1 + a,
             lambda: a * datetime.timedelta(1), lambda: a + a):
    _raised = False
    try:
        expr()
    except TypeError:
        _raised = True
    assert _raised, "datetime_plus_int_raises: expected TypeError"
print("datetime_plus_int_raises OK")
"###);
    assert_output(&out, r###"datetime_plus_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/fromisoformat_garbage_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_fromisoformat_garbage_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "fromisoformat_garbage_raises"
# subject = "datetime.date.fromisoformat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date.fromisoformat: fromisoformat_garbage_raises (errors)."""
import datetime

_raised = False
try:
    datetime.date.fromisoformat("garbage")
except ValueError:
    _raised = True
assert _raised, "fromisoformat_garbage_raises: expected ValueError"
print("fromisoformat_garbage_raises OK")
"###);
    assert_output(&out, r###"fromisoformat_garbage_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/malformed_isoformat_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_malformed_isoformat_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "malformed_isoformat_raises"
# subject = "datetime.datetime.fromisoformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime.fromisoformat: datetime.fromisoformat rejects each of a set of malformed ISO strings with ValueError rather than returning silent None"""
import datetime

for bad in ("", "009-03-04", "200a-12-04", "2009-01-32", "2009-02-29",
            "2020-W25-0", "2020-W25-8"):
    _raised = False
    try:
        datetime.datetime.fromisoformat(bad)
    except ValueError:
        _raised = True
    assert _raised, f"bad iso {bad!r}: expected ValueError"
print("malformed_isoformat_raises OK")
"###);
    assert_output(&out, r###"malformed_isoformat_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/naive_aware_compare_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_naive_aware_compare_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "naive_aware_compare_raises"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: comparing a naive datetime against an aware one with `<` raises TypeError (CPython refuses to order across the naive/aware boundary)"""
import datetime

naive = datetime.datetime(2024, 1, 1)
aware = datetime.datetime(2024, 1, 1, tzinfo=datetime.timezone.utc)
_raised = False
try:
    _ = naive < aware
except TypeError:
    _raised = True
assert _raised, "naive_aware_compare_raises: expected TypeError"
print("naive_aware_compare_raises OK")
"###);
    assert_output(&out, r###"naive_aware_compare_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/out_of_range_offset_format_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_out_of_range_offset_format_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "out_of_range_offset_format_raises"
# subject = "datetime.tzinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.tzinfo: a custom tzinfo whose utcoffset is +/-1439 minutes formats fine, but a 1440-minute offset is rejected with ValueError at format time"""
import datetime

class Edgy(datetime.tzinfo):
    def __init__(self, minutes):
        self.offset = datetime.timedelta(minutes=minutes)
    def utcoffset(self, dt):
        return self.offset

# +/-1439 minutes is the legal boundary; it formats fine.
ok = datetime.time(1, 2, 3, tzinfo=Edgy(1439))
assert str(ok) == "01:02:03+23:59", f"edge offset str = {str(ok)!r}"

# 1440 minutes is out of range and is rejected at format time.
_raised = False
try:
    str(datetime.time(1, 2, 3, tzinfo=Edgy(1440)))
except ValueError:
    _raised = True
assert _raised, "offset_1440: expected ValueError"
print("out_of_range_offset_format_raises OK")
"###);
    assert_output(&out, r###"out_of_range_offset_format_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/strptime_mismatch_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_strptime_mismatch_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "strptime_mismatch_raises"
# subject = "datetime.datetime.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.strptime: strptime_mismatch_raises (errors)."""
import datetime

_raised = False
try:
    datetime.datetime.strptime("not_a_date", "%Y-%m-%d")
except ValueError:
    _raised = True
assert _raised, "strptime_mismatch_raises: expected ValueError"
print("strptime_mismatch_raises OK")
"###);
    assert_output(&out, r###"strptime_mismatch_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/time_hour_24_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_time_hour_24_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "time_hour_24_raises"
# subject = "datetime.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.time: time_hour_24_raises (errors)."""
import datetime

_raised = False
try:
    datetime.time(24, 0, 0)
except ValueError:
    _raised = True
assert _raised, "time_hour_24_raises: expected ValueError"
print("time_hour_24_raises OK")
"###);
    assert_output(&out, r###"time_hour_24_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timedelta_bounds_overflow_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timedelta_bounds_overflow_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_bounds_overflow_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: stepping one resolution past timedelta.min/.max, negating timedelta.max, and scaling a day beyond range (int * and tiny /) each raise OverflowError"""
import datetime

tiny = datetime.timedelta.resolution

# Stepping one resolution past min / max overflows.
_raised = False
try:
    datetime.timedelta.min.__sub__(tiny)
except OverflowError:
    _raised = True
assert _raised, "min - tiny: expected OverflowError"
_raised = False
try:
    datetime.timedelta.max.__add__(tiny)
except OverflowError:
    _raised = True
assert _raised, "max + tiny: expected OverflowError"

# Negating the extreme positive delta overflows the negative range.
_raised = False
try:
    -datetime.timedelta.max
except OverflowError:
    _raised = True
assert _raised, "neg max: expected OverflowError"

# Scaling a single day beyond range overflows (int * and tiny /).
day = datetime.timedelta(1)
_raised = False
try:
    day.__mul__(10 ** 9)
except OverflowError:
    _raised = True
assert _raised, "day * 1e9: expected OverflowError"
_raised = False
try:
    day.__truediv__(1e-20)
except OverflowError:
    _raised = True
assert _raised, "day / 1e-20: expected OverflowError"
print("timedelta_bounds_overflow_raises OK")
"###);
    assert_output(&out, r###"timedelta_bounds_overflow_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timedelta_div_by_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timedelta_div_by_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_div_by_zero_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: true-div, floor-div, and modulo of a timedelta by a zero timedelta each raise ZeroDivisionError"""
import datetime

from operator import truediv, floordiv, mod

t = datetime.timedelta(minutes=2, seconds=30)
zero = datetime.timedelta(0)
for op in (truediv, floordiv, mod):
    _raised = False
    try:
        op(t, zero)
    except ZeroDivisionError:
        _raised = True
    assert _raised, f"{op.__name__} by zero: expected ZeroDivisionError"
print("timedelta_div_by_zero_raises OK")
"###);
    assert_output(&out, r###"timedelta_div_by_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timedelta_divmod_by_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timedelta_divmod_by_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_divmod_by_int_raises"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: divmod(timedelta, int) is unsupported (TypeError) even though timedelta / int and timedelta // int are valid"""
import datetime

t = datetime.timedelta(minutes=2, seconds=30)
# timedelta / int is valid (scales the delta).
assert t / 10 == datetime.timedelta(seconds=15), f"td / int = {t / 10!r}"
# divmod against a plain int is unsupported.
_raised = False
try:
    divmod(t, 10)
except TypeError:
    _raised = True
assert _raised, "timedelta_divmod_by_int_raises: expected TypeError"
print("timedelta_divmod_by_int_raises OK")
"###);
    assert_output(&out, r###"timedelta_divmod_by_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timedelta_mixed_int_ops_raise.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timedelta_mixed_int_ops_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_mixed_int_ops_raise"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: timedelta +/- int (either operand order) raises TypeError; int // timedelta raises TypeError; timedelta // 0 and timedelta / 0.0 raise ZeroDivisionError"""
import datetime

a = datetime.timedelta(42)
for value in (1, 1.0):
    for expr in (lambda: a + value, lambda: value + a,
                 lambda: a - value, lambda: value - a):
        _raised = False
        try:
            expr()
        except TypeError:
            _raised = True
        assert _raised, f"mixed {value!r}: expected TypeError"

# int // timedelta is unsupported (TypeError).
_raised = False
try:
    0 // a
except TypeError:
    _raised = True
assert _raised, "int // timedelta: expected TypeError"

# timedelta // 0 and timedelta / 0.0 are ZeroDivisionError.
_raised = False
try:
    a // 0
except ZeroDivisionError:
    _raised = True
assert _raised, "td // 0: expected ZeroDivisionError"
_raised = False
try:
    a / 0.0
except ZeroDivisionError:
    _raised = True
assert _raised, "td / 0.0: expected ZeroDivisionError"
print("timedelta_mixed_int_ops_raise OK")
"###);
    assert_output(&out, r###"timedelta_mixed_int_ops_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timedelta_overflow_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timedelta_overflow_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_overflow_raises"
# subject = "datetime.timedelta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta_overflow_raises (errors)."""
import datetime

_raised = False
try:
    datetime.timedelta(days=10**10) + datetime.timedelta(days=10**10)
except OverflowError:
    _raised = True
assert _raised, "timedelta_overflow_raises: expected OverflowError"
print("timedelta_overflow_raises OK")
"###);
    assert_output(&out, r###"timedelta_overflow_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/datetime/timezone_dst_non_datetime_raises.py`.
#[test]
fn test_gen_errors_std_libs_datetime_timezone_dst_non_datetime_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timezone_dst_non_datetime_raises"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: timezone.dst() applied to a non-datetime argument (str / int) raises TypeError"""
import datetime

EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for bad in ("", 5):
    _raised = False
    try:
        EST.dst(bad)
    except TypeError:
        _raised = True
    assert _raised, f"dst({bad!r}): expected TypeError"
print("timezone_dst_non_datetime_raises OK")
"###);
    assert_output(&out, r###"timezone_dst_non_datetime_raises OK
"###);
}
