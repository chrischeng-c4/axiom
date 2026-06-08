# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_time_is_present_2"
# subject = "sqlite3.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.time: api_time_is_present_2 (surface)."""
import sqlite3

assert hasattr(sqlite3, "time")
print("api_time_is_present_2 OK")
