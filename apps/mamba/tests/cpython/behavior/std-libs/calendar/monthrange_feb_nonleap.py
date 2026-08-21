# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "monthrange_feb_nonleap"
# subject = "calendar.monthrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.monthrange: February of non-leap year 2010 has 28 days -> monthrange (0, 28)"""
import calendar

assert calendar.monthrange(2010, 2) == (0, 28), 'monthrange Feb nonleap'
print("monthrange_feb_nonleap OK")
