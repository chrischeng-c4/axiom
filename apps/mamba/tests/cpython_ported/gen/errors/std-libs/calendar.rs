use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/calendar/day_abbr_negative_index_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_day_abbr_negative_index_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "day_abbr_negative_index_raises"
# subject = "calendar.day_abbr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_abbr: day_abbr_negative_index_raises (errors)."""
import calendar

_raised = False
try:
    calendar.day_abbr[-10]
except IndexError:
    _raised = True
assert _raised, "day_abbr_negative_index_raises: expected IndexError"
print("day_abbr_negative_index_raises OK")
"###);
    assert_output(&out, r###"day_abbr_negative_index_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/day_name_index_7_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_day_name_index_7_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "day_name_index_7_raises"
# subject = "calendar.day_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_name: day_name_index_7_raises (errors)."""
import calendar

_raised = False
try:
    calendar.day_name[7]
except IndexError:
    _raised = True
assert _raised, "day_name_index_7_raises: expected IndexError"
print("day_name_index_7_raises OK")
"###);
    assert_output(&out, r###"day_name_index_7_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/formatmonth_month_13_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_formatmonth_month_13_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "formatmonth_month_13_raises"
# subject = "calendar.TextCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: formatmonth_month_13_raises (errors)."""
import calendar

_raised = False
try:
    calendar.TextCalendar().formatmonth(2017, 13)
except calendar.IllegalMonthError:
    _raised = True
assert _raised, "formatmonth_month_13_raises: expected calendar.IllegalMonthError"
print("formatmonth_month_13_raises OK")
"###);
    assert_output(&out, r###"formatmonth_month_13_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/formatmonth_month_neg1_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_formatmonth_month_neg1_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "formatmonth_month_neg1_raises"
# subject = "calendar.TextCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: formatmonth_month_neg1_raises (errors)."""
import calendar

_raised = False
try:
    calendar.TextCalendar().formatmonth(2017, -1)
except calendar.IllegalMonthError:
    _raised = True
assert _raised, "formatmonth_month_neg1_raises: expected calendar.IllegalMonthError"
print("formatmonth_month_neg1_raises OK")
"###);
    assert_output(&out, r###"formatmonth_month_neg1_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/itermonthdays2_bad_month_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_itermonthdays2_bad_month_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "itermonthdays2_bad_month_raises"
# subject = "calendar.Calendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays2_bad_month_raises (errors)."""
import calendar

_raised = False
try:
    list(calendar.Calendar().itermonthdays2(2024, 13))
except calendar.IllegalMonthError:
    _raised = True
assert _raised, "itermonthdays2_bad_month_raises: expected calendar.IllegalMonthError"
print("itermonthdays2_bad_month_raises OK")
"###);
    assert_output(&out, r###"itermonthdays2_bad_month_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/month_name_index_13_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_month_name_index_13_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "month_name_index_13_raises"
# subject = "calendar.month_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.month_name: month_name_index_13_raises (errors)."""
import calendar

_raised = False
try:
    calendar.month_name[13]
except IndexError:
    _raised = True
assert _raised, "month_name_index_13_raises: expected IndexError"
print("month_name_index_13_raises OK")
"###);
    assert_output(&out, r###"month_name_index_13_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/monthrange_month_65_echoes_value.py`.
#[test]
fn test_gen_errors_std_libs_calendar_monthrange_month_65_echoes_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_month_65_echoes_value"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange(2004, 65) raises IllegalMonthError whose message text echoes the offending month number 65"""
import calendar

try:
    calendar.monthrange(2004, 65)
    print("month65: no_raise")
except calendar.IllegalMonthError as e:
    print("month65:", type(e).__name__, "echoes 65:", "65" in str(e))
print("monthrange_month_65_echoes_value OK")
"###);
    assert_output(&out, r###"month65: IllegalMonthError echoes 65: True
monthrange_month_65_echoes_value OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/monthrange_negative_year_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_calendar_monthrange_negative_year_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_negative_year_no_raise"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange accepts a negative (proleptic Gregorian) year; monthrange(-1, 1) does NOT raise"""
import calendar

print("negative_year:", calendar.monthrange(-1, 1))
print("monthrange_negative_year_no_raise OK")
"###);
    assert_output(&out, r###"negative_year: (calendar.FRIDAY, 31)
monthrange_negative_year_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/monthrange_valid_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_calendar_monthrange_valid_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_valid_no_raise"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: the happy path: monthrange(2024, 2) returns (weekday_of_first, num_days) without raising"""
import calendar

print("monthrange:", calendar.monthrange(2024, 2))
print("monthrange_valid_no_raise OK")
"###);
    assert_output(&out, r###"monthrange: (calendar.THURSDAY, 29)
monthrange_valid_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/setfirstweekday_123_echoes_value.py`.
#[test]
fn test_gen_errors_std_libs_calendar_setfirstweekday_123_echoes_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_123_echoes_value"
# subject = "calendar.setfirstweekday"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday(123) raises IllegalWeekdayError whose message text echoes the offending weekday number 123"""
import calendar

try:
    calendar.setfirstweekday(123)
    print("weekday123: no_raise")
except calendar.IllegalWeekdayError as e:
    print("weekday123:", type(e).__name__, "echoes 123:", "123" in str(e))
print("setfirstweekday_123_echoes_value OK")
"###);
    assert_output(&out, r###"weekday123: IllegalWeekdayError echoes 123: True
setfirstweekday_123_echoes_value OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/setfirstweekday_7_raises.py`.
#[test]
fn test_gen_errors_std_libs_calendar_setfirstweekday_7_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_7_raises"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday_7_raises (errors)."""
import calendar

_raised = False
try:
    calendar.setfirstweekday(7)
except ValueError:
    _raised = True
assert _raised, "setfirstweekday_7_raises: expected ValueError"
print("setfirstweekday_7_raises OK")
"###);
    assert_output(&out, r###"setfirstweekday_7_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/calendar/setfirstweekday_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_calendar_setfirstweekday_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_negative_valueerror"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday_negative_valueerror (errors)."""
import calendar

_raised = False
try:
    calendar.setfirstweekday(-1)
except ValueError:
    _raised = True
assert _raised, "setfirstweekday_negative_valueerror: expected ValueError"
print("setfirstweekday_negative_valueerror OK")
"###);
    assert_output(&out, r###"setfirstweekday_negative_valueerror OK
"###);
}
