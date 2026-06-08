# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_weekday_monday_zero"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date.weekday() is Monday=0..Sunday=6 (Jan2 2023 Mon=0, Jan7 Sat=5, Jan8 Sun=6)"""
import datetime

assert datetime.date(2023, 1, 2).weekday() == 0, "Mon=0"
assert datetime.date(2023, 1, 7).weekday() == 5, "Sat=5"
assert datetime.date(2023, 1, 8).weekday() == 6, "Sun=6"
print("date_weekday_monday_zero OK")
