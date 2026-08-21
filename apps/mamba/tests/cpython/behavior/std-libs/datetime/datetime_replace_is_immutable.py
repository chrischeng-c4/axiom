# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "datetime_replace_is_immutable"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime: datetime.replace() returns a new datetime with changed fields and leaves the original unchanged"""
import datetime

dt2 = datetime.datetime(2023, 6, 15, 12, 0, 0)
dt3 = dt2.replace(hour=18, minute=30)
assert dt3 == datetime.datetime(2023, 6, 15, 18, 30, 0), f"replace = {dt3!r}"
assert dt2 == datetime.datetime(2023, 6, 15, 12, 0, 0), "original unchanged"
print("datetime_replace_is_immutable OK")
