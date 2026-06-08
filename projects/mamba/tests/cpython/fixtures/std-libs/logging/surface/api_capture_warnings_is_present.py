# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_capture_warnings_is_present"
# subject = "logging.captureWarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.captureWarnings: api_capture_warnings_is_present (surface)."""
import logging

assert hasattr(logging, "captureWarnings")
print("api_capture_warnings_is_present OK")
