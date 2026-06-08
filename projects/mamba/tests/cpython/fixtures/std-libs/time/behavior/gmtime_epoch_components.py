# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_epoch_components"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: time.gmtime(0) is the unix epoch 1970-01-01 00:00:00 UTC: year/mon/mday/hour/min/sec and tm_wday=3 (Thu) tm_yday=1"""
import time

_epoch = time.gmtime(0)
assert _epoch.tm_year == 1970, f"epoch year = {_epoch.tm_year!r}"
assert _epoch.tm_mon == 1, f"epoch month = {_epoch.tm_mon!r}"
assert _epoch.tm_mday == 1, f"epoch mday = {_epoch.tm_mday!r}"
assert _epoch.tm_hour == 0, f"epoch hour = {_epoch.tm_hour!r}"
assert _epoch.tm_min == 0, f"epoch min = {_epoch.tm_min!r}"
assert _epoch.tm_sec == 0, f"epoch sec = {_epoch.tm_sec!r}"
assert _epoch.tm_wday == 3, f"epoch wday (Thu=3) = {_epoch.tm_wday!r}"
assert _epoch.tm_yday == 1, f"epoch yday = {_epoch.tm_yday!r}"
print("gmtime_epoch_components OK")
