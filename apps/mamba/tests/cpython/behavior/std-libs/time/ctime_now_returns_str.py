# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_now_returns_str"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: ctime() and ctime(None) both use the current time and return a str without raising"""
import time

assert isinstance(time.ctime(), str), "ctime() returns str"
assert isinstance(time.ctime(None), str), "ctime(None) returns str"
print("ctime_now_returns_str OK")
