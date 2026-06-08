# Operational AssertionPass seed for `time.strftime` formatting.
# Surface: %Y/%m/%d/%H/%M/%S/%j/%A/%B format codes anchored on the
# epoch (1970-01-01 00:00:00 UTC) via time.gmtime(0), plus tm_*
# attribute access on struct_time.
# Companion to stub/test_strftime.py — vendored unittest seed.
import time
_ledger: list[int] = []
gm = time.gmtime(0)
assert gm.tm_year == 1970; _ledger.append(1)
assert gm.tm_mon == 1; _ledger.append(1)
assert gm.tm_mday == 1; _ledger.append(1)
assert gm.tm_hour == 0; _ledger.append(1)
assert gm.tm_min == 0; _ledger.append(1)
assert gm.tm_sec == 0; _ledger.append(1)
assert time.strftime("%Y-%m-%d", gm) == "1970-01-01"; _ledger.append(1)
assert time.strftime("%H:%M:%S", gm) == "00:00:00"; _ledger.append(1)
assert time.strftime("%Y", gm) == "1970"; _ledger.append(1)
assert time.strftime("%j", gm) == "001"; _ledger.append(1)
assert time.strftime("%A", gm) == "Thursday"; _ledger.append(1)
assert time.strftime("%B", gm) == "January"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_strftime_ops {sum(_ledger)} asserts")
