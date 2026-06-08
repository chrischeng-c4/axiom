# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_timedelta_is_present"
# subject = "datetime.timedelta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.timedelta: api_timedelta_is_present (surface)."""
import datetime

assert hasattr(datetime, "timedelta")
print("api_timedelta_is_present OK")
