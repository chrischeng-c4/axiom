# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_div_and_floordiv"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.timedelta: timedelta / unit-timedelta is a float ratio and // is the floored int (whole and fractional cases)"""
import datetime

t = datetime.timedelta(hours=1, minutes=24, seconds=19)
second = datetime.timedelta(seconds=1)
assert t / second == 5059.0, f"truediv = {t / second!r}"
assert t // second == 5059, f"floordiv = {t // second!r}"

t = datetime.timedelta(minutes=2, seconds=30)
minute = datetime.timedelta(minutes=1)
assert t / minute == 2.5, f"truediv frac = {t / minute!r}"
assert t // minute == 2, f"floordiv frac = {t // minute!r}"
print("timedelta_div_and_floordiv OK")
