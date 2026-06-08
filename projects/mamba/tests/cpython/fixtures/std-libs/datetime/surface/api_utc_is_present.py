# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_utc_is_present"
# subject = "datetime.UTC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.UTC: api_utc_is_present (surface)."""
import datetime

assert hasattr(datetime, "UTC")
print("api_utc_is_present OK")
