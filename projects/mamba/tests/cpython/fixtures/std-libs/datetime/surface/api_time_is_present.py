# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_time_is_present"
# subject = "datetime.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.time: api_time_is_present (surface)."""
import datetime

assert hasattr(datetime, "time")
print("api_time_is_present OK")
