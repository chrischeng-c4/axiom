# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "date_field_accessors"
# subject = "datetime.date"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date(2025,3,15) exposes .year/.month/.day; a second date(2000,1,2) confirms the accessors are per-instance"""
import datetime

d = datetime.date(2025, 3, 15)
assert d.year == 2025, f"year = {d.year!r}"
assert d.month == 3, f"month = {d.month!r}"
assert d.day == 15, f"day = {d.day!r}"
d2 = datetime.date(2000, 1, 2)
assert d2.year == 2000, f"d2 year = {d2.year!r}"
assert d2.month == 1, f"d2 month = {d2.month!r}"
assert d2.day == 2, f"d2 day = {d2.day!r}"
print("date_field_accessors OK")
