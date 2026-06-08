# Operational AssertionPass seed for `datetime.date`, `datetime.datetime`,
# and `datetime.timedelta` construction, attribute access, and the
# arithmetic that links them. Surface: `date(y, m, d)` exposes
# `.year`, `.month`, `.day`. `date + timedelta(days=n)` produces a
# new date n days later, and `< / >` compare chronologically. Month
# rollover at the day boundary works: `date(2024, 1, 31) +
# timedelta(days=1)` lands in February. `datetime(y, m, d, H, M, S)`
# exposes the same date attributes plus `.hour`, `.minute`, `.second`.
# `timedelta(days=n)` exposes `.days`. The `isinstance` predicate
# against the unbound class symbols and class-level zero-arg
# constructors `date.today()`/`datetime.now()` are NOT asserted here
# — those surfaces are mamba-specific and tracked elsewhere.
import datetime
_ledger: list[int] = []

# date construction and attribute access
d = datetime.date(2024, 1, 15)
assert d.year == 2024; _ledger.append(1)
assert d.month == 1; _ledger.append(1)
assert d.day == 15; _ledger.append(1)

# date + timedelta
d2 = d + datetime.timedelta(days=5)
assert d2.day == 20; _ledger.append(1)

# Chronological comparison
assert d2 > d; _ledger.append(1)
assert d < d2; _ledger.append(1)

# datetime construction and attribute access
dt = datetime.datetime(2024, 1, 15, 10, 30, 45)
assert dt.year == 2024; _ledger.append(1)
assert dt.hour == 10; _ledger.append(1)
assert dt.minute == 30; _ledger.append(1)
assert dt.second == 45; _ledger.append(1)

# timedelta construction and .days
td = datetime.timedelta(days=1)
assert td.days == 1; _ledger.append(1)

# Multi-day timedelta construction
td2 = datetime.timedelta(days=2)
assert td2.days == 2; _ledger.append(1)

# Month-rollover via day addition
d_end = datetime.date(2024, 1, 31)
d_next = d_end + datetime.timedelta(days=1)
assert d_next.month == 2; _ledger.append(1)
assert d_next.day == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_datetime_date_timedelta_ops {sum(_ledger)} asserts")
