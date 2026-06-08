# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_ns_returns_positive_int"
# subject = "time.time_ns"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.time_ns: time.time_ns() returns an int of nanoseconds since the epoch, greater than zero, consistent with time.time() to within one second"""
import time

_ns = time.time_ns()
assert isinstance(_ns, int), f"time_ns type = {type(_ns)!r}"
assert _ns > 0, f"time_ns > 0: {_ns!r}"

# time_ns is close to time() * 1e9.
_t = time.time()
_diff = abs(_ns - _t * 1_000_000_000)
assert _diff < 1_000_000_000, f"time_ns ~ time*1e9, diff={_diff}"
print("time_ns_returns_positive_int OK")
