# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "real_world"
# case = "surface_walkthrough"
# subject = "time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time: a downstream consumer drives a clock/conversion flow end-to-end: read a wall clock, format with strftime, parse it back with strptime, round-trip through mktime/localtime, and time a workload with monotonic/perf_counter — asserting each step"""
import time

# 1. Format a fixed UTC instant and round-trip it through strptime.
fixed = time.gmtime(1_700_000_000)
stamp = time.strftime("%Y-%m-%d %H:%M:%S", fixed)
assert isinstance(stamp, str) and len(stamp) == 19, f"stamp = {stamp!r}"
reparsed = time.strptime(stamp, "%Y-%m-%d %H:%M:%S")
assert reparsed.tm_year == fixed.tm_year, "strftime/strptime round-trip year"
assert reparsed.tm_mon == fixed.tm_mon, "round-trip month"
assert reparsed.tm_mday == fixed.tm_mday, "round-trip day"
assert reparsed.tm_hour == fixed.tm_hour, "round-trip hour"

# 2. localtime <-> mktime round-trip for the current instant.
now = time.time()
lt = time.localtime(now)
assert int(time.mktime(lt)) == int(now), "mktime(localtime(now)) round-trips"
assert time.ctime(now) == time.asctime(lt), "ctime == asctime(localtime)"

# 3. Time a small workload with monotonic and perf_counter; both advance.
t0_mono = time.monotonic()
p0 = time.perf_counter()
work = 0
for i in range(50_000):
    work += i
t1_mono = time.monotonic()
p1 = time.perf_counter()
assert work == 50_000 * 49_999 // 2, f"workload result = {work}"
assert t1_mono >= t0_mono, "monotonic did not go backward"
assert p1 >= p0, "perf_counter did not go backward"

# 4. time_ns and time agree to within a second.
assert abs(time.time_ns() - time.time() * 1_000_000_000) < 1_000_000_000, "time_ns ~ time"
print("surface_walkthrough OK")
