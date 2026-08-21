# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_returns_positive_float"
# subject = "time.time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.time: time.time() returns a float of seconds since the unix epoch, greater than 1e9"""
import time

_t = time.time()
assert isinstance(_t, float), f"time() type = {type(_t)!r}"
assert _t > 1_000_000_000.0, f"time() > 1e9: {_t!r}"
print("time_returns_positive_float OK")
