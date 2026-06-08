# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_date_is_present"
# subject = "datetime.date"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.date: api_date_is_present (surface)."""
import datetime

assert hasattr(datetime, "date")
print("api_date_is_present OK")
