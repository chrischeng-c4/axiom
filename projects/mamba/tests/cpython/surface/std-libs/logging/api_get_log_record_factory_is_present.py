# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_log_record_factory_is_present"
# subject = "logging.getLogRecordFactory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getLogRecordFactory: api_get_log_record_factory_is_present (surface)."""
import logging

assert hasattr(logging, "getLogRecordFactory")
print("api_get_log_record_factory_is_present OK")
