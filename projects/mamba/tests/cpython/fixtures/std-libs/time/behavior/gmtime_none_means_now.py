# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_none_means_now"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: gmtime() and gmtime(None) both mean 'now': their mktime values differ by less than one second"""
import time

assert abs(time.mktime(time.gmtime()) - time.mktime(time.gmtime(None))) < 1.0, \
    "gmtime() == gmtime(None)"
print("gmtime_none_means_now OK")
