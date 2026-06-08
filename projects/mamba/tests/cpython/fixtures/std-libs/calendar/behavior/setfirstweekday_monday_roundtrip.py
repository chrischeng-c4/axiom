# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "setfirstweekday_monday_roundtrip"
# subject = "calendar.setfirstweekday"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday(MONDAY) is observable via firstweekday(), then restored to the original"""
import calendar

orig = calendar.firstweekday()
calendar.setfirstweekday(calendar.MONDAY)
assert calendar.firstweekday() == calendar.MONDAY, 'firstweekday Monday'
calendar.setfirstweekday(orig)
print("setfirstweekday_monday_roundtrip OK")
