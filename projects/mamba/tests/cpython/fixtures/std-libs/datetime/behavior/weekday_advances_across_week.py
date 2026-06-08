# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "weekday_advances_across_week"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: weekday()/isoweekday() advance one step per day across a full week starting Mon 2002-03-04"""
import datetime

for i in range(7):
    assert datetime.datetime(2002, 3, 4 + i).weekday() == i, f"weekday {i}"
    assert datetime.datetime(2002, 3, 4 + i).isoweekday() == i + 1, f"isoweekday {i}"
print("weekday_advances_across_week OK")
