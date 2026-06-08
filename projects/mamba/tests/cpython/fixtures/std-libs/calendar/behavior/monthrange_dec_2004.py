# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "monthrange_dec_2004"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: December 2004 -> monthrange (2, 31)"""
import calendar

assert calendar.monthrange(2004, 12) == (2, 31), 'monthrange Dec'
print("monthrange_dec_2004 OK")
