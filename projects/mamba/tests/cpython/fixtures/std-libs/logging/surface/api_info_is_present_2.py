# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_info_is_present_2"
# subject = "logging.info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.info: api_info_is_present_2 (surface)."""
import logging

assert hasattr(logging, "info")
print("api_info_is_present_2 OK")
