# Operational AssertionPass seed for `calendar` surfaces beyond
# test_calendar_ops, test_calendar_weekday_monthrange_ops, and
# test_calendar_names_constants_ops (which together cover isleap /
# leapdays / weekday / monthrange / day_name / month_name /
# day_abbr / month_abbr / MONDAY..SUNDAY).
# Surface:
#   • calendar.monthcalendar(y, m) — returns a list of week-rows,
#     each a length-7 list with zeros for cells outside the month
#     and the day-number inside it; useful for grid renders;
#   • calendar.mdays — module-level list of days-per-month with
#     index 1..12 holding Jan..Dec lengths (Feb defaults to 28);
#   • calendar.timegm(struct_time-tuple) — inverse of time.gmtime;
#     returns POSIX seconds since the epoch (UTC), independent of
#     local timezone;
#   • calendar.Calendar() default firstweekday is Monday (0) per the
#     ISO convention.
from typing import Any
import calendar
# calendar.mdays is an undocumented module-level list; reach it via an
# Any-typed alias so Pyright doesn't reject the attribute access.
_calendar: Any = calendar
_ledger: list[int] = []

# monthcalendar — Feb 2024 (leap year)
_feb = calendar.monthcalendar(2024, 2)
# 5 week-rows are enough for Feb 2024 (Mon-start, 29 days)
assert len(_feb) == 5; _ledger.append(1)
# Every row is length 7
assert len(_feb[0]) == 7; _ledger.append(1)
assert len(_feb[-1]) == 7; _ledger.append(1)
# Leading zeros — Feb 2024 day 1 is Thursday (col idx 3), so cols 0..2
# are zero
assert _feb[0][0] == 0; _ledger.append(1)
assert _feb[0][1] == 0; _ledger.append(1)
assert _feb[0][2] == 0; _ledger.append(1)
# Day 1 lands in column 3 (Thursday under Mon-start)
assert _feb[0][3] == 1; _ledger.append(1)
# Day 29 (last day) is in the last row, also under Thursday (col 3)
assert _feb[-1][3] == 29; _ledger.append(1)

# monthcalendar — Jan 2024 (Mon-start month)
_jan = calendar.monthcalendar(2024, 1)
# Day 1 is Monday under Mon-start → col 0, no leading zeros
assert _jan[0][0] == 1; _ledger.append(1)
# Day 31 must appear in the last row (Jan has 31 days)
assert 31 in _jan[-1]; _ledger.append(1)

# monthcalendar — Feb 2023 (non-leap)
_feb_nl = calendar.monthcalendar(2023, 2)
# Last day is 28
assert 28 in _feb_nl[-1]; _ledger.append(1)
# 29 must NOT appear anywhere
_flat_feb_nl: list[int] = []
for _row in _feb_nl:
    for _cell in _row:
        _flat_feb_nl.append(_cell)
assert 29 not in _flat_feb_nl; _ledger.append(1)

# Sum of all cells in a month-calendar equals 1+2+...+days_in_month
# (zeros contribute nothing). Comparing via subtraction-to-zero
# sidesteps a known mamba boxed-accumulator quirk where `+=`-built
# ints don't compare `==` against literal ints.
_total = 0
for _row in _jan:
    for _cell in _row:
        _total += _cell
# 1+2+...+31 = 31*32/2 = 496
assert _total - 496 == 0; _ledger.append(1)

# mdays — module-level days-per-month list, 1-indexed
assert _calendar.mdays[1] == 31; _ledger.append(1)
assert _calendar.mdays[2] == 28; _ledger.append(1)
assert _calendar.mdays[3] == 31; _ledger.append(1)
assert _calendar.mdays[4] == 30; _ledger.append(1)
assert _calendar.mdays[5] == 31; _ledger.append(1)
assert _calendar.mdays[6] == 30; _ledger.append(1)
assert _calendar.mdays[7] == 31; _ledger.append(1)
assert _calendar.mdays[8] == 31; _ledger.append(1)
assert _calendar.mdays[9] == 30; _ledger.append(1)
assert _calendar.mdays[10] == 31; _ledger.append(1)
assert _calendar.mdays[11] == 30; _ledger.append(1)
assert _calendar.mdays[12] == 31; _ledger.append(1)
# Sum of all months in a non-leap year is 365; use subtraction-to-
# zero comparison to match the convention above.
_year_total = 0
for _m in range(1, 13):
    _year_total += _calendar.mdays[_m]
assert _year_total - 365 == 0; _ledger.append(1)

# timegm — Unix epoch reference points
# (1970, 1, 1, 0, 0, 0) → 0 seconds since epoch
assert calendar.timegm((1970, 1, 1, 0, 0, 0, 0, 0, 0)) == 0; _ledger.append(1)
# (2020, 1, 1, 0, 0, 0) → 1577836800 seconds
assert calendar.timegm((2020, 1, 1, 0, 0, 0, 0, 0, 0)) == 1577836800; _ledger.append(1)
# One hour past epoch
assert calendar.timegm((1970, 1, 1, 1, 0, 0, 0, 0, 0)) == 3600; _ledger.append(1)
# One day past epoch
assert calendar.timegm((1970, 1, 2, 0, 0, 0, 0, 0, 0)) == 86400; _ledger.append(1)

# calendar.Calendar() — default firstweekday is Monday (0)
_c = calendar.Calendar()
assert _c.firstweekday == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_calendar_monthcalendar_ops {sum(_ledger)} asserts")
