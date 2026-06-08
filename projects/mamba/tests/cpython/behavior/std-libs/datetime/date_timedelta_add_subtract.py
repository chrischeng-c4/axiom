# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_timedelta_add_subtract"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date + timedelta rolls into the next month (Jan31+1=Feb1) and date - timedelta rolls back (Mar1-1=Feb28 in non-leap 2023)"""
import datetime

plus = datetime.date(2023, 1, 31) + datetime.timedelta(days=1)
assert plus == datetime.date(2023, 2, 1), f"jan31 + 1 = {plus!r}"
# 2023 is not a leap year, so Mar1 - 1 day is Feb 28.
minus = datetime.date(2023, 3, 1) - datetime.timedelta(days=1)
assert minus == datetime.date(2023, 2, 28), f"mar1 - 1 = {minus!r}"
print("date_timedelta_add_subtract OK")
