# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_logger_adapter_is_present"
# subject = "logging.LoggerAdapter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.LoggerAdapter: api_logger_adapter_is_present (surface)."""
import logging

assert hasattr(logging, "LoggerAdapter")
print("api_logger_adapter_is_present OK")
