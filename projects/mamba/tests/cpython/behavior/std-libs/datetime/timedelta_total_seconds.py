# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "timedelta_total_seconds"
# subject = "datetime.timedelta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta(days=1,hours=1,minutes=1,seconds=1).total_seconds() == 86400+3600+60+1 as a float"""
import datetime

td = datetime.timedelta(days=1, hours=1, minutes=1, seconds=1)
expected = 86400 + 3600 + 60 + 1
assert td.total_seconds() == float(expected), f"total_seconds = {td.total_seconds()!r}"
print("timedelta_total_seconds OK")
