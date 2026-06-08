# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_123_echoes_value"
# subject = "calendar.setfirstweekday"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday(123) raises IllegalWeekdayError whose message text echoes the offending weekday number 123"""
import calendar

try:
    calendar.setfirstweekday(123)
    print("weekday123: no_raise")
except calendar.IllegalWeekdayError as e:
    print("weekday123:", type(e).__name__, "echoes 123:", "123" in str(e))
print("setfirstweekday_123_echoes_value OK")
