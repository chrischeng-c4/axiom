# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_isoweekday_monday_one"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.isoweekday() is Monday=1..Sunday=7 (Jan2 2023 Mon=1, Jan8 Sun=7)"""
import datetime

assert datetime.date(2023, 1, 2).isoweekday() == 1, "Mon=1"
assert datetime.date(2023, 1, 8).isoweekday() == 7, "Sun=7"
print("date_isoweekday_monday_one OK")
