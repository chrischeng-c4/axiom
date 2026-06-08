# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_calendar_leap_weekday_ops"
# subject = "cpython321.test_calendar_leap_weekday_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_calendar_leap_weekday_ops.py"
# status = "filled"
# ///
"""cpython321.test_calendar_leap_weekday_ops: execute CPython 3.12 seed test_calendar_leap_weekday_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `calendar` module — the
# stdlib calendar utilities (`isleap`, `leapdays`, `weekday`,
# `monthrange`, `month_name`, `month_abbr`, `day_name`, `day_abbr`,
# `MONDAY`/.../`SUNDAY` constants) used by date arithmetic,
# scheduling, holiday computation, and any code that needs to query
# day-of-week / month-length / leap-year facts independent of
# `datetime` instances. Surface focuses on the matching subset
# between mamba and CPython on the integer-day-of-week / count-of-
# leap-days / English-locale-name arrays. Mamba returns `int` for
# `monthrange(..)[0]` whereas CPython returns a `calendar.Day` enum
# (an IntEnum subclass), but `monthrange(..)[0] == 3` is True in
# both runtimes because `Day` is an IntEnum. No fixture coverage
# yet for the calendar module.
#
# Surface:
#   • calendar.isleap(year: int) → bool
#       — 2024 leap, 2023 not, 2000 leap (divisible by 400),
#         1900 not (divisible by 100 but not 400);
#   • calendar.leapdays(y1: int, y2: int) → int
#       — count of leap-year days in range [y1, y2);
#   • calendar.weekday(year, month, day) → int
#       — Monday=0 ... Sunday=6;
#   • calendar.monthrange(year, month) → (first_weekday, length)
#       — first day-of-week (0..6) + length of month;
#   • calendar.month_name / month_abbr — indexed [1..12] English
#       names ("January".."December") / abbrs ("Jan".."Dec");
#   • calendar.day_name / day_abbr — indexed [0..6] English names
#       ("Monday".."Sunday") / abbrs ("Mon".."Sun");
#   • calendar.MONDAY=0 ... SUNDAY=6 — canonical integer constants.
import calendar
_ledger: list[int] = []

# isleap — leap year rules
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2020) == True; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2021) == False; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)
assert calendar.isleap(2100) == False; _ledger.append(1)
assert calendar.isleap(2400) == True; _ledger.append(1)

# leapdays — count in range [y1, y2)
assert calendar.leapdays(2000, 2024) == 6; _ledger.append(1)
assert calendar.leapdays(2020, 2024) == 1; _ledger.append(1)
assert calendar.leapdays(2024, 2025) == 1; _ledger.append(1)
assert calendar.leapdays(2023, 2024) == 0; _ledger.append(1)
assert calendar.leapdays(2000, 2001) == 1; _ledger.append(1)
assert calendar.leapdays(1900, 1901) == 0; _ledger.append(1)

# weekday — known days (Monday=0..Sunday=6)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)  # Jan 1, 2024 = Monday
assert calendar.weekday(2024, 1, 15) == 0; _ledger.append(1)  # Jan 15, 2024 = Monday
assert calendar.weekday(2025, 12, 25) == 3; _ledger.append(1)  # Dec 25, 2025 = Thursday
assert calendar.weekday(2026, 1, 1) == 3; _ledger.append(1)  # Jan 1, 2026 = Thursday
assert calendar.weekday(2000, 1, 1) == 5; _ledger.append(1)  # Jan 1, 2000 = Saturday

# monthrange — first weekday + month length (compare via int)
assert calendar.monthrange(2024, 2)[0] == 3; _ledger.append(1)  # Thursday
assert calendar.monthrange(2024, 2)[1] == 29; _ledger.append(1)  # leap year Feb
assert calendar.monthrange(2023, 2)[1] == 28; _ledger.append(1)  # common year Feb
assert calendar.monthrange(2024, 1)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2024, 4)[1] == 30; _ledger.append(1)
assert calendar.monthrange(2024, 7)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2024, 12)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2024, 4)[0] == 0; _ledger.append(1)  # Monday

# Month-length table (non-leap year)
assert calendar.monthrange(2023, 1)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2023, 3)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2023, 5)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2023, 6)[1] == 30; _ledger.append(1)
assert calendar.monthrange(2023, 9)[1] == 30; _ledger.append(1)
assert calendar.monthrange(2023, 11)[1] == 30; _ledger.append(1)

# month_name — English names, indexed [1..12]
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[2] == "February"; _ledger.append(1)
assert calendar.month_name[6] == "June"; _ledger.append(1)
assert calendar.month_name[7] == "July"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)

# month_abbr — English abbreviations
assert calendar.month_abbr[1] == "Jan"; _ledger.append(1)
assert calendar.month_abbr[2] == "Feb"; _ledger.append(1)
assert calendar.month_abbr[6] == "Jun"; _ledger.append(1)
assert calendar.month_abbr[12] == "Dec"; _ledger.append(1)

# day_name — English names, indexed [0..6]
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.day_name[1] == "Tuesday"; _ledger.append(1)
assert calendar.day_name[2] == "Wednesday"; _ledger.append(1)
assert calendar.day_name[3] == "Thursday"; _ledger.append(1)
assert calendar.day_name[4] == "Friday"; _ledger.append(1)
assert calendar.day_name[5] == "Saturday"; _ledger.append(1)
assert calendar.day_name[6] == "Sunday"; _ledger.append(1)

# day_abbr — English abbreviations
assert calendar.day_abbr[0] == "Mon"; _ledger.append(1)
assert calendar.day_abbr[1] == "Tue"; _ledger.append(1)
assert calendar.day_abbr[2] == "Wed"; _ledger.append(1)
assert calendar.day_abbr[3] == "Thu"; _ledger.append(1)
assert calendar.day_abbr[6] == "Sun"; _ledger.append(1)

# Canonical day-of-week constants
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.TUESDAY == 1; _ledger.append(1)
assert calendar.WEDNESDAY == 2; _ledger.append(1)
assert calendar.THURSDAY == 3; _ledger.append(1)
assert calendar.FRIDAY == 4; _ledger.append(1)
assert calendar.SATURDAY == 5; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# firstweekday — defaults to Monday (0)
assert calendar.firstweekday() == 0; _ledger.append(1)

# Return type discipline
assert isinstance(calendar.isleap(2024), bool); _ledger.append(1)
assert isinstance(calendar.leapdays(2000, 2024), int); _ledger.append(1)
assert isinstance(calendar.weekday(2024, 1, 1), int); _ledger.append(1)
assert isinstance(calendar.monthrange(2024, 1), tuple); _ledger.append(1)
assert isinstance(calendar.month_name[1], str); _ledger.append(1)
assert isinstance(calendar.day_name[0], str); _ledger.append(1)
assert isinstance(calendar.firstweekday(), int); _ledger.append(1)

# Module-level attribute discipline
for _name in ["isleap", "leapdays", "weekday", "monthrange",
              "month_name", "month_abbr", "day_name", "day_abbr",
              "firstweekday", "MONDAY", "TUESDAY", "WEDNESDAY",
              "THURSDAY", "FRIDAY", "SATURDAY", "SUNDAY"]:
    assert hasattr(calendar, _name); _ledger.append(1)

# Day constants are mutually distinct and span 0..6
_days = [calendar.MONDAY, calendar.TUESDAY, calendar.WEDNESDAY,
         calendar.THURSDAY, calendar.FRIDAY, calendar.SATURDAY,
         calendar.SUNDAY]
assert len(_days) == len(set(_days)); _ledger.append(1)
assert min(_days) == 0; _ledger.append(1)
assert max(_days) == 6; _ledger.append(1)

# Idempotence — same query, same result
assert calendar.isleap(2024) == calendar.isleap(2024); _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == calendar.weekday(2024, 1, 1); _ledger.append(1)
assert calendar.monthrange(2024, 2) == calendar.monthrange(2024, 2); _ledger.append(1)

# monthrange day count is always in [28, 31]
for _y in [2023, 2024]:
    for _m in range(1, 13):
        _days_in = calendar.monthrange(_y, _m)[1]
        assert 28 <= _days_in <= 31; _ledger.append(1)

# weekday is always in [0, 6]
for _d in [1, 5, 10, 15, 20, 25]:
    _wd = calendar.weekday(2024, 1, _d)
    assert 0 <= _wd <= 6; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_calendar_leap_weekday_ops {sum(_ledger)} asserts")
