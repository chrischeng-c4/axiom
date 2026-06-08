# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_stream_handler_is_present"
# subject = "logging.StreamHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.StreamHandler: api_stream_handler_is_present (surface)."""
import logging

assert hasattr(logging, "StreamHandler")
print("api_stream_handler_is_present OK")
