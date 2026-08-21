# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "date_is_callable"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date: date_is_callable (surface)."""
import datetime

assert callable(datetime.date)
print("date_is_callable OK")
