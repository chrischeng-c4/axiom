# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "weekday_2024_jan1_monday"
# subject = "calendar.weekday"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.weekday: weekday(2024, 1, 1) == 0 (Monday, Mon=0)"""
import calendar

assert str(calendar.weekday(2024, 1, 1)) == '0', 'calendar.weekday(2024, 1, 1)'
print("weekday_2024_jan1_monday OK")
