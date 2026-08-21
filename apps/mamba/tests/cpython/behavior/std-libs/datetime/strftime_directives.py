# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "strftime_directives"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: strftime honors %Y/%m/%d on a date and %H:%M on a datetime"""
import datetime

d = datetime.date(2023, 6, 15)
assert d.strftime("%Y/%m/%d") == "2023/06/15", f"strftime date = {d.strftime('%Y/%m/%d')!r}"
dt = datetime.datetime(2023, 6, 15, 12, 30, 45)
assert dt.strftime("%H:%M") == "12:30", f"strftime time = {dt.strftime('%H:%M')!r}"
print("strftime_directives OK")
