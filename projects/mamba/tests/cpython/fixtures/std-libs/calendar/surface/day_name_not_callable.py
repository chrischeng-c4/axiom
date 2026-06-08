# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "day_name_not_callable"
# subject = "calendar.day_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.day_name: day_name_not_callable (surface)."""
import calendar

assert not callable(calendar.day_name)
print("day_name_not_callable OK")
