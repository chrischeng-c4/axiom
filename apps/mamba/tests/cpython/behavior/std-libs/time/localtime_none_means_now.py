# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "localtime_none_means_now"
# subject = "time.localtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.localtime: localtime() and localtime(None) both mean 'now': their mktime values differ by less than one second"""
import time

assert abs(time.mktime(time.localtime()) - time.mktime(time.localtime(None))) < 1.0, \
    "localtime() == localtime(None)"
print("localtime_none_means_now OK")
