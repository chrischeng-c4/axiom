# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_time_is_present"
# subject = "logging.handlers.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.time: api_time_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "time")
print("api_time_is_present OK")
