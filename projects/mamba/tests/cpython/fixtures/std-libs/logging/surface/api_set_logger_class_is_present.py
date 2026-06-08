# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_set_logger_class_is_present"
# subject = "logging.setLoggerClass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.setLoggerClass: api_set_logger_class_is_present (surface)."""
import logging

assert hasattr(logging, "setLoggerClass")
print("api_set_logger_class_is_present OK")
