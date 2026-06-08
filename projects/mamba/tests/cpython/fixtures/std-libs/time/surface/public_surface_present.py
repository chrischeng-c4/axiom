# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "public_surface_present"
# subject = "time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time: all 36 typeshed public entries (clock constants, clock readers, conversion fns, tz globals, struct_time) resolve via getattr(time, name)"""
import time

surface = [
    'CLOCK_MONOTONIC', 'CLOCK_MONOTONIC_RAW', 'CLOCK_PROCESS_CPUTIME_ID',
    'CLOCK_REALTIME', 'CLOCK_THREAD_CPUTIME_ID', 'CLOCK_UPTIME_RAW',
    'altzone', 'asctime', 'clock_getres', 'clock_gettime',
    'clock_gettime_ns', 'clock_settime', 'clock_settime_ns', 'ctime',
    'daylight', 'get_clock_info', 'gmtime', 'localtime', 'mktime',
    'monotonic', 'monotonic_ns', 'perf_counter', 'perf_counter_ns',
    'process_time', 'process_time_ns', 'sleep', 'strftime', 'strptime',
    'struct_time', 'thread_time', 'thread_time_ns', 'time', 'time_ns',
    'timezone', 'tzname', 'tzset',
]
missing = [name for name in surface if not hasattr(time, name)]
assert len(surface) == 36, f"surface count = {len(surface)}"
assert missing == [], f"missing entries = {missing!r}"
print("public_surface_present OK")
