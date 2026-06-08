# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "timezone_is_callable"
# subject = "datetime.timezone"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timezone: timezone_is_callable (surface)."""
import datetime

assert callable(datetime.timezone)
print("timezone_is_callable OK")
