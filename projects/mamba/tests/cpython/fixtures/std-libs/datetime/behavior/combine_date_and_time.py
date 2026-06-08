# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "combine_date_and_time"
# subject = "datetime.datetime.combine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.combine: datetime.combine(date, time) builds the combined instant whose .date() and .time() recover the parts"""
import datetime

d = datetime.date(2025, 3, 15)
t = datetime.time(10, 30, 45)
combined = datetime.datetime.combine(d, t)
assert combined == datetime.datetime(2025, 3, 15, 10, 30, 45), f"combine = {combined!r}"
assert combined.date() == d, "combined.date()"
assert combined.time() == t, "combined.time()"
print("combine_date_and_time OK")
