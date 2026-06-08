# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_handler_is_present"
# subject = "logging.Handler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.Handler: api_handler_is_present (surface)."""
import logging

assert hasattr(logging, "Handler")
print("api_handler_is_present OK")
