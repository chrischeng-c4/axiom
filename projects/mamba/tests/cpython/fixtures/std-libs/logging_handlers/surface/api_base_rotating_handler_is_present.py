# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_base_rotating_handler_is_present"
# subject = "logging.handlers.BaseRotatingHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.BaseRotatingHandler: api_base_rotating_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "BaseRotatingHandler")
print("api_base_rotating_handler_is_present OK")
