# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_sys_log_handler_is_present"
# subject = "logging.handlers.SysLogHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.SysLogHandler: api_sys_log_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "SysLogHandler")
print("api_sys_log_handler_is_present OK")
