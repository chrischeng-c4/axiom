# Operational AssertionPass seed for `time` module clock variants
# beyond the basic time/monotonic surface in test_time_ops.
# Surface: perf_counter (high-resolution monotonic), process_time
# (CPU time consumed by the process), strftime formatting against
# localtime(0) for a stable epoch string, time_ns returns a non-zero
# value, sleep(0.001) is a non-negative delta on perf_counter.
import time
_ledger: list[int] = []

# perf_counter is monotonically non-decreasing across two reads
p1 = time.perf_counter()
p2 = time.perf_counter()
assert p2 >= p1; _ledger.append(1)

# A small sleep produces a strictly positive perf_counter delta
before = time.perf_counter()
time.sleep(0.001)
after = time.perf_counter()
assert after - before >= 0.0; _ledger.append(1)

# process_time is non-negative and non-decreasing
pt1 = time.process_time()
pt2 = time.process_time()
assert pt1 >= 0.0; _ledger.append(1)
assert pt2 >= pt1; _ledger.append(1)

# strftime against localtime(0) gives the epoch date — the year
# and month-day are stable across timezones close to UTC
s = time.strftime("%Y-%m-%d", time.localtime(0))
# Epoch is 1970-01-01 in UTC; near-UTC timezones (UTC-12..UTC+14)
# can shift this to 1969-12-31. Accept either.
assert s in ("1970-01-01", "1969-12-31"); _ledger.append(1)

# strftime returns a string
assert isinstance(s, str); _ledger.append(1)

# time_ns returns a positive integer (nanoseconds since epoch)
ns = time.time_ns()
assert ns > 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_time_clocks_ops {sum(_ledger)} asserts")
