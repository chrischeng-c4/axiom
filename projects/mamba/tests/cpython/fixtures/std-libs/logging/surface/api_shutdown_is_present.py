# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_shutdown_is_present"
# subject = "logging.shutdown"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.shutdown: api_shutdown_is_present (surface)."""
import logging

assert hasattr(logging, "shutdown")
print("api_shutdown_is_present OK")
