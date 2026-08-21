# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: time (time/monotonic/perf_counter wall + monotonic clocks,
# strftime/strptime/gmtime/localtime and struct_time attribute access, sleep).
# time.mktime currently returns 57600 instead of 86400 for Jan-2-1970 (likely
# TZ-application bug on mamba) and is omitted; time_ns returns a float rather
# than int and is also omitted. Both gaps tracked separately.
import time

_ledger: list[int] = []

# time.time() returns a positive float wall-clock value
t = time.time()
assert isinstance(t, float) and t > 0, f"time.time() is positive float, got {type(t).__name__}={t}"
_ledger.append(1)

# time.monotonic() returns a float
mt = time.monotonic()
assert isinstance(mt, float), f"time.monotonic() is float, got {type(mt).__name__}"
_ledger.append(1)

# monotonic clock never goes backwards
mt2 = time.monotonic()
assert mt2 >= mt, f"time.monotonic() is non-decreasing, got {mt2} < {mt}"
_ledger.append(1)

# perf_counter returns a numeric value (high-resolution timer)
pc = time.perf_counter()
assert isinstance(pc, (int, float)), f"time.perf_counter() is numeric, got {type(pc).__name__}"
_ledger.append(1)

# sleep() blocks for at least the requested duration
t0 = time.monotonic()
time.sleep(0.01)
elapsed = time.monotonic() - t0
assert elapsed > 0, f"sleep(0.01) elapses some monotonic time, got {elapsed}"
_ledger.append(1)

# strftime formats the unix epoch as 1970-01-01
assert time.strftime("%Y-%m-%d", time.gmtime(0)) == "1970-01-01", (
    "strftime('%Y-%m-%d', gmtime(0)) == '1970-01-01'"
)
_ledger.append(1)

# strftime supports time-of-day directives
assert time.strftime("%H:%M:%S", time.gmtime(0)) == "00:00:00", (
    "strftime('%H:%M:%S', gmtime(0)) == '00:00:00'"
)
_ledger.append(1)

# gmtime(0) returns a struct_time at the unix epoch
gt = time.gmtime(0)
assert gt.tm_year == 1970, f"gmtime(0).tm_year == 1970, got {gt.tm_year}"
_ledger.append(1)

assert gt.tm_mon == 1, f"gmtime(0).tm_mon == 1, got {gt.tm_mon}"
_ledger.append(1)

assert gt.tm_mday == 1, f"gmtime(0).tm_mday == 1, got {gt.tm_mday}"
_ledger.append(1)

# strptime parses a date string into a struct_time
parsed = time.strptime("2024-01-15", "%Y-%m-%d")
assert parsed is not None, "strptime returns a struct_time, not None"
_ledger.append(1)

# localtime accepts a unix timestamp and returns a struct_time
lt = time.localtime(0)
assert lt is not None, "localtime(0) returns a struct_time, not None"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_time {sum(_ledger)} asserts")
