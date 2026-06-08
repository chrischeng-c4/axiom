# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "monthrange_negative_year_no_raise"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: monthrange accepts a negative (proleptic Gregorian) year; monthrange(-1, 1) does NOT raise"""
import calendar

print("negative_year:", calendar.monthrange(-1, 1))
print("monthrange_negative_year_no_raise OK")
