# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "month_name_january"
# subject = "calendar.month_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.month_name: month_name[1] is the English name 'January'"""
import calendar

assert calendar.month_name[1] == "January", "month_name[1]"
print("month_name_january OK")
