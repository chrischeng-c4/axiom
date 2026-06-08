# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_warning_is_present_2"
# subject = "logging.warning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.warning: api_warning_is_present_2 (surface)."""
import logging

assert hasattr(logging, "warning")
print("api_warning_is_present_2 OK")
