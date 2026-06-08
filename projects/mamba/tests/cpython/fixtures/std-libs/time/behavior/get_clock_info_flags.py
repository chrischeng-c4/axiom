# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "get_clock_info_flags"
# subject = "time.get_clock_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.get_clock_info: get_clock_info reports the right monotonic/adjustable flags: 'monotonic' is monotonic & not adjustable, 'time' is not monotonic & adjustable, 'process_time' is monotonic & not adjustable"""
import time

_mono = time.get_clock_info("monotonic")
assert _mono.monotonic is True, "monotonic clock is monotonic"
assert _mono.adjustable is False, "monotonic clock is not adjustable"
_wall = time.get_clock_info("time")
assert _wall.monotonic is False, "wall clock is not monotonic"
assert _wall.adjustable is True, "wall clock is adjustable"
_proc = time.get_clock_info("process_time")
assert _proc.monotonic is True, "process_time is monotonic"
assert _proc.adjustable is False, "process_time is not adjustable"
print("get_clock_info_flags OK")
