# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_calendar_names_constants_ops"
# subject = "cpython321.test_calendar_names_constants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_calendar_names_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_calendar_names_constants_ops: execute CPython 3.12 seed test_calendar_names_constants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for calendar module surfaces beyond
# test_calendar_ops (which covers isleap / monthrange / weekday).
# Surface: month_name[1..12] (January..December) and month_abbr[1..12]
# (Jan..Dec); day_name[0..6] (Monday..Sunday) and day_abbr[0..6]
# (Mon..Sun); mdays[i] returns the days-in-month for month i with
# Feb defaulting to 28; weekday constants MONDAY..SUNDAY are 0..6;
# leapdays(y1, y2) counts the leap years in [y1, y2); the
# month_name and day_name indices follow Python's ISO weekday
# convention (Mon=0..Sun=6).
import calendar
_ledger: list[int] = []

# Full month names — index 0 is empty, 1..12 are the calendar months
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[2] == "February"; _ledger.append(1)
assert calendar.month_name[3] == "March"; _ledger.append(1)
assert calendar.month_name[6] == "June"; _ledger.append(1)
assert calendar.month_name[9] == "September"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)

# Month abbreviations — same 1-based indexing
assert calendar.month_abbr[1] == "Jan"; _ledger.append(1)
assert calendar.month_abbr[2] == "Feb"; _ledger.append(1)
assert calendar.month_abbr[3] == "Mar"; _ledger.append(1)
assert calendar.month_abbr[12] == "Dec"; _ledger.append(1)

# Day names — index 0 is Monday, 6 is Sunday (ISO convention)
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.day_name[1] == "Tuesday"; _ledger.append(1)
assert calendar.day_name[2] == "Wednesday"; _ledger.append(1)
assert calendar.day_name[3] == "Thursday"; _ledger.append(1)
assert calendar.day_name[4] == "Friday"; _ledger.append(1)
assert calendar.day_name[5] == "Saturday"; _ledger.append(1)
assert calendar.day_name[6] == "Sunday"; _ledger.append(1)

# Day abbreviations
assert calendar.day_abbr[0] == "Mon"; _ledger.append(1)
assert calendar.day_abbr[1] == "Tue"; _ledger.append(1)
assert calendar.day_abbr[5] == "Sat"; _ledger.append(1)
assert calendar.day_abbr[6] == "Sun"; _ledger.append(1)

# Weekday constants
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.TUESDAY == 1; _ledger.append(1)
assert calendar.WEDNESDAY == 2; _ledger.append(1)
assert calendar.THURSDAY == 3; _ledger.append(1)
assert calendar.FRIDAY == 4; _ledger.append(1)
assert calendar.SATURDAY == 5; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# mdays — days-in-month for each month (Feb defaults to 28)
assert calendar.mdays[1] == 31; _ledger.append(1)
assert calendar.mdays[2] == 28; _ledger.append(1)
assert calendar.mdays[3] == 31; _ledger.append(1)
assert calendar.mdays[4] == 30; _ledger.append(1)
assert calendar.mdays[5] == 31; _ledger.append(1)
assert calendar.mdays[6] == 30; _ledger.append(1)
assert calendar.mdays[7] == 31; _ledger.append(1)
assert calendar.mdays[8] == 31; _ledger.append(1)
assert calendar.mdays[9] == 30; _ledger.append(1)
assert calendar.mdays[10] == 31; _ledger.append(1)
assert calendar.mdays[11] == 30; _ledger.append(1)
assert calendar.mdays[12] == 31; _ledger.append(1)

# leapdays(y1, y2) counts the leap years in the half-open range [y1, y2)
# 2000..2010 contains 2000, 2004, 2008 → 3 leap years
assert calendar.leapdays(2000, 2010) == 3; _ledger.append(1)
# 2020..2025 contains 2020, 2024 → 2
assert calendar.leapdays(2020, 2025) == 2; _ledger.append(1)
# Empty range → 0
assert calendar.leapdays(2020, 2020) == 0; _ledger.append(1)
# Single leap year in range
assert calendar.leapdays(2024, 2025) == 1; _ledger.append(1)
# Range spanning a century non-leap (1900 isn't, 2000 is)
assert calendar.leapdays(1900, 2001) == 25; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_calendar_names_constants_ops {sum(_ledger)} asserts")
