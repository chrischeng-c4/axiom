# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "isocalendar_triple"
# subject = "datetime.datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/datetimetester.py"
# status = "filled"
# ///
"""datetime.datetime: datetime(2019,1,1).isocalendar() == (2019, 1, 2) — the (ISO year, week, weekday) triple"""
import datetime

assert datetime.datetime(2019, 1, 1).isocalendar() == (2019, 1, 2), "isocalendar"
print("isocalendar_triple OK")
