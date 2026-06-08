# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_maxyear_is_present"
# subject = "datetime.MAXYEAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.MAXYEAR: api_maxyear_is_present (surface)."""
import datetime

assert hasattr(datetime, "MAXYEAR")
print("api_maxyear_is_present OK")
