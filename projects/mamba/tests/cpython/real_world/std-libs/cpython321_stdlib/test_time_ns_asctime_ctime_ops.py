# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_time_ns_asctime_ctime_ops"
# subject = "cpython321.test_time_ns_asctime_ctime_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_time_ns_asctime_ctime_ops.py"
# status = "filled"
# ///
"""cpython321.test_time_ns_asctime_ctime_ops: execute CPython 3.12 seed test_time_ns_asctime_ctime_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for time-module surface not covered
# by `test_time_ops`, `test_time_clocks_ops`,
# `test_time_strftime_struct_invariants_ops`, or `test_strftime_ops`.
# Surface:
#   * monotonic_ns / perf_counter_ns — nanosecond-resolution
#     counterparts of monotonic / perf_counter, returning int.
#   * asctime / ctime — render a struct_time / timestamp to the
#     canonical "Day Mon dd HH:MM:SS YYYY" string.
#   * struct_time.tm_isdst — daylight-saving flag (0, 1, or -1).
import time
_ledger: list[int] = []

# monotonic_ns — nanosecond counter, non-decreasing
n1 = time.monotonic_ns()
n2 = time.monotonic_ns()
assert n2 >= n1; _ledger.append(1)
assert n1 > 0; _ledger.append(1)
assert n2 > 0; _ledger.append(1)

# perf_counter_ns — nanosecond counter, non-decreasing
p1 = time.perf_counter_ns()
p2 = time.perf_counter_ns()
assert p2 >= p1; _ledger.append(1)
assert p1 > 0; _ledger.append(1)
assert p2 > 0; _ledger.append(1)

# time_ns — wall-clock nanoseconds since epoch
t_ns = time.time_ns()
assert t_ns > 0; _ledger.append(1)
# Same order of magnitude as time() — both larger than 1 billion (post-2001)
assert t_ns > 1e9; _ledger.append(1)

# asctime on the epoch — string ending with the year 1970
epoch_st = time.gmtime(0)
s = time.asctime(epoch_st)
assert isinstance(s, str); _ledger.append(1)
assert "1970" in s; _ledger.append(1)

# ctime on the epoch timestamp 0 — also produces a string with "1970"
c = time.ctime(0)
assert isinstance(c, str); _ledger.append(1)
assert "1970" in c; _ledger.append(1)

# struct_time fields on the epoch
assert epoch_st.tm_year == 1970; _ledger.append(1)
assert epoch_st.tm_mon == 1; _ledger.append(1)
assert epoch_st.tm_mday == 1; _ledger.append(1)
assert epoch_st.tm_hour == 0; _ledger.append(1)
assert epoch_st.tm_min == 0; _ledger.append(1)
assert epoch_st.tm_sec == 0; _ledger.append(1)
# Jan 1, 1970 was a Thursday; tm_wday is 0=Monday so Thu == 3
assert epoch_st.tm_wday == 3; _ledger.append(1)
# tm_yday is 1-based day-of-year; Jan 1 == 1
assert epoch_st.tm_yday == 1; _ledger.append(1)

# tm_isdst is in {-1, 0, 1}
assert epoch_st.tm_isdst in (-1, 0, 1); _ledger.append(1)

# sleep(0) — non-blocking smoke test
time.sleep(0)
assert True; _ledger.append(1)

# Coherence: time_ns / 1e9 is in the same order of magnitude as time()
t_float = time.time()
ratio = t_ns / 1e9
# They should agree to within a couple of seconds
assert abs(ratio - t_float) < 5; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_time_ns_asctime_ctime_ops {sum(_ledger)} asserts")
