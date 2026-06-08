# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "timedelta_is_callable"
# subject = "datetime.timedelta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta_is_callable (surface)."""
import datetime

assert callable(datetime.timedelta)
print("timedelta_is_callable OK")
