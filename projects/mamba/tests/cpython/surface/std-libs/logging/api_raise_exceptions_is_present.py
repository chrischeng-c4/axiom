# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_raise_exceptions_is_present"
# subject = "logging.raiseExceptions"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.raiseExceptions: api_raise_exceptions_is_present (surface)."""
import logging

assert hasattr(logging, "raiseExceptions")
print("api_raise_exceptions_is_present OK")
