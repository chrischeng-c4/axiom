# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_smtp_handler_is_present"
# subject = "logging.handlers.SMTPHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.SMTPHandler: api_smtp_handler_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "SMTPHandler")
print("api_smtp_handler_is_present OK")
