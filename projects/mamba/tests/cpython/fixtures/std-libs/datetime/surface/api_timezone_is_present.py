# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_timezone_is_present"
# subject = "datetime.timezone"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.timezone: api_timezone_is_present (surface)."""
import datetime

assert hasattr(datetime, "timezone")
print("api_timezone_is_present OK")
