# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_nt_event_log_handler_is_present"
# subject = "logging.handlers.NTEventLogHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.NTEventLogHandler: api_nt_event_log_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "NTEventLogHandler")
print("api_nt_event_log_handler_is_present OK")
