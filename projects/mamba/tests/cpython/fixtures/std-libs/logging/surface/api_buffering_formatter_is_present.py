# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_buffering_formatter_is_present"
# subject = "logging.BufferingFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.BufferingFormatter: api_buffering_formatter_is_present (surface)."""
import logging

assert hasattr(logging, "BufferingFormatter")
print("api_buffering_formatter_is_present OK")
