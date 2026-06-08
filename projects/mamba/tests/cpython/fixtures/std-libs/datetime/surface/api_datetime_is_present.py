# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_datetime_is_present"
# subject = "datetime.datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.datetime: api_datetime_is_present (surface)."""
import datetime

assert hasattr(datetime, "datetime")
print("api_datetime_is_present OK")
