# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_epoch_datetime"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strftime: strftime('%Y-%m-%d %H:%M:%S', gmtime(0)) renders the epoch as '1970-01-01 00:00:00'"""
import time

_fmt = time.strftime("%Y-%m-%d %H:%M:%S", time.gmtime(0))
assert _fmt == "1970-01-01 00:00:00", f"strftime = {_fmt!r}"
print("strftime_epoch_datetime OK")
