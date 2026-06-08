# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_os_is_present"
# subject = "logging.handlers.os"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.os: api_os_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "os")
print("api_os_is_present OK")
