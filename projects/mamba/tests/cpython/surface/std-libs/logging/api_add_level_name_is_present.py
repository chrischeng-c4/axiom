# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_add_level_name_is_present"
# subject = "logging.addLevelName"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.addLevelName: api_add_level_name_is_present (surface)."""
import logging

assert hasattr(logging, "addLevelName")
print("api_add_level_name_is_present OK")
