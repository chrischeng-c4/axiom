# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_logger_is_present"
# subject = "logging.getLogger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getLogger: api_get_logger_is_present (surface)."""
import logging

assert hasattr(logging, "getLogger")
print("api_get_logger_is_present OK")
