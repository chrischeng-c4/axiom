# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_datetime"
# subject = "cpython321.test_datetime"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_datetime.py"
# status = "filled"
# ///
"""cpython321.test_datetime: execute CPython 3.12 seed test_datetime"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_datetime.py — #2692 CPython datetime seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/datetimetester.py
# (the upstream module is ~7000 lines covering the full pure-Python +
# C-accelerated datetime surface). Instead it is the *smallest*
# Mamba-authored seed distilled from CPython's date/timedelta arithmetic
# smoke surface: it asserts deterministic calendar invariants
# (leap-year February 28→29, year boundary December 31→January 1,
# negative-delta January 1→prior December 31) with raw `assert`
# statements and emits a positive proof-of-execution marker that
# the runner (`cpython_lib_test_runner.rs`, #2691) classifies as
# `AssertionPass` — not `ImportPass` or `Stub`.
#
# Why so small? Mamba's current datetime surface presents `date`,
# `datetime`, and `timedelta` constructors plus `date + timedelta(days=N)`
# arithmetic. Richer APIs (`isoformat`, `weekday`, `total_seconds`,
# `timedelta(hours=…, minutes=…)`, `date - date`, `timedelta * int`)
# return None or AttributeError on mamba today, so this seed stays on
# the working `days`-only arithmetic path. Asserts will be added in the
# same commit that closes each gap.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba (lookup
# yields a stale empty value). Asserts are therefore inlined at module
# top-level so every check executes in the same scope the ledger lives in.
#
# Why no timezone or locale instability? The whole fixture is fixed-date
# arithmetic on naive `date` objects. No `now()`, no `astimezone()`, no
# `strftime()` — those depend on system clock/TZ/locale and would break
# the deterministic assertion guarantee the runner relies on.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inversion (e.g. flipping
#     `d.year == 2024` to `d.year == 2025`) raises AssertionError →
#     non-zero exit → runner classifies as `Fail`, never silently passes.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: datetime N asserts` to stdout. The
#     runner sees the marker and classifies as `AssertionPass`.

import datetime

_ledger: list[int] = []

# 1. Module identity: datetime's own __name__ must be "datetime".
#    First-line invariant; inverting it fails the runner.
assert datetime.__name__ == "datetime", "datetime.__name__ must be 'datetime'"
_ledger.append(1)

# 2. date(year, month, day) round-trips its constructor args. Catches
#    a class of bootstrap regressions where the year/month/day slots
#    get swapped or default-initialized.
_d_anchor = datetime.date(2024, 2, 28)
assert _d_anchor.year == 2024, "date.year must reflect constructor"
_ledger.append(1)
assert _d_anchor.month == 2, "date.month must reflect constructor"
_ledger.append(1)
assert _d_anchor.day == 28, "date.day must reflect constructor"
_ledger.append(1)

# 3. timedelta(days=N).days round-trips. Anchors the timedelta side of
#    the arithmetic before we exercise date+timedelta.
_td1 = datetime.timedelta(days=1)
assert _td1.days == 1, "timedelta(days=1).days must be 1"
_ledger.append(1)

# 4. Leap-year February crossover: 2024-02-28 + 1d == 2024-02-29.
#    The canonical date-arithmetic regression: a runtime that miscomputes
#    leap years emits 2024-03-01 here. 2024 is divisible by 4 and by 400,
#    so it IS a leap year; this is the load-bearing case.
_feb29 = _d_anchor + _td1
assert _feb29.year == 2024, "Feb 28 + 1d stays in 2024"
_ledger.append(1)
assert _feb29.month == 2, "Feb 28 + 1d stays in February (leap year)"
_ledger.append(1)
assert _feb29.day == 29, "Feb 28 + 1d == Feb 29 in leap year 2024"
_ledger.append(1)

# 5. Past-leap-day crossover: 2024-02-29 + 1d == 2024-03-01. Distinguishes
#    "we handled Feb 28 by accident" from "we actually rolled the month".
_mar1 = datetime.date(2024, 2, 29) + _td1
assert _mar1.month == 3, "Feb 29 + 1d rolls to March"
_ledger.append(1)
assert _mar1.day == 1, "Feb 29 + 1d lands on the 1st"
_ledger.append(1)

# 6. Year boundary: 2024-12-31 + 1d == 2025-01-01. Verifies the year
#    increments when day arithmetic crosses December.
_new_year = datetime.date(2024, 12, 31) + _td1
assert _new_year.year == 2025, "Dec 31 + 1d rolls the year"
_ledger.append(1)
assert _new_year.month == 1, "Dec 31 + 1d lands in January"
_ledger.append(1)
assert _new_year.day == 1, "Dec 31 + 1d lands on the 1st"
_ledger.append(1)

# 7. Negative timedelta: 2024-01-01 + timedelta(days=-1) == 2023-12-31.
#    Verifies the opposite direction works — a runtime that only handles
#    positive deltas would land on a wrong day here.
_back = datetime.date(2024, 1, 1) + datetime.timedelta(days=-1)
assert _back.year == 2023, "Jan 1 + (-1d) rolls year back"
_ledger.append(1)
assert _back.month == 12, "Jan 1 + (-1d) lands in December"
_ledger.append(1)
assert _back.day == 31, "Jan 1 + (-1d) lands on the 31st"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: datetime {len(_ledger)} asserts")
