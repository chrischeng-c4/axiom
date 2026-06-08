# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_error_is_present_2"
# subject = "logging.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.error: api_error_is_present_2 (surface)."""
import logging

assert hasattr(logging, "error")
print("api_error_is_present_2 OK")
