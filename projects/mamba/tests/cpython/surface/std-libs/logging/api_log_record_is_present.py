# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_log_record_is_present"
# subject = "logging.LogRecord"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.LogRecord: api_log_record_is_present (surface)."""
import logging

assert hasattr(logging, "LogRecord")
print("api_log_record_is_present OK")
