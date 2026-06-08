# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_asctime_mktime_consistent"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: for 'now', ctime(t) == asctime(localtime(t)) and int(mktime(localtime(t))) round-trips to int(t)"""
import time

_now = time.time()
assert time.ctime(_now) == time.asctime(time.localtime(_now)), \
    "ctime == asctime(localtime)"
assert int(time.mktime(time.localtime(_now))) == int(_now), \
    "mktime(localtime) round-trips to the same integer second"
print("ctime_asctime_mktime_consistent OK")
