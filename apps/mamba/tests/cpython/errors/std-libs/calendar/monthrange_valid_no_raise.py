# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_valid_no_raise"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: the happy path: monthrange(2024, 2) returns (weekday_of_first, num_days) without raising"""
import calendar

print("monthrange:", calendar.monthrange(2024, 2))
print("monthrange_valid_no_raise OK")
