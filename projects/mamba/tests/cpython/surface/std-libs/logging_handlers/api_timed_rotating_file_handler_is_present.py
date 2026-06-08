# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_timed_rotating_file_handler_is_present"
# subject = "logging.handlers.TimedRotatingFileHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.TimedRotatingFileHandler: api_timed_rotating_file_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "TimedRotatingFileHandler")
print("api_timed_rotating_file_handler_is_present OK")
