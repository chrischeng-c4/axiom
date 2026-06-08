# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_copy_is_present"
# subject = "logging.handlers.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.copy: api_copy_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "copy")
print("api_copy_is_present OK")
