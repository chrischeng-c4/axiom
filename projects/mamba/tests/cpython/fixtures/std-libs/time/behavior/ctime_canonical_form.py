# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "ctime_canonical_form"
# subject = "time.ctime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.ctime: ctime renders a fixed timestamp in the 24-char canonical form: 1973-09-16 01:03:52 -> 'Sun Sep 16 01:03:52 1973' and 2000-01-01 -> 'Sat Jan  1 00:00:00 2000' (struct built with tm_isdst=-1 so mktime resolves DST)"""
import time

# Use tm_isdst=-1 so mktime picks the right DST for the local zone.
_c1 = time.ctime(time.mktime((1973, 9, 16, 1, 3, 52, 0, 0, -1)))
assert _c1 == "Sun Sep 16 01:03:52 1973", f"ctime 1973 = {_c1!r}"
_c2 = time.ctime(time.mktime((2000, 1, 1, 0, 0, 0, 0, 0, -1)))
assert _c2 == "Sat Jan  1 00:00:00 2000", f"ctime 2000 = {_c2!r}"
print("ctime_canonical_form OK")
