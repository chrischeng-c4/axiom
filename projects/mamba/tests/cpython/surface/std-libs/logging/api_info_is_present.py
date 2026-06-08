# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_info_is_present"
# subject = "logging.INFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.INFO: api_info_is_present (surface)."""
import logging

assert hasattr(logging, "INFO")
print("api_info_is_present OK")
