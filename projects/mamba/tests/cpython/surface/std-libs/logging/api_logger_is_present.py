# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_logger_is_present"
# subject = "logging.Logger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.Logger: api_logger_is_present (surface)."""
import logging

assert hasattr(logging, "Logger")
print("api_logger_is_present OK")
