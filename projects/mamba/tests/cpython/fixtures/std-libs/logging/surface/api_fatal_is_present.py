# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_fatal_is_present"
# subject = "logging.FATAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.FATAL: api_fatal_is_present (surface)."""
import logging

assert hasattr(logging, "FATAL")
print("api_fatal_is_present OK")
