# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_debug_is_present"
# subject = "logging.DEBUG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.DEBUG: api_debug_is_present (surface)."""
import logging

assert hasattr(logging, "DEBUG")
print("api_debug_is_present OK")
