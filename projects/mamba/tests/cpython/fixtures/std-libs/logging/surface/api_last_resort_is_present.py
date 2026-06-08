# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_last_resort_is_present"
# subject = "logging.lastResort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.lastResort: api_last_resort_is_present (surface)."""
import logging

assert hasattr(logging, "lastResort")
print("api_last_resort_is_present OK")
