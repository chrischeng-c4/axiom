import calendar

_ledger: list[int] = []

# isleap on 4/100/400 boundaries
assert calendar.isleap(2024), "2024 is a leap year"
_ledger.append(1)

assert not calendar.isleap(2023), "2023 is not a leap year"
_ledger.append(1)

assert calendar.isleap(2000), "2000 (divisible by 400) is a leap year"
_ledger.append(1)

assert not calendar.isleap(1900), "1900 (divisible by 100, not 400) is not a leap year"
_ledger.append(1)

# leapdays counts inclusive-exclusive
assert calendar.leapdays(2000, 2030) == 8, "leapdays(2000, 2030) is 8 (2000,04,08,12,16,20,24,28)"
_ledger.append(1)

# month_name and month_abbr
assert calendar.month_name[1] == "January", "month_name[1] is January"
_ledger.append(1)

assert calendar.month_name[12] == "December", "month_name[12] is December"
_ledger.append(1)

assert calendar.month_abbr[1] == "Jan", "month_abbr[1] is Jan"
_ledger.append(1)

# day_name and day_abbr
assert calendar.day_name[0] == "Monday", "day_name[0] is Monday"
_ledger.append(1)

assert calendar.day_abbr[0] == "Mon", "day_abbr[0] is Mon"
_ledger.append(1)

# weekday: 2024-01-01 was a Monday → 0
assert calendar.weekday(2024, 1, 1) == 0, "2024-01-01 weekday is 0 (Monday)"
_ledger.append(1)

# monthrange in leap February
assert calendar.monthrange(2024, 2) == (3, 29), "Feb 2024 starts Thu, 29 days"
_ledger.append(1)

# monthrange in non-leap February
assert calendar.monthrange(2023, 2) == (2, 28), "Feb 2023 starts Wed, 28 days"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_calendar {sum(_ledger)} asserts")
