# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "day_name_monday"
# subject = "calendar.day_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.day_name: day_name[0] is the English name 'Monday' (Mon=0)"""
import calendar

assert calendar.day_name[0] == "Monday", "day_name[0]"
print("day_name_monday OK")
