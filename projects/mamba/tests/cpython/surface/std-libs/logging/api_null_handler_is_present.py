# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_null_handler_is_present"
# subject = "logging.NullHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.NullHandler: api_null_handler_is_present (surface)."""
import logging

assert hasattr(logging, "NullHandler")
print("api_null_handler_is_present OK")
