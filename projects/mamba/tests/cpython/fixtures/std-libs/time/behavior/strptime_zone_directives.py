# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strptime_zone_directives"
# subject = "time.strptime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime understands %Z (zone name -> tm_zone == 'UTC') and %z (offset -> tm_gmtoff == 5*3600)"""
import time

assert time.strptime("UTC", "%Z").tm_zone == "UTC", "strptime %Z -> tm_zone"
assert time.strptime("+0500", "%z").tm_gmtoff == 5 * 3600, "strptime %z -> tm_gmtoff"
print("strptime_zone_directives OK")
