# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_calendar_weekday_monthrange_ops"
# subject = "cpython321.test_calendar_weekday_monthrange_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_calendar_weekday_monthrange_ops.py"
# status = "filled"
# ///
"""cpython321.test_calendar_weekday_monthrange_ops: execute CPython 3.12 seed test_calendar_weekday_monthrange_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the calendar-module date-math
# surface. Surface: calendar.isleap(year) returns True for the canonical
# Gregorian leap years — divisible by 4, divisible by 400 (2000 leap,
# 1900 not), and the modern 2020/2024 leaps; calendar.leapdays(y1, y2)
# counts leap years in [y1, y2); calendar.weekday(y, m, d) returns the
# Mon=0..Sun=6 weekday for a given date (verified against
# 2024-01-01 being Monday and 2024-01-07 being Sunday); calendar.
# monthrange(y, m) returns (first_weekday, day_count) for the month —
# verified for January 2024 (31 days), February of a leap year (29
# days), February of a common year (28 days), and a 30-day April.
# Companion to test_calendar_names_constants_ops (which covers the
# day_name / month_name / weekday-int constants).
import calendar
_ledger: list[int] = []

# isleap — canonical Gregorian rule
assert calendar.isleap(2020) == True; _ledger.append(1)
assert calendar.isleap(2021) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)

# leapdays — count leap years in [y1, y2)
assert calendar.leapdays(2000, 2010) == 3; _ledger.append(1)
assert calendar.leapdays(2020, 2025) == 2; _ledger.append(1)

# weekday — Mon=0, Sun=6
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
assert calendar.weekday(2024, 1, 7) == 6; _ledger.append(1)

# monthrange — (first_weekday, day_count)
assert calendar.monthrange(2024, 1) == (0, 31); _ledger.append(1)
assert calendar.monthrange(2024, 2) == (3, 29); _ledger.append(1)
assert calendar.monthrange(2023, 2) == (2, 28); _ledger.append(1)
assert calendar.monthrange(2024, 4) == (0, 30); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_calendar_weekday_monthrange_ops {sum(_ledger)} asserts")
