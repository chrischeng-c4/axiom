# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_logger_class_is_present"
# subject = "logging.getLoggerClass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getLoggerClass: api_get_logger_class_is_present (surface)."""
import logging

assert hasattr(logging, "getLoggerClass")
print("api_get_logger_class_is_present OK")
