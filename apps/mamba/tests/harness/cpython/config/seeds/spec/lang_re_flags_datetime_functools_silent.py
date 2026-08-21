# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `re.IGNORECASE` flag in `re.search`
# (the documented "case-insensitive flag makes pattern match against
# differently-cased text" — mamba returns None for `re.search('abc',
# 'ABC', re.IGNORECASE)`), `re.MULTILINE` flag in `re.findall` (the
# documented "anchor ^ matches each line start under MULTILINE" —
# mamba returns ['line'] for `re.findall(r'^line', 'line1\\nline2',
# re.MULTILINE)` instead of ['line', 'line']), `re.DOTALL` flag in
# `re.search` (the documented "`.` matches newline under DOTALL" —
# mamba returns None for `re.search(r'a.b', 'a\\nb', re.DOTALL)`),
# `type(datetime.date(...))` (the documented "date constructor
# returns a `date` instance" — mamba returns `datetime.datetime`
# because the date class is aliased to datetime), `str(datetime.date
# (2024,1,15))` (the documented "str(date) renders only the YYYY-MM-DD
# date portion" — mamba returns '2024-01-15 00:00:00' with a time
# component because date is aliased to datetime), `hasattr(date,
# 'weekday')` (the documented "date instances expose .weekday()
# returning 0-6 weekday number" — mamba returns False because the
# method is missing), `(d2 - d1)` for two date objects (the documented
# "date subtraction returns a `timedelta`" — mamba returns None),
# `hasattr(datetime, 'timezone')` (the documented "datetime module
# exposes timezone class" — mamba returns False because the symbol is
# missing), `functools.partial(fn, b=5)(3)` (the documented "partial
# with keyword-only binding still produces the bound result" — mamba
# returns None because keyword-binding through partial is broken),
# and `functools.wraps(orig)(wrapped).__doc__` (the documented "wraps
# copies the original docstring to the wrapped function" — mamba
# returns None because __doc__ is not copied).
# Ten-pack pinned to atomic 254.
#
# Behavioral edges that CONFORM on mamba (re — basic match/search +
# group(0)/group(1)/group(2)/group('n'), multi-group .groups(),
# finditer, fullmatch full + partial-None, findall/sub/subn/split/
# escape, Pattern.pattern/.search/.findall/.sub, Match.start/end/span,
# hasattr Pattern/Match/error/IGNORECASE/MULTILINE/DOTALL/VERBOSE.
# datetime — date.year/.month/.day, datetime.year/.month/.day/.hour/
# .minute/.second, timedelta.days/.seconds, date+timedelta, date<date,
# date==date, str(datetime), hasattr date/datetime/timedelta.
# functools — reduce sum/mul/initial, partial positional binding,
# @cache + cache_info hits, @lru_cache, @wraps __name__, hasattr
# reduce/partial/cache/lru_cache/wraps/cmp_to_key/singledispatch/
# total_ordering/update_wrapper) are covered in the matching pass
# fixture `test_re_datetime_functools_value_ops`.
import re
import datetime
import functools
from typing import Any


_ledger: list[int] = []

# 1) re.IGNORECASE flag — must match 'ABC' for pattern 'abc'
#    (mamba: returns None — flag not respected)
def _re_ignorecase() -> Any:
    m = re.search(r"abc", "ABC", re.IGNORECASE)
    if m is None:
        return None
    return m.group(0)
assert _re_ignorecase() == "ABC"; _ledger.append(1)

# 2) re.MULTILINE flag — ^ matches each line start
#    (mamba: returns ['line'] — only matches first line start)
def _re_multiline() -> list:
    return re.findall(r"^line", "line1\nline2", re.MULTILINE)
assert _re_multiline() == ["line", "line"]; _ledger.append(1)

# 3) re.DOTALL flag — `.` matches newline
#    (mamba: returns None — flag not respected)
def _re_dotall() -> Any:
    m = re.search(r"a.b", "a\nb", re.DOTALL)
    if m is None:
        return None
    return m.group(0)
assert _re_dotall() == "a\nb"; _ledger.append(1)

# 4) type(datetime.date(...)) must be 'date'
#    (mamba: returns 'datetime' — class aliased)
assert type(datetime.date(2024, 1, 15)).__name__ == "date"; _ledger.append(1)

# 5) str(datetime.date(...)) must drop time component
#    (mamba: returns '2024-01-15 00:00:00' — time appended)
assert str(datetime.date(2024, 1, 15)) == "2024-01-15"; _ledger.append(1)

# 6) date instance .weekday() must exist
#    (mamba: hasattr returns False — method missing)
assert hasattr(datetime.date(2024, 1, 15), "weekday") == True; _ledger.append(1)

# 7) (d2 - d1) for two date objects must return a timedelta
#    (mamba: returns None — date-date subtraction silently broken)
def _date_sub() -> Any:
    diff = datetime.date(2024, 1, 20) - datetime.date(2024, 1, 15)
    if diff is None:
        return None
    return diff.days
assert _date_sub() == 5; _ledger.append(1)

# 8) datetime module must expose 'timezone' class
#    (mamba: returns False — symbol missing)
assert hasattr(datetime, "timezone") == True; _ledger.append(1)

# 9) functools.partial with keyword-binding then positional call
#    (mamba: returns None — keyword-bound partial broken)
def _partial_kw() -> Any:
    return functools.partial(lambda a, b: a * b, b=5)(3)
assert _partial_kw() == 15; _ledger.append(1)

# 10) functools.wraps must copy __doc__ from original
#     (mamba: returns None — __doc__ is not copied)
def _orig_for_wraps_doc() -> int:
    """orig doc"""
    return 1
@functools.wraps(_orig_for_wraps_doc)
def _wrapped_for_wraps_doc() -> int:
    return 2
assert _wrapped_for_wraps_doc.__doc__ == "orig doc"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_re_flags_datetime_functools_silent {sum(_ledger)} asserts")
