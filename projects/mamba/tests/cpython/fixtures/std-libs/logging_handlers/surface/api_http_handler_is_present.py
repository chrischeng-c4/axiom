# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_http_handler_is_present"
# subject = "logging.handlers.HTTPHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.HTTPHandler: api_http_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "HTTPHandler")
print("api_http_handler_is_present OK")
