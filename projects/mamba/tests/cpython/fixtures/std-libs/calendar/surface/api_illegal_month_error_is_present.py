# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_illegal_month_error_is_present"
# subject = "calendar.IllegalMonthError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.IllegalMonthError: api_illegal_month_error_is_present (surface)."""
import calendar

assert hasattr(calendar, "IllegalMonthError")
print("api_illegal_month_error_is_present OK")
