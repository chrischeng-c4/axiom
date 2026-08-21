# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "today_is_callable"
# subject = "datetime.date.today"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date.today: today_is_callable (surface)."""
import datetime

assert callable(datetime.date.today)
print("today_is_callable OK")
