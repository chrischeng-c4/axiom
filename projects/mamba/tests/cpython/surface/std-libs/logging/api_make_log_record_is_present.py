# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_make_log_record_is_present"
# subject = "logging.makeLogRecord"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.makeLogRecord: api_make_log_record_is_present (surface)."""
import logging

assert hasattr(logging, "makeLogRecord")
print("api_make_log_record_is_present OK")
