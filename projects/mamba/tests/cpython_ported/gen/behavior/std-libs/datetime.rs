use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/datetime/all_names_are_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_all_names_are_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "all_names_are_attributes"
# subject = "datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: every name in datetime.__all__ is a real module attribute"""
import datetime

for name in datetime.__all__:
    assert hasattr(datetime, name), f"__all__ name missing: {name!r}"
print("all_names_are_attributes OK")
"###);
    assert_output(&out, r###"all_names_are_attributes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/aware_datetime_isoformat_offset.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_aware_datetime_isoformat_offset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "aware_datetime_isoformat_offset"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: a datetime built with tzinfo=timezone.utc keeps that tzinfo identity and renders '+00:00' in isoformat()"""
import datetime

utc_dt = datetime.datetime(2023, 6, 15, 12, 0, 0, tzinfo=datetime.timezone.utc)
assert utc_dt.tzinfo is datetime.timezone.utc, "tzinfo set"
tz_str = utc_dt.isoformat()
assert "+00:00" in tz_str, f"UTC isoformat = {tz_str!r}"
print("aware_datetime_isoformat_offset OK")
"###);
    assert_output(&out, r###"aware_datetime_isoformat_offset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/aware_timestamp_offset.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_aware_timestamp_offset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "aware_timestamp_offset"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: an aware datetime's POSIX timestamp accounts for the offset: 1970-01-01 UTC is 0.0, and an EST-5 instant adds the 5h offset plus its sub-second part"""
import datetime

t = datetime.datetime(1970, 1, 1, tzinfo=datetime.timezone.utc)
assert t.timestamp() == 0.0, f"epoch timestamp = {t.timestamp()!r}"
est = datetime.datetime(1970, 1, 1, 1, 2, 3, 4,
                        tzinfo=datetime.timezone(datetime.timedelta(hours=-5)))
assert est.timestamp() == 18000 + 3600 + 2 * 60 + 3 + 4e-06, f"EST ts = {est.timestamp()!r}"
print("aware_timestamp_offset OK")
"###);
    assert_output(&out, r###"aware_timestamp_offset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/combine_date_and_time.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_combine_date_and_time() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "combine_date_and_time"
# subject = "datetime.datetime.combine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.combine: datetime.combine(date, time) builds the combined instant whose .date() and .time() recover the parts"""
import datetime

d = datetime.date(2025, 3, 15)
t = datetime.time(10, 30, 45)
combined = datetime.datetime.combine(d, t)
assert combined == datetime.datetime(2025, 3, 15, 10, 30, 45), f"combine = {combined!r}"
assert combined.date() == d, "combined.date()"
assert combined.time() == t, "combined.time()"
print("combine_date_and_time OK")
"###);
    assert_output(&out, r###"combine_date_and_time OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/ctime_format.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_ctime_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "ctime_format"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime(2002,3,2).ctime() == 'Sat Mar  2 00:00:00 2002' (note the space-padded day)"""
import datetime

assert datetime.datetime(2002, 3, 2).ctime() == "Sat Mar  2 00:00:00 2002", "ctime"
print("ctime_format OK")
"###);
    assert_output(&out, r###"ctime_format OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_comparison_and_equality.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_comparison_and_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_comparison_and_equality"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: dates order chronologically (<) and compare by value: equal dates are ==, different days are !="""
import datetime

assert datetime.date(2023, 1, 1) < datetime.date(2023, 12, 31), "date <"
assert datetime.date(2023, 6, 15) == datetime.date(2023, 6, 15), "date =="
assert datetime.date(2023, 6, 15) != datetime.date(2023, 6, 16), "date !="
print("date_comparison_and_equality OK")
"###);
    assert_output(&out, r###"date_comparison_and_equality OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_difference_is_timedelta.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_difference_is_timedelta() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_difference_is_timedelta"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: subtracting two dates yields a timedelta; Dec31 2023 - Jan1 2023 is 364 days"""
import datetime

diff = datetime.date(2023, 12, 31) - datetime.date(2023, 1, 1)
assert isinstance(diff, datetime.timedelta), f"diff type = {type(diff)!r}"
assert diff.days == 364, f"diff days = {diff.days!r}"
print("date_difference_is_timedelta OK")
"###);
    assert_output(&out, r###"date_difference_is_timedelta OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_field_accessors.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_field_accessors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_field_accessors"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date(2025,3,15) exposes .year/.month/.day; a second date(2000,1,2) confirms the accessors are per-instance"""
import datetime

d = datetime.date(2025, 3, 15)
assert d.year == 2025, f"year = {d.year!r}"
assert d.month == 3, f"month = {d.month!r}"
assert d.day == 15, f"day = {d.day!r}"
d2 = datetime.date(2000, 1, 2)
assert d2.year == 2000, f"d2 year = {d2.year!r}"
assert d2.month == 1, f"d2 month = {d2.month!r}"
assert d2.day == 2, f"d2 day = {d2.day!r}"
print("date_field_accessors OK")
"###);
    assert_output(&out, r###"date_field_accessors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_isoformat_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_isoformat_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_isoformat_roundtrip"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.isoformat() yields YYYY-MM-DD and date.fromisoformat round-trips it back"""
import datetime

d = datetime.date(2023, 6, 15)
assert d.isoformat() == "2023-06-15", f"date iso = {d.isoformat()!r}"
assert datetime.date.fromisoformat(d.isoformat()) == d, "date iso round-trip"
print("date_isoformat_roundtrip OK")
"###);
    assert_output(&out, r###"date_isoformat_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_isoweekday_monday_one.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_isoweekday_monday_one() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_isoweekday_monday_one"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.isoweekday() is Monday=1..Sunday=7 (Jan2 2023 Mon=1, Jan8 Sun=7)"""
import datetime

assert datetime.date(2023, 1, 2).isoweekday() == 1, "Mon=1"
assert datetime.date(2023, 1, 8).isoweekday() == 7, "Sun=7"
print("date_isoweekday_monday_one OK")
"###);
    assert_output(&out, r###"date_isoweekday_monday_one OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_timedelta_add_subtract.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_timedelta_add_subtract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_timedelta_add_subtract"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date + timedelta rolls into the next month (Jan31+1=Feb1) and date - timedelta rolls back (Mar1-1=Feb28 in non-leap 2023)"""
import datetime

plus = datetime.date(2023, 1, 31) + datetime.timedelta(days=1)
assert plus == datetime.date(2023, 2, 1), f"jan31 + 1 = {plus!r}"
# 2023 is not a leap year, so Mar1 - 1 day is Feb 28.
minus = datetime.date(2023, 3, 1) - datetime.timedelta(days=1)
assert minus == datetime.date(2023, 2, 28), f"mar1 - 1 = {minus!r}"
print("date_timedelta_add_subtract OK")
"###);
    assert_output(&out, r###"date_timedelta_add_subtract OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_timetuple_struct_time.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_timetuple_struct_time() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_timetuple_struct_time"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.timetuple() returns a time.struct_time whose tm_year/tm_mon/tm_mday match the date"""
import datetime

import time as _time

d = datetime.date(2023, 11, 5)
tt = d.timetuple()
assert isinstance(tt, _time.struct_time), f"timetuple type = {type(tt)!r}"
assert tt.tm_year == 2023, f"tt year = {tt.tm_year!r}"
assert tt.tm_mon == 11, f"tt mon = {tt.tm_mon!r}"
assert tt.tm_mday == 5, f"tt mday = {tt.tm_mday!r}"
print("date_timetuple_struct_time OK")
"###);
    assert_output(&out, r###"date_timetuple_struct_time OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/date_weekday_monday_zero.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_date_weekday_monday_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_weekday_monday_zero"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.weekday() is Monday=0..Sunday=6 (Jan2 2023 Mon=0, Jan7 Sat=5, Jan8 Sun=6)"""
import datetime

assert datetime.date(2023, 1, 2).weekday() == 0, "Mon=0"
assert datetime.date(2023, 1, 7).weekday() == 5, "Sat=5"
assert datetime.date(2023, 1, 8).weekday() == 6, "Sun=6"
print("date_weekday_monday_zero OK")
"###);
    assert_output(&out, r###"date_weekday_monday_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/datetime_field_accessors.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_datetime_field_accessors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_field_accessors"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime(2025,3,15,10,20,30) exposes year/month/day and hour/minute/second; a second instance confirms per-instance fields"""
import datetime

dt = datetime.datetime(2025, 3, 15, 10, 20, 30)
assert (dt.year, dt.month, dt.day) == (2025, 3, 15), f"date part = {dt!r}"
assert (dt.hour, dt.minute, dt.second) == (10, 20, 30), f"time part = {dt!r}"
dt2 = datetime.datetime(1999, 12, 31, 23, 59, 59)
assert (dt2.year, dt2.month, dt2.day) == (1999, 12, 31), f"dt2 date = {dt2!r}"
assert (dt2.hour, dt2.minute, dt2.second) == (23, 59, 59), f"dt2 time = {dt2!r}"
print("datetime_field_accessors OK")
"###);
    assert_output(&out, r###"datetime_field_accessors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/datetime_isoformat_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_datetime_isoformat_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_isoformat_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime.isoformat() yields YYYY-MM-DDTHH:MM:SS, fromisoformat round-trips it, and sep=' ' swaps the date/time separator"""
import datetime

dt = datetime.datetime(2023, 6, 15, 13, 45, 30)
assert dt.isoformat() == "2023-06-15T13:45:30", f"dt iso = {dt.isoformat()!r}"
assert datetime.datetime.fromisoformat(dt.isoformat()) == dt, "dt iso round-trip"
assert dt.isoformat(sep=" ") == "2023-06-15 13:45:30", f"sep iso = {dt.isoformat(sep=' ')!r}"
print("datetime_isoformat_roundtrip OK")
"###);
    assert_output(&out, r###"datetime_isoformat_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/datetime_replace_is_immutable.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_datetime_replace_is_immutable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_replace_is_immutable"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime.replace() returns a new datetime with changed fields and leaves the original unchanged"""
import datetime

dt2 = datetime.datetime(2023, 6, 15, 12, 0, 0)
dt3 = dt2.replace(hour=18, minute=30)
assert dt3 == datetime.datetime(2023, 6, 15, 18, 30, 0), f"replace = {dt3!r}"
assert dt2 == datetime.datetime(2023, 6, 15, 12, 0, 0), "original unchanged"
print("datetime_replace_is_immutable OK")
"###);
    assert_output(&out, r###"datetime_replace_is_immutable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/datetime_timedelta_instant_math.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_datetime_timedelta_instant_math() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_timedelta_instant_math"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: adding/subtracting timedeltas lands on exact instants: a+hour==hour+a, a-hour==a+-hour, a+week, and a-(a+week)==-week"""
import datetime

a = datetime.datetime(2002, 3, 2, 17, 6)
hour = datetime.timedelta(0, 3600)
assert a + hour == datetime.datetime(2002, 3, 2, 18, 6), "a + hour"
assert hour + a == datetime.datetime(2002, 3, 2, 18, 6), "hour + a commutes"
assert a - hour == a + -hour, "subtract == add negative"
assert a + datetime.timedelta(7) == datetime.datetime(2002, 3, 9, 17, 6), "a + week"
assert a - (a + datetime.timedelta(7)) == -datetime.timedelta(7), "datetime difference"
print("datetime_timedelta_instant_math OK")
"###);
    assert_output(&out, r###"datetime_timedelta_instant_math OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/fold_does_not_affect_hash.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_fold_does_not_affect_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_does_not_affect_hash"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: fold does not affect hashing: hash(t) == hash(t.replace(fold=1)) for both time and datetime"""
import datetime

t = datetime.time(0)
assert hash(t) == hash(t.replace(fold=1)), "time fold hash stable"
dt = datetime.datetime(1, 1, 1)
assert hash(dt) == hash(dt.replace(fold=1)), "datetime fold hash stable"
print("fold_does_not_affect_hash OK")
"###);
    assert_output(&out, r###"fold_does_not_affect_hash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/fold_in_repr_only_when_set.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_fold_in_repr_only_when_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_in_repr_only_when_set"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: fold appears in repr only when set to 1: repr(time(fold=1)) and repr(datetime(...,fold=1)) include 'fold=1'"""
import datetime

assert repr(datetime.time(fold=1)) == "datetime.time(0, 0, fold=1)", \
    f"time fold repr = {repr(datetime.time(fold=1))!r}"
assert repr(datetime.datetime(1, 1, 1, fold=1)) == \
    "datetime.datetime(1, 1, 1, 0, 0, fold=1)", "datetime fold repr"
print("fold_in_repr_only_when_set OK")
"###);
    assert_output(&out, r###"fold_in_repr_only_when_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/fold_propagates_through_projections.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_fold_propagates_through_projections() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fold_propagates_through_projections"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: fold propagates through .time() and .timetz() projections of a datetime built with fold=1"""
import datetime

dtf = datetime.datetime(1, 1, 1, fold=1)
assert dtf.time().fold == 1, "fold via time()"
assert dtf.timetz().fold == 1, "fold via timetz()"
print("fold_propagates_through_projections OK")
"###);
    assert_output(&out, r###"fold_propagates_through_projections OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/fromisoformat_utc_suffix.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_fromisoformat_utc_suffix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "fromisoformat_utc_suffix"
# subject = "datetime.datetime.fromisoformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime.fromisoformat: datetime.fromisoformat('...+00:00') resolves the tzinfo to timezone.utc"""
import datetime

dt = datetime.datetime.fromisoformat("2014-04-19T13:21:13+00:00")
assert dt.tzinfo is datetime.timezone.utc, f"utc suffix = {dt.tzinfo!r}"
print("fromisoformat_utc_suffix OK")
"###);
    assert_output(&out, r###"fromisoformat_utc_suffix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/isocalendar_triple.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_isocalendar_triple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "isocalendar_triple"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime(2019,1,1).isocalendar() == (2019, 1, 2) — the (ISO year, week, weekday) triple"""
import datetime

assert datetime.datetime(2019, 1, 1).isocalendar() == (2019, 1, 2), "isocalendar"
print("isocalendar_triple OK")
"###);
    assert_output(&out, r###"isocalendar_triple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/isoformat_timespec_precision.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_isoformat_timespec_precision() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "isoformat_timespec_precision"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: isoformat(timespec=...) controls trailing precision (hours/minutes/seconds/milliseconds/microseconds/auto), and auto drops the fraction when microseconds are zero"""
import datetime

full = datetime.time(12, 34, 56, 123456)
assert full.isoformat(timespec="hours") == "12", "timespec hours"
assert full.isoformat(timespec="minutes") == "12:34", "timespec minutes"
assert full.isoformat(timespec="seconds") == "12:34:56", "timespec seconds"
assert full.isoformat(timespec="milliseconds") == "12:34:56.123", "timespec millis"
assert full.isoformat(timespec="microseconds") == "12:34:56.123456", "timespec micros"
assert full.isoformat(timespec="auto") == "12:34:56.123456", "timespec auto"

# auto drops the fractional part when microseconds are zero.
assert datetime.time(12, 34, 56).isoformat(timespec="auto") == "12:34:56", "auto no-frac"
assert datetime.time(12, 34, 56).isoformat(timespec="milliseconds") == "12:34:56.000", "millis zero-pad"
print("isoformat_timespec_precision OK")
"###);
    assert_output(&out, r###"isoformat_timespec_precision OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/module_constants_and_utc_alias.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_module_constants_and_utc_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "module_constants_and_utc_alias"
# subject = "datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: datetime.MINYEAR==1, datetime.MAXYEAR==9999, and datetime.UTC is timezone.utc"""
import datetime

assert datetime.MINYEAR == 1, f"MINYEAR = {datetime.MINYEAR!r}"
assert datetime.MAXYEAR == 9999, f"MAXYEAR = {datetime.MAXYEAR!r}"
assert datetime.UTC is datetime.timezone.utc, "UTC is timezone.utc"
print("module_constants_and_utc_alias OK")
"###);
    assert_output(&out, r###"module_constants_and_utc_alias OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/strftime_directives.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_strftime_directives() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "strftime_directives"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: strftime honors %Y/%m/%d on a date and %H:%M on a datetime"""
import datetime

d = datetime.date(2023, 6, 15)
assert d.strftime("%Y/%m/%d") == "2023/06/15", f"strftime date = {d.strftime('%Y/%m/%d')!r}"
dt = datetime.datetime(2023, 6, 15, 12, 30, 45)
assert dt.strftime("%H:%M") == "12:30", f"strftime time = {dt.strftime('%H:%M')!r}"
print("strftime_directives OK")
"###);
    assert_output(&out, r###"strftime_directives OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/strftime_strptime_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_strftime_strptime_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "strftime_strptime_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: strftime('%Y-%m-%d %H:%M:%S') renders the expected text and strptime parses it back to the same datetime"""
import datetime

dt = datetime.datetime(2023, 6, 15, 9, 5, 3)
s = dt.strftime("%Y-%m-%d %H:%M:%S")
assert s == "2023-06-15 09:05:03", f"strftime = {s!r}"
parsed = datetime.datetime.strptime(s, "%Y-%m-%d %H:%M:%S")
assert parsed == dt, f"strptime round-trip = {parsed!r}"
print("strftime_strptime_roundtrip OK")
"###);
    assert_output(&out, r###"strftime_strptime_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/time_isoformat_micros_padding.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_time_isoformat_micros_padding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "time_isoformat_micros_padding"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.time: time.isoformat() pads microseconds to six digits, equals str(time), and midnight prints '00:00:00'"""
import datetime

assert datetime.time(4, 5, 1, 123).isoformat() == "04:05:01.000123", "micros padded"
assert str(datetime.time(microsecond=10)) == "00:00:00.000010", "str == isoformat"
assert datetime.time().isoformat() == "00:00:00", "midnight isoformat"
print("time_isoformat_micros_padding OK")
"###);
    assert_output(&out, r###"time_isoformat_micros_padding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/time_isoformat_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_time_isoformat_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "time_isoformat_roundtrip"
# subject = "datetime.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.time: time(13,45,30,250000).isoformat() == '13:45:30.250000' and time.fromisoformat round-trips it"""
import datetime

t = datetime.time(13, 45, 30, 250000)
assert t.isoformat() == "13:45:30.250000", f"time iso = {t.isoformat()!r}"
assert datetime.time.fromisoformat(t.isoformat()) == t, "time iso round-trip"
print("time_isoformat_roundtrip OK")
"###);
    assert_output(&out, r###"time_isoformat_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_days_field.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_days_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_days_field"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=N).days returns N for N in {7,3,100,0}"""
import datetime

for days in (7, 3, 100, 0):
    assert datetime.timedelta(days=days).days == days, f"days={days!r}"
print("timedelta_days_field OK")
"###);
    assert_output(&out, r###"timedelta_days_field OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_div_and_floordiv.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_div_and_floordiv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_div_and_floordiv"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: timedelta / unit-timedelta is a float ratio and // is the floored int (whole and fractional cases)"""
import datetime

t = datetime.timedelta(hours=1, minutes=24, seconds=19)
second = datetime.timedelta(seconds=1)
assert t / second == 5059.0, f"truediv = {t / second!r}"
assert t // second == 5059, f"floordiv = {t // second!r}"

t = datetime.timedelta(minutes=2, seconds=30)
minute = datetime.timedelta(minutes=1)
assert t / minute == 2.5, f"truediv frac = {t / minute!r}"
assert t // minute == 2, f"floordiv frac = {t // minute!r}"
print("timedelta_div_and_floordiv OK")
"###);
    assert_output(&out, r###"timedelta_div_and_floordiv OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_divmod_and_mod.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_divmod_and_mod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_divmod_and_mod"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: divmod(td, unit) returns (int quotient, timedelta remainder), flooring toward -inf for a negative dividend, and % mirrors the remainder"""
import datetime

minute = datetime.timedelta(minutes=1)

t = datetime.timedelta(minutes=2, seconds=30)
q, r = divmod(t, minute)
assert q == 2, f"divmod q = {q!r}"
assert r == datetime.timedelta(seconds=30), f"divmod r = {r!r}"

# Negative dividend: quotient floors toward negative infinity.
tn = datetime.timedelta(minutes=-2, seconds=30)
q, r = divmod(tn, minute)
assert q == -2, f"neg divmod q = {q!r}"
assert r == datetime.timedelta(seconds=30), f"neg divmod r = {r!r}"

# Modulo mirrors divmod's remainder.
assert tn % minute == datetime.timedelta(seconds=30), f"mod = {tn % minute!r}"
print("timedelta_divmod_and_mod OK")
"###);
    assert_output(&out, r###"timedelta_divmod_and_mod OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_mixed_units_normalize.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_mixed_units_normalize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_mixed_units_normalize"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: mixed/negative units normalize down to a single microsecond (the canonical carry-cancellation identity)"""
import datetime

t1 = datetime.timedelta(days=100, weeks=-7, hours=-24 * (100 - 49), minutes=-3,
                        seconds=12, microseconds=(3 * 60 - 12) * 1000000.0 + 1)
assert t1 == datetime.timedelta(microseconds=1), f"normalize = {t1!r}"
print("timedelta_mixed_units_normalize OK")
"###);
    assert_output(&out, r###"timedelta_mixed_units_normalize OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_negation_and_abs.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_negation_and_abs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_negation_and_abs"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=-1).days == -1 and abs() of it has days == 1"""
import datetime

td_neg = datetime.timedelta(days=-1)
assert td_neg.days == -1, f"negative td days = {td_neg.days!r}"
td_abs = abs(td_neg)
assert td_abs.days == 1, f"abs(neg td) = {td_abs.days!r}"
print("timedelta_negation_and_abs OK")
"###);
    assert_output(&out, r###"timedelta_negation_and_abs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timedelta_total_seconds.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timedelta_total_seconds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_total_seconds"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=1,hours=1,minutes=1,seconds=1).total_seconds() == 86400+3600+60+1 as a float"""
import datetime

td = datetime.timedelta(days=1, hours=1, minutes=1, seconds=1)
expected = 86400 + 3600 + 60 + 1
assert td.total_seconds() == float(expected), f"total_seconds = {td.total_seconds()!r}"
print("timedelta_total_seconds OK")
"###);
    assert_output(&out, r###"timedelta_total_seconds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timezone_not_equal_bare_tzinfo.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timezone_not_equal_bare_tzinfo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_not_equal_bare_tzinfo"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: a concrete timezone (utc, or a fixed offset) never compares equal to a bare tzinfo() instance"""
import datetime

assert datetime.timezone.utc != datetime.tzinfo(), "utc != bare tzinfo"
assert datetime.timezone(datetime.timedelta(hours=1)) != datetime.tzinfo(), "offset != bare tzinfo"
print("timezone_not_equal_bare_tzinfo OK")
"###);
    assert_output(&out, r###"timezone_not_equal_bare_tzinfo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timezone_repr_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timezone_repr_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_repr_roundtrip"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: repr(timezone) is a valid constructor expression: eval(repr(tz)) == tz for named offsets and utc/min/max"""
import datetime

timezone = datetime.timezone  # eval'd reprs reference datetime.timezone(...)
ACDT = datetime.timezone(datetime.timedelta(hours=9.5), "ACDT")
EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for tz in (ACDT, EST, datetime.timezone.utc,
           datetime.timezone.min, datetime.timezone.max):
    assert tz == eval(repr(tz)), f"repr round-trip = {repr(tz)!r}"
print("timezone_repr_roundtrip OK")
"###);
    assert_output(&out, r###"timezone_repr_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timezone_str_is_name.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timezone_str_is_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_str_is_name"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: str(timezone) equals its tzname(None); a named offset reports its given name"""
import datetime

ACDT = datetime.timezone(datetime.timedelta(hours=9.5), "ACDT")
EST = datetime.timezone(-datetime.timedelta(hours=5), "EST")
for tz in (ACDT, EST, datetime.timezone.utc,
           datetime.timezone.min, datetime.timezone.max):
    assert str(tz) == tz.tzname(None), f"str = {str(tz)!r}"
assert ACDT.tzname(None) == "ACDT", f"ACDT name = {ACDT.tzname(None)!r}"
print("timezone_str_is_name OK")
"###);
    assert_output(&out, r###"timezone_str_is_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/timezone_utc_dst_none.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_timezone_utc_dst_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timezone_utc_dst_none"
# subject = "datetime.timezone"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timezone: timezone.utc.dst(datetime) is always None"""
import datetime

assert datetime.timezone.utc.dst(datetime.datetime(2010, 1, 1)) is None, "utc dst None"
print("timezone_utc_dst_none OK")
"###);
    assert_output(&out, r###"timezone_utc_dst_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/toordinal_fromordinal_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_toordinal_fromordinal_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "toordinal_fromordinal_roundtrip"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: toordinal/fromordinal are exact inverses at known anchors (1-01-01=1, 1-12-31=365, 2-01-01=366, 1945-11-12=710347) with time-zero on the way back"""
import datetime

for y, m, d, n in [(1, 1, 1, 1), (1, 12, 31, 365), (2, 1, 1, 366),
                   (1945, 11, 12, 710347)]:
    dt = datetime.datetime(y, m, d)
    assert dt.toordinal() == n, f"toordinal({y}) = {dt.toordinal()!r}"
    back = datetime.datetime.fromordinal(n)
    assert back == dt, f"fromordinal({n}) = {back!r}"
    assert back.hour == 0 and back.microsecond == 0, "fromordinal time-zero"
print("toordinal_fromordinal_roundtrip OK")
"###);
    assert_output(&out, r###"toordinal_fromordinal_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/datetime/weekday_advances_across_week.py`.
#[test]
fn test_gen_behavior_std_libs_datetime_weekday_advances_across_week() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "weekday_advances_across_week"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: weekday()/isoweekday() advance one step per day across a full week starting Mon 2002-03-04"""
import datetime

for i in range(7):
    assert datetime.datetime(2002, 3, 4 + i).weekday() == i, f"weekday {i}"
    assert datetime.datetime(2002, 3, 4 + i).isoweekday() == i + 1, f"isoweekday {i}"
print("weekday_advances_across_week OK")
"###);
    assert_output(&out, r###"weekday_advances_across_week OK
"###);
}
