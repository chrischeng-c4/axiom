# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "isleap_is_callable"
# subject = "calendar.isleap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.isleap: isleap_is_callable (surface)."""
import calendar

assert callable(calendar.isleap)
print("isleap_is_callable OK")
