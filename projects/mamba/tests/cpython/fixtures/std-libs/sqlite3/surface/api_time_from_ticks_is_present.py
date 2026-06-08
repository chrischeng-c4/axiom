# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_time_from_ticks_is_present"
# subject = "sqlite3.TimeFromTicks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.TimeFromTicks: api_time_from_ticks_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "TimeFromTicks")
print("api_time_from_ticks_is_present OK")
