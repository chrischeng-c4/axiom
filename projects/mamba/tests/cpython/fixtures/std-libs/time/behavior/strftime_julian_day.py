# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "strftime_julian_day"
# subject = "time.strftime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.strftime: %j is the zero-padded day-of-year: nine days after the epoch is Jan 10, tm_yday==10, strftime('%j') == '010'"""
import time

_jan10 = time.gmtime(9 * 86400)  # 9 days after epoch = Jan 10
assert _jan10.tm_yday == 10, f"yday = {_jan10.tm_yday!r}"
_jfmt = time.strftime("%j", _jan10)
assert _jfmt == "010", f"strftime %%j = {_jfmt!r}"
print("strftime_julian_day OK")
