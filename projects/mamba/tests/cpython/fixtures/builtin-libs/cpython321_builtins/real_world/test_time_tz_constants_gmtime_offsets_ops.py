# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_time_tz_constants_gmtime_offsets_ops"
# subject = "cpython321.test_time_tz_constants_gmtime_offsets_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_time_tz_constants_gmtime_offsets_ops.py"
# status = "filled"
# ///
"""cpython321.test_time_tz_constants_gmtime_offsets_ops: execute CPython 3.12 seed test_time_tz_constants_gmtime_offsets_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 165: time module timezone constants + gmtime non-epoch offsets
#
# Covers `time` surface not yet asserted by test_time_ops /
# test_time_clocks_ops / test_time_ns_asctime_ctime_ops /
# test_time_strftime_struct_invariants_ops:
#   - module-level tz constants: time.timezone, time.altzone,
#     time.tzname, time.daylight
#   - gmtime() field values at non-epoch seconds (3661 = 01:01:01;
#     86400 = 1970-01-02) covering tm_hour/tm_min/tm_sec rollover
#     and tm_mday/tm_yday day-of-year increment
#   - strftime token coverage of %m, %d, %H, %M, %S against gmtime(0)
import time

_ledger = []

# --- module-level timezone constants ---
assert isinstance(time.timezone, int); _ledger.append(1)
assert isinstance(time.altzone, int); _ledger.append(1)
assert isinstance(time.tzname, tuple); _ledger.append(1)
assert len(time.tzname) == 2; _ledger.append(1)
assert isinstance(time.tzname[0], str); _ledger.append(1)
assert isinstance(time.tzname[1], str); _ledger.append(1)
assert time.daylight in (0, 1); _ledger.append(1)

# --- gmtime(0) field invariants at epoch ---
g0 = time.gmtime(0)
assert g0.tm_year == 1970; _ledger.append(1)
assert g0.tm_mon == 1; _ledger.append(1)
assert g0.tm_mday == 1; _ledger.append(1)
assert g0.tm_hour == 0; _ledger.append(1)
assert g0.tm_min == 0; _ledger.append(1)
assert g0.tm_sec == 0; _ledger.append(1)
assert g0.tm_wday == 3; _ledger.append(1)  # 1970-01-01 was Thursday (0=Mon)
assert g0.tm_yday == 1; _ledger.append(1)
assert g0.tm_isdst in (-1, 0, 1); _ledger.append(1)

# --- gmtime(3661) — hour/min/sec rollover within first day ---
g_hms = time.gmtime(3661)  # 1*3600 + 1*60 + 1
assert g_hms.tm_year == 1970; _ledger.append(1)
assert g_hms.tm_mon == 1; _ledger.append(1)
assert g_hms.tm_mday == 1; _ledger.append(1)
assert g_hms.tm_hour == 1; _ledger.append(1)
assert g_hms.tm_min == 1; _ledger.append(1)
assert g_hms.tm_sec == 1; _ledger.append(1)
assert g_hms.tm_yday == 1; _ledger.append(1)

# --- gmtime(86400) — exactly one day after epoch ---
g_day = time.gmtime(86400)
assert g_day.tm_year == 1970; _ledger.append(1)
assert g_day.tm_mon == 1; _ledger.append(1)
assert g_day.tm_mday == 2; _ledger.append(1)
assert g_day.tm_hour == 0; _ledger.append(1)
assert g_day.tm_min == 0; _ledger.append(1)
assert g_day.tm_sec == 0; _ledger.append(1)
assert g_day.tm_wday == 4; _ledger.append(1)  # 1970-01-02 was Friday
assert g_day.tm_yday == 2; _ledger.append(1)

# --- strftime token coverage against gmtime(0) ---
assert time.strftime("%Y", g0) == "1970"; _ledger.append(1)
assert time.strftime("%m", g0) == "01"; _ledger.append(1)
assert time.strftime("%d", g0) == "01"; _ledger.append(1)
assert time.strftime("%H", g0) == "00"; _ledger.append(1)
assert time.strftime("%M", g0) == "00"; _ledger.append(1)
assert time.strftime("%S", g0) == "00"; _ledger.append(1)
assert time.strftime("%H:%M:%S", g0) == "00:00:00"; _ledger.append(1)
assert time.strftime("%Y-%m-%d", g0) == "1970-01-01"; _ledger.append(1)

# --- strftime against gmtime(3661) — non-zero hh:mm:ss ---
assert time.strftime("%H", g_hms) == "01"; _ledger.append(1)
assert time.strftime("%M", g_hms) == "01"; _ledger.append(1)
assert time.strftime("%S", g_hms) == "01"; _ledger.append(1)
assert time.strftime("%H:%M:%S", g_hms) == "01:01:01"; _ledger.append(1)

# --- strftime against gmtime(86400) — day rollover ---
assert time.strftime("%d", g_day) == "02"; _ledger.append(1)
assert time.strftime("%Y-%m-%d", g_day) == "1970-01-02"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_time_tz_constants_gmtime_offsets_ops {sum(_ledger)} asserts")
