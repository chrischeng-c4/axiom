# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_warning_is_present"
# subject = "logging.WARNING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.WARNING: api_warning_is_present (surface)."""
import logging

assert hasattr(logging, "WARNING")
print("api_warning_is_present OK")
