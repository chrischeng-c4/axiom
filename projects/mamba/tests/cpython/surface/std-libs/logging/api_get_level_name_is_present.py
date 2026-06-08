# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_get_level_name_is_present"
# subject = "logging.getLevelName"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.getLevelName: api_get_level_name_is_present (surface)."""
import logging

assert hasattr(logging, "getLevelName")
print("api_get_level_name_is_present OK")
