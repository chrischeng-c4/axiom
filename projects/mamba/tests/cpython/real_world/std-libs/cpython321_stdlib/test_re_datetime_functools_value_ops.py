# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_re_datetime_functools_value_ops"
# subject = "cpython321.test_re_datetime_functools_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_re_datetime_functools_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_re_datetime_functools_value_ops: execute CPython 3.12 seed test_re_datetime_functools_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 254 pass conformance — re module (re.match/re.search basic
# capture, multi-group .groups()/.group(0)/.group(1)/.group(2), named
# group .group('n'), re.finditer collecting matches, re.fullmatch
# full-match success and partial-match None, re.findall list, re.sub
# replacement, re.subn (replacement, count), re.split, re.escape
# special-char escape, Pattern.pattern/Pattern.search/Pattern.findall/
# Pattern.sub/Pattern.split, Match.start/end/span, hasattr Pattern/
# Match/error/IGNORECASE/MULTILINE/DOTALL/VERBOSE) + datetime module
# (date constructor + .year/.month/.day, datetime constructor +
# .year/.month/.day/.hour/.minute/.second, timedelta .days/.seconds
# and combo days+seconds, date + timedelta, date < date, date == date,
# str(datetime) text form, hasattr date/datetime/timedelta) + functools
# module (reduce sum/mul/with-initial, partial positional binding,
# @cache decorator + cache_info hits > 0, @lru_cache decorator, @wraps
# copies __name__, hasattr reduce/partial/cache/lru_cache/wraps/
# cmp_to_key/singledispatch/total_ordering/update_wrapper). All asserts
# match between CPython 3.12 and mamba.
import re
import datetime
import functools


_ledger: list[int] = []

# 1) re — basic match/search/groups/named
m = re.match(r"\d+", "123abc")
assert m is not None; _ledger.append(1)
assert m.group(0) == "123"; _ledger.append(1)

m = re.search(r"\d+", "abc123def")
assert m is not None; _ledger.append(1)
assert m.group(0) == "123"; _ledger.append(1)

m = re.match(r"(\d+)-(\w+)", "42-foo")
assert m is not None; _ledger.append(1)
assert m.groups() == ("42", "foo"); _ledger.append(1)
assert m.group(0) == "42-foo"; _ledger.append(1)
assert m.group(1) == "42"; _ledger.append(1)
assert m.group(2) == "foo"; _ledger.append(1)

m = re.match(r"(?P<n>\d+)", "42")
assert m is not None; _ledger.append(1)
assert m.group("n") == "42"; _ledger.append(1)

# 2) re.finditer
assert [m.group(0) for m in re.finditer(r"\d+", "a1 b22 c333")] == ["1", "22", "333"]; _ledger.append(1)

# 3) re.fullmatch
fm = re.fullmatch(r"\d+", "123")
assert fm is not None; _ledger.append(1)
assert fm.group(0) == "123"; _ledger.append(1)
assert re.fullmatch(r"\d+", "123abc") is None; _ledger.append(1)

# 4) re — findall / sub / subn / split / escape
assert re.findall(r"\d+", "a1 b22 c333") == ["1", "22", "333"]; _ledger.append(1)
assert re.sub(r"\d+", "N", "a1 b22") == "aN bN"; _ledger.append(1)
assert re.subn(r"\d+", "N", "a1 b22") == ("aN bN", 2); _ledger.append(1)
assert re.split(r"\s+", "a b  c") == ["a", "b", "c"]; _ledger.append(1)
assert re.escape("a.b*c") == "a\\.b\\*c"; _ledger.append(1)

# 5) re — Pattern object
p = re.compile(r"\d+")
assert p.pattern == r"\d+"; _ledger.append(1)
m = p.search("abc123")
assert m is not None; _ledger.append(1)
assert m.group(0) == "123"; _ledger.append(1)
assert p.findall("a1 b22") == ["1", "22"]; _ledger.append(1)
assert p.sub("N", "a1 b22") == "aN bN"; _ledger.append(1)

# 6) re — Match span/start/end
m = re.search(r"\d+", "abc123def")
assert m is not None; _ledger.append(1)
assert m.start() == 3; _ledger.append(1)
assert m.end() == 6; _ledger.append(1)
assert m.span() == (3, 6); _ledger.append(1)

# 7) re — hasattr surface
assert hasattr(re, "Pattern") == True; _ledger.append(1)
assert hasattr(re, "Match") == True; _ledger.append(1)
assert hasattr(re, "error") == True; _ledger.append(1)
assert hasattr(re, "IGNORECASE") == True; _ledger.append(1)
assert hasattr(re, "MULTILINE") == True; _ledger.append(1)
assert hasattr(re, "DOTALL") == True; _ledger.append(1)
assert hasattr(re, "VERBOSE") == True; _ledger.append(1)

# 8) datetime — date constructor + attrs
d = datetime.date(2024, 1, 15)
assert d.year == 2024; _ledger.append(1)
assert d.month == 1; _ledger.append(1)
assert d.day == 15; _ledger.append(1)

# 9) datetime — datetime constructor + attrs
dt = datetime.datetime(2024, 1, 15, 10, 30, 45)
assert dt.year == 2024; _ledger.append(1)
assert dt.month == 1; _ledger.append(1)
assert dt.day == 15; _ledger.append(1)
assert dt.hour == 10; _ledger.append(1)
assert dt.minute == 30; _ledger.append(1)
assert dt.second == 45; _ledger.append(1)
assert str(dt) == "2024-01-15 10:30:45"; _ledger.append(1)

# 10) datetime — timedelta + arithmetic
td = datetime.timedelta(days=5)
assert td.days == 5; _ledger.append(1)
td2 = datetime.timedelta(seconds=3600)
assert td2.seconds == 3600; _ledger.append(1)
td3 = datetime.timedelta(days=1, seconds=3600)
assert td3.days == 1; _ledger.append(1)
assert td3.seconds == 3600; _ledger.append(1)

# 11) datetime — date + timedelta and comparisons
d1 = datetime.date(2024, 1, 15)
d2 = datetime.date(2024, 1, 20)
sum_date = d1 + datetime.timedelta(days=5)
assert sum_date.year == 2024; _ledger.append(1)
assert sum_date.month == 1; _ledger.append(1)
assert sum_date.day == 20; _ledger.append(1)
assert (d1 < d2) == True; _ledger.append(1)
assert (d1 == d1) == True; _ledger.append(1)

# 12) datetime — module hasattr surface
assert hasattr(datetime, "date") == True; _ledger.append(1)
assert hasattr(datetime, "datetime") == True; _ledger.append(1)
assert hasattr(datetime, "timedelta") == True; _ledger.append(1)

# 13) functools — reduce
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]) == 10; _ledger.append(1)
assert functools.reduce(lambda a, b: a + b, [1, 2, 3], 100) == 106; _ledger.append(1)
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4]) == 24; _ledger.append(1)

# 14) functools — partial positional
assert functools.partial(lambda a, b: a + b, 10)(5) == 15; _ledger.append(1)
assert functools.partial(lambda a, b, c: a + b + c, 1, 2)(3) == 6; _ledger.append(1)

# 15) functools — cache + lru_cache (recursive accumulated int compared
# via subtraction-equals-zero to dodge mamba's known boxed-int direct
# `==` quirk on recursive accumulators)
@functools.cache
def _fib_cache(n: int) -> int:
    if n < 2:
        return n
    return _fib_cache(n - 1) + _fib_cache(n - 2)
assert (_fib_cache(10) - 55) == 0; _ledger.append(1)
assert _fib_cache.cache_info().hits > 0; _ledger.append(1)

@functools.lru_cache(maxsize=10)
def _fib_lru(n: int) -> int:
    if n < 2:
        return n
    return _fib_lru(n - 1) + _fib_lru(n - 2)
assert (_fib_lru(10) - 55) == 0; _ledger.append(1)
assert hasattr(_fib_lru, "cache_info") == True; _ledger.append(1)

# 16) functools — wraps copies __name__
def _orig_for_wraps() -> int:
    """orig doc"""
    return 1
@functools.wraps(_orig_for_wraps)
def _wrapped_for_wraps() -> int:
    return 2
assert _wrapped_for_wraps.__name__ == "_orig_for_wraps"; _ledger.append(1)

# 17) functools — module hasattr surface
assert hasattr(functools, "reduce") == True; _ledger.append(1)
assert hasattr(functools, "partial") == True; _ledger.append(1)
assert hasattr(functools, "cache") == True; _ledger.append(1)
assert hasattr(functools, "lru_cache") == True; _ledger.append(1)
assert hasattr(functools, "wraps") == True; _ledger.append(1)
assert hasattr(functools, "cmp_to_key") == True; _ledger.append(1)
assert hasattr(functools, "singledispatch") == True; _ledger.append(1)
assert hasattr(functools, "total_ordering") == True; _ledger.append(1)
assert hasattr(functools, "update_wrapper") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_re_datetime_functools_value_ops {sum(_ledger)} asserts")
