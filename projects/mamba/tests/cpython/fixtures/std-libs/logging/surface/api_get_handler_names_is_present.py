# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_handler_names_is_present"
# subject = "logging.getHandlerNames"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getHandlerNames: api_get_handler_names_is_present (surface)."""
import logging

assert hasattr(logging, "getHandlerNames")
print("api_get_handler_names_is_present OK")
