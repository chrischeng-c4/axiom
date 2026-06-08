# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_time_strftime_struct_invariants_ops"
# subject = "cpython321.test_time_strftime_struct_invariants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_time_strftime_struct_invariants_ops.py"
# status = "filled"
# ///
"""cpython321.test_time_strftime_struct_invariants_ops: execute CPython 3.12 seed test_time_strftime_struct_invariants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `time.strftime` formatting
# against an arbitrary input tuple, plus `time.localtime` returned
# `struct_time` range invariants. Existing `test_strftime_ops` /
# `test_time_clocks_ops` exercise only the gmtime(0) epoch case;
# this seed asserts that strftime renders custom calendar fields
# (year/month/day/hour/min/sec) into their zero-padded canonical
# substrings, and that `localtime()` returned fields fall inside
# the documented ranges (1≤mon≤12, 1≤mday≤31, 0≤hour≤23,
# 0≤min≤59, 0≤sec≤60, 0≤wday≤6, 1≤yday≤366).
import time
_ledger: list[int] = []

# strftime against an arbitrary calendar tuple
# (year, mon, mday, hour, min, sec, wday, yday, isdst) = Wed 2024-05-15 12:30:45
t = (2024, 5, 15, 12, 30, 45, 2, 136, 0)
assert time.strftime("%Y", t) == "2024"; _ledger.append(1)
assert time.strftime("%m", t) == "05"; _ledger.append(1)
assert time.strftime("%d", t) == "15"; _ledger.append(1)
assert time.strftime("%H", t) == "12"; _ledger.append(1)
assert time.strftime("%M", t) == "30"; _ledger.append(1)
assert time.strftime("%S", t) == "45"; _ledger.append(1)
assert time.strftime("%Y-%m-%d", t) == "2024-05-15"; _ledger.append(1)
assert time.strftime("%H:%M:%S", t) == "12:30:45"; _ledger.append(1)

# struct_time invariants on localtime()
lt = time.localtime()
assert isinstance(lt.tm_year, int); _ledger.append(1)
assert isinstance(lt.tm_mon, int); _ledger.append(1)
assert isinstance(lt.tm_mday, int); _ledger.append(1)
assert 1 <= lt.tm_mon <= 12; _ledger.append(1)
assert 1 <= lt.tm_mday <= 31; _ledger.append(1)
assert 0 <= lt.tm_hour <= 23; _ledger.append(1)
assert 0 <= lt.tm_min <= 59; _ledger.append(1)
assert 0 <= lt.tm_sec <= 60; _ledger.append(1)
assert 0 <= lt.tm_wday <= 6; _ledger.append(1)
assert 1 <= lt.tm_yday <= 366; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_time_strftime_struct_invariants_ops {sum(_ledger)} asserts")
