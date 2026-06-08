# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_stream_request_handler_is_present"
# subject = "logging.config.StreamRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.StreamRequestHandler: api_stream_request_handler_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "StreamRequestHandler")
print("api_stream_request_handler_is_present OK")
