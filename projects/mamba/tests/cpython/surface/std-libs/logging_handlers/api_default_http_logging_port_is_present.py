# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_default_http_logging_port_is_present"
# subject = "logging.handlers.DEFAULT_HTTP_LOGGING_PORT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.DEFAULT_HTTP_LOGGING_PORT: api_default_http_logging_port_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "DEFAULT_HTTP_LOGGING_PORT")
print("api_default_http_logging_port_is_present OK")
