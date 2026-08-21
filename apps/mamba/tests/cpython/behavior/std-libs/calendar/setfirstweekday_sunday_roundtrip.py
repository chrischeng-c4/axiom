# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "setfirstweekday_sunday_roundtrip"
# subject = "calendar.setfirstweekday"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday(SUNDAY) is observable via firstweekday(), then restored to the original"""
import calendar

orig = calendar.firstweekday()
calendar.setfirstweekday(calendar.SUNDAY)
assert calendar.firstweekday() == calendar.SUNDAY, 'firstweekday Sunday'
calendar.setfirstweekday(orig)
print("setfirstweekday_sunday_roundtrip OK")
